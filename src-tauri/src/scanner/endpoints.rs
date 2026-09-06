use crate::models::{Category, EndpointReport, Finding, Severity};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use url::Url;

#[derive(Debug, Clone)]
struct BaselineProbe {
    is_soft_404: bool,
    content_len: usize,
    title: Option<String>,
}

#[derive(Clone, Copy)]
enum CheckType {
    EnvFile,
    GitHead,
    GitConfig,
    SqlDump,
    SqliteDb,
    PhpInfo,
    SwaggerJson,
    SymfonyProfiler,
    LaravelDebugbar,
    LaravelTelescope,
    ElmahLog,
    AdminPortal(&'static str),
    MetricsPrometheus,
    SpringActuatorHealth,
    ConfigBackup,
    DockerCompose,
}

#[derive(Clone)]
struct PathProbeDef {
    path: &'static str,
    id: &'static str,
    title: &'static str,
    severity: Severity,
    description: &'static str,
    impact: &'static str,
    remediation: &'static str,
    owasp_category: &'static str,
    cve_id: Option<&'static str>,
    check_type: CheckType,
    references: &'static [&'static str],
}

pub async fn audit_endpoints(client: &Client, base_url: &Url) -> (EndpointReport, Vec<Finding>) {
    let mut report = EndpointReport::default();
    let mut findings = Vec::new();

    // 1. Audit /robots.txt
    if let Ok(robots_url) = base_url.join("/robots.txt") {
        if let Ok(resp) = client.get(robots_url.as_str()).timeout(Duration::from_secs(4)).send().await {
            if resp.status().is_success() {
                report.robots_txt_found = true;
                if let Ok(body) = resp.text().await {
                    let mut disallowed = Vec::new();
                    let mut sensitive_disallowed = Vec::new();

                    let sensitive_keywords = [
                        "admin",
                        "administrator",
                        "api",
                        "internal",
                        "staging",
                        "secret",
                        "backup",
                        "config",
                        "database",
                        "db",
                        "private",
                        ".git",
                        ".env",
                        "wp-admin",
                        "dashboard",
                    ];

                    for line in body.lines() {
                        let trimmed = line.trim();
                        if trimmed.to_lowercase().starts_with("disallow:") {
                            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                let path = parts[1].trim().to_string();
                                if !path.is_empty() && path != "/" {
                                    disallowed.push(path.clone());
                                    let path_lower = path.to_lowercase();
                                    if sensitive_keywords.iter().any(|k| path_lower.contains(k)) {
                                        sensitive_disallowed.push(path);
                                    }
                                }
                            }
                        }
                    }

                    if !sensitive_disallowed.is_empty() {
                        findings.push(Finding {
                            id: "robots-sensitive-paths-exposed".to_string(),
                            title: "Sensitive Administrative/Internal Endpoints Disclosed in robots.txt".to_string(),
                            severity: Severity::Medium,
                            category: Category::EndpointExposure,
                            description: format!(
                                "The robots.txt file discloses {} potentially sensitive administrative or internal paths (e.g., {}). Attackers actively inspect robots.txt to discover hidden assets.",
                                sensitive_disallowed.len(),
                                sensitive_disallowed.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                            ),
                            impact: "Disclosing hidden directories aids malicious actors during reconnaissance to pinpoint admin portals, API endpoints, and staging environments.".to_string(),
                            remediation: "Do not rely on robots.txt for security or access control. Enforce strong authentication and IP allowlisting on sensitive administrative routes.".to_string(),
                            evidence: Some(sensitive_disallowed.join("\n")),
                            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                            cve_id: None,
                            references: vec![
                                "https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/03-Review_Webserver_Metafiles_for_Information_Leakage".to_string(),
                            ],
                        });
                    }

                    report.disallowed_paths = disallowed;
                    report.sensitive_disallowed_paths = sensitive_disallowed;
                }
            }
        }
    }

    // 2. Audit /.well-known/security.txt
    if let Ok(sec_url) = base_url.join("/.well-known/security.txt") {
        if let Ok(resp) = client.get(sec_url.as_str()).timeout(Duration::from_secs(4)).send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.text().await {
                    if body.to_lowercase().contains("contact:") {
                        report.security_txt_found = true;
                        report.security_txt_content = Some(body);
                    }
                }
            }
        }
    }

    if !report.security_txt_found {
        findings.push(Finding {
            id: "missing-security-txt".to_string(),
            title: "Missing security.txt Security Disclosure Policy (RFC 9116)".to_string(),
            severity: Severity::Info,
            category: Category::EndpointExposure,
            description: "No RFC 9116 security.txt file was found at /.well-known/security.txt. A security.txt file helps ethical security researchers responsibly report vulnerabilities directly to your security team.".to_string(),
            impact: "Vulnerability disclosures may be delayed or misdirected if security researchers cannot find your designated disclosure contacts.".to_string(),
            remediation: "Deploy a security.txt file under the /.well-known/ directory specifying Contact, Expires, and Encryption keys according to RFC 9116.".to_string(),
            evidence: None,
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec![
                "https://securitytxt.org/".to_string(),
                "https://www.rfc-editor.org/rfc/rfc9116".to_string(),
            ],
        });
    }

    // 3. Calibrate Soft-404 / Catch-All SPA Baseline (False Positive Prevention)
    let baseline = probe_soft_404_baseline(client, base_url).await;

    // 4. Audit High-Risk Exposed Vulnerability Paths
    let probes = get_vulnerability_path_definitions();
    report.scanned_paths_count = probes.len();

    let path_findings = probe_vulnerable_paths(client, base_url, &baseline, probes).await;
    report.exposed_paths_count = path_findings.len();
    findings.extend(path_findings);

    (report, findings)
}

/// Probes a randomized non-existent path to detect wildcard routing, Single Page Applications,
/// and custom 200 OK soft-404 pages. This baseline is essential for eliminating false positives.
async fn probe_soft_404_baseline(client: &Client, base_url: &Url) -> BaselineProbe {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(987654321);
    let rand_path = format!("/_vr_probe_nonexistent_{:x}", nanos);

    let probe_url = match base_url.join(&rand_path) {
        Ok(u) => u,
        Err(_) => {
            return BaselineProbe {
                is_soft_404: false,
                content_len: 0,
                title: None,
            }
        }
    };

    let resp = match client.get(probe_url.as_str()).timeout(Duration::from_secs(4)).send().await {
        Ok(r) => r,
        Err(_) => {
            return BaselineProbe {
                is_soft_404: false,
                content_len: 0,
                title: None,
            }
        }
    };

    let status = resp.status().as_u16();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    let body = resp.text().await.unwrap_or_default();
    let content_len = body.len();
    let title = extract_html_title(&body);

    let is_soft_404 = status == 200 || status == 206 || (status < 400 && content_type.contains("html"));

    BaselineProbe {
        is_soft_404,
        content_len,
        title,
    }
}

fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start_tag = "<title>";
    let end_tag = "</title>";
    if let Some(start_idx) = lower.find(start_tag) {
        if let Some(end_idx) = lower[start_idx..].find(end_tag) {
            let title = &html[start_idx + start_tag.len()..start_idx + end_idx];
            return Some(title.trim().to_string());
        }
    }
    None
}

fn is_html_response(content_type: &str, body: &str) -> bool {
    let lower_ct = content_type.to_lowercase();
    if lower_ct.contains("text/html") || lower_ct.contains("application/xhtml") {
        return true;
    }
    let trimmed = body.trim_start();
    let lower_start: String = trimmed.chars().take(200).collect::<String>().to_lowercase();
    lower_start.starts_with("<!doctype html")
        || lower_start.starts_with("<html")
        || lower_start.starts_with("<?xml")
        || lower_start.contains("<head>")
        || lower_start.contains("<body>")
}

fn matches_baseline(
    baseline: &BaselineProbe,
    content_type: &str,
    body: &str,
    final_url: &Url,
    base_url: &Url,
) -> bool {
    // Discard if redirected back to root or home
    if final_url.path() == "/" || (final_url.path() == base_url.path() && final_url.path() != "") {
        return true;
    }

    if baseline.is_soft_404 {
        // Discard if HTML title matches the baseline non-existent page title
        if let (Some(b_title), Some(p_title)) = (&baseline.title, extract_html_title(body)) {
            if !b_title.is_empty() && b_title == &p_title {
                return true;
            }
        }

        // Discard if response length is almost identical to the baseline catch-all page and is HTML
        let diff = (body.len() as isize - baseline.content_len as isize).abs();
        if diff < 100 && is_html_response(content_type, body) {
            return true;
        }
    }

    false
}

fn verify_signature(check: CheckType, content_type: &str, body: &str) -> (bool, Option<String>) {
    let is_html = is_html_response(content_type, body);

    match check {
        CheckType::EnvFile => {
            if is_html {
                return (false, None);
            }
            let env_indicators = [
                "DB_PASSWORD=",
                "DATABASE_URL=",
                "AWS_SECRET",
                "APP_KEY=",
                "SECRET_KEY=",
                "JWT_SECRET=",
                "MYSQL_ROOT_PASSWORD=",
                "POSTGRES_PASSWORD=",
                "REDIS_PASSWORD=",
                "MAIL_PASSWORD=",
                "STRIPE_SECRET=",
                "API_KEY=",
                "DB_HOST=",
                "DB_USERNAME=",
                "DATABASE_PASSWORD=",
                "PRIVATE_KEY=",
            ];
            let mut matched = Vec::new();
            for ind in env_indicators {
                if body.contains(ind) {
                    matched.push(ind);
                }
            }
            if matched.len() >= 2
                || body.contains("DB_PASSWORD=")
                || body.contains("APP_KEY=")
                || body.contains("DATABASE_URL=")
            {
                let sample: Vec<&str> = body
                    .lines()
                    .take(6)
                    .filter(|l| !l.trim().is_empty())
                    .collect();
                return (
                    true,
                    Some(format!(
                        "Disclosed environment variable directives:\n{}",
                        sample.join("\n")
                    )),
                );
            }
            (false, None)
        }

        CheckType::GitHead => {
            if is_html {
                return (false, None);
            }
            let trimmed = body.trim();
            if trimmed.starts_with("ref: refs/heads/") {
                return (true, Some(format!("Git HEAD ref: {}", trimmed)));
            }
            if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                return (true, Some(format!("Git commit SHA-1: {}", trimmed)));
            }
            (false, None)
        }

        CheckType::GitConfig => {
            if is_html {
                return (false, None);
            }
            if body.contains("[core]")
                && (body.contains("repositoryformatversion")
                    || body.contains("remote \"origin\"")
                    || body.contains("filemode"))
            {
                let sample: Vec<&str> = body.lines().take(6).collect();
                return (
                    true,
                    Some(format!("Git repository config:\n{}", sample.join("\n"))),
                );
            }
            (false, None)
        }

        CheckType::SqlDump => {
            if is_html {
                return (false, None);
            }
            if body.contains("CREATE TABLE ")
                || body.contains("INSERT INTO ")
                || body.contains("-- MySQL dump")
                || body.contains("-- PostgreSQL database dump")
                || body.contains("/*!40101 SET @OLD_CHARACTER_SET_CLIENT")
            {
                let sample: Vec<&str> = body.lines().take(4).collect();
                return (
                    true,
                    Some(format!("SQL dump statements:\n{}", sample.join("\n"))),
                );
            }
            (false, None)
        }

        CheckType::SqliteDb => {
            if body.starts_with("SQLite format 3")
                || content_type.contains("sqlite")
                || content_type.contains("application/x-sqlite3")
            {
                return (
                    true,
                    Some("SQLite 3 database binary header validated.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::PhpInfo => {
            if body.contains("PHP Version")
                && (body.contains("Configuration File (php.ini) Path")
                    || body.contains("Zend Engine")
                    || body.contains("phpinfo()"))
            {
                return (
                    true,
                    Some("PHP runtime configuration info page disclosed.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::SwaggerJson => {
            if (content_type.contains("json") || body.trim_start().starts_with('{'))
                && (body.contains("\"swagger\":")
                    || body.contains("\"openapi\":")
                    || (body.contains("\"paths\":") && body.contains("\"info\":")))
            {
                return (
                    true,
                    Some("OpenAPI / Swagger JSON API specification exposed.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::SymfonyProfiler => {
            if body.contains("Symfony Profiler")
                || body.contains("sf-toolbar")
                || body.contains("class=\"sf-dump\"")
            {
                return (
                    true,
                    Some("Active Symfony Web Profiler interface detected.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::LaravelDebugbar => {
            if body.contains("phpdebugbar") || body.contains("Laravel Debugbar") {
                return (
                    true,
                    Some("Laravel Debugbar diagnostic assets detected.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::LaravelTelescope => {
            if body.contains("Laravel Telescope") || body.contains("telescope-data") {
                return (
                    true,
                    Some("Laravel Telescope diagnostic dashboard accessible.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::ElmahLog => {
            if body.contains("Error Log for ")
                && (body.contains("ELMAH") || body.contains("System.Web.HttpUnhandledException"))
            {
                return (
                    true,
                    Some("ASP.NET ELMAH unhandled exception log accessible.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::AdminPortal(token) => {
            if is_html && body.contains(token) {
                return (
                    true,
                    Some(format!(
                        "Administrative login portal confirmed (matched '{}').",
                        token
                    )),
                );
            }
            (false, None)
        }

        CheckType::MetricsPrometheus => {
            if !is_html && (body.contains("# HELP ") || body.contains("# TYPE ")) {
                let sample: Vec<&str> = body.lines().take(4).collect();
                return (
                    true,
                    Some(format!("Prometheus metrics:\n{}", sample.join("\n"))),
                );
            }
            (false, None)
        }

        CheckType::SpringActuatorHealth => {
            if (content_type.contains("json") || body.trim_start().starts_with('{'))
                && body.contains("\"status\"")
                && (body.contains("\"UP\"")
                    || body.contains("\"DOWN\"")
                    || body.contains("\"components\""))
            {
                return (
                    true,
                    Some("Spring Boot Actuator health telemetry status exposed.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::ConfigBackup => {
            if !is_html
                && (body.contains("DB_NAME")
                    || body.contains("DB_PASSWORD")
                    || body.contains("AUTH_KEY")
                    || (body.contains("<?php") && body.contains("define(")))
            {
                return (
                    true,
                    Some("Raw PHP configuration backup file exposed.".to_string()),
                );
            }
            (false, None)
        }

        CheckType::DockerCompose => {
            if !is_html
                && body.contains("services:")
                && (body.contains("image:")
                    || body.contains("ports:")
                    || body.contains("environment:"))
            {
                return (
                    true,
                    Some("Docker Compose container definition file exposed.".to_string()),
                );
            }
            (false, None)
        }
    }
}

async fn probe_vulnerable_paths(
    client: &Client,
    base_url: &Url,
    baseline: &BaselineProbe,
    probes: Vec<PathProbeDef>,
) -> Vec<Finding> {
    let semaphore = Arc::new(Semaphore::new(5));
    let mut set = JoinSet::new();

    for probe in probes {
        let client = client.clone();
        let base_url = base_url.clone();
        let baseline = baseline.clone();
        let sem = semaphore.clone();

        set.spawn(async move {
            let _permit = sem.acquire().await.ok()?;
            let target_url = base_url.join(probe.path).ok()?;

            let resp = client
                .get(target_url.as_str())
                .timeout(Duration::from_secs(4))
                .send()
                .await
                .ok()?;

            let status = resp.status();
            if !status.is_success() {
                return None;
            }

            let status_code = status.as_u16();
            let final_url = resp.url().clone();
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("")
                .to_lowercase();

            let body = resp.text().await.unwrap_or_default();

            // Anti-false-positive check: compare against baseline soft-404
            if matches_baseline(&baseline, &content_type, &body, &final_url, &base_url) {
                return None;
            }

            // Verify positive content signature
            let (is_confirmed, evidence_detail) =
                verify_signature(probe.check_type, &content_type, &body);
            if !is_confirmed {
                return None;
            }

            let mut evidence_lines = vec![
                format!("URL: {}", target_url),
                format!("HTTP Status: {}", status_code),
                format!("Content-Type: {}", content_type),
            ];
            if let Some(detail) = evidence_detail {
                evidence_lines.push(String::new());
                evidence_lines.push(detail);
            }

            Some(Finding {
                id: probe.id.to_string(),
                title: probe.title.to_string(),
                severity: probe.severity,
                category: Category::EndpointExposure,
                description: probe.description.to_string(),
                impact: probe.impact.to_string(),
                remediation: probe.remediation.to_string(),
                evidence: Some(evidence_lines.join("\n")),
                owasp_category: probe.owasp_category.to_string(),
                cve_id: probe.cve_id.map(|s| s.to_string()),
                references: probe.references.iter().map(|s| s.to_string()).collect(),
            })
        });
    }

    let mut findings = Vec::new();
    while let Some(res) = set.join_next().await {
        if let Ok(Some(finding)) = res {
            findings.push(finding);
        }
    }

    findings
}

fn get_vulnerability_path_definitions() -> Vec<PathProbeDef> {
    vec![
        // 1. Environment & Secret Configuration Files
        PathProbeDef {
            path: "/.env",
            id: "exposed-env-file",
            title: "Critical Environment Configuration File Exposed (.env)",
            severity: Severity::Critical,
            description: "A production environment file (.env) is publicly accessible at the web root without authentication. Environment files contain plaintext infrastructure credentials, database passwords, cloud tokens, and application secrets.",
            impact: "Unauthenticated threat actors can download this file to harvest database passwords, AWS/Stripe API keys, JWT secret salts, and mail server credentials, allowing direct backend database takeover, data exfiltration, or lateral movement into cloud infrastructure.",
            remediation: "Immediately deny HTTP access to hidden dot-files in your web server configuration (e.g., in Nginx: 'location ~ /\\.(?!well-known).* { deny all; }' or in Apache: '<FilesMatch \"^\\.\"> Require all denied </FilesMatch>'). Move all sensitive environment files outside the document root and immediately rotate all disclosed credentials.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::EnvFile,
            references: &[
                "https://owasp.org/www-project-top-ten/2017/A6_2017-Security_Misconfiguration",
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },
        PathProbeDef {
            path: "/.env.local",
            id: "exposed-env-local",
            title: "Local Environment Configuration File Exposed (.env.local)",
            severity: Severity::Critical,
            description: "A local environment file (.env.local) was deployed to the public web root. Local environment files frequently contain developer API keys, database connection strings, and debug passwords.",
            impact: "Attackers can extract private keys, test database passwords, and third-party API credentials, compromising backend services and sensitive records.",
            remediation: "Remove .env.local from the production web root and configure your web server to reject requests to all dot-prefixed configuration files.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::EnvFile,
            references: &[
                "https://owasp.org/www-project-top-ten/2017/A6_2017-Security_Misconfiguration",
            ],
        },
        PathProbeDef {
            path: "/.env.production",
            id: "exposed-env-production",
            title: "Production Environment Secrets File Exposed (.env.production)",
            severity: Severity::Critical,
            description: "The production-specific environment secrets file (.env.production) is publicly exposed over HTTP.",
            impact: "Allows immediate exfiltration of live production database credentials and third-party payment/cloud access keys.",
            remediation: "Block HTTP access to .env files in the web server configuration and rotate all compromised production secrets immediately.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::EnvFile,
            references: &[
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },

        // 2. Git Version Control Repositories
        PathProbeDef {
            path: "/.git/HEAD",
            id: "exposed-git-head",
            title: "Exposed Git Version Control Repository (/.git/HEAD)",
            severity: Severity::High,
            description: "The application exposes its Git version control metadata directory over HTTP. Using standard tools such as git-dumper, attackers can traverse the object tree and reconstruct the complete application repository source code, commit history, and branches.",
            impact: "Attackers can review the full application source code to discover zero-day business logic flaws, hardcoded credentials, hidden internal API endpoints, and private developer communications.",
            remediation: "Deny access to the /.git/ folder in your web server (e.g., Nginx: 'location ~ /\\.git { deny all; }') and ensure production build artifacts do not include development version control folders.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::GitHead,
            references: &[
                "https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/05-Review_Webpage_Content_for_Information_Leakage",
                "https://cwe.mitre.org/data/definitions/538.html",
            ],
        },
        PathProbeDef {
            path: "/.git/config",
            id: "exposed-git-config",
            title: "Git Repository Configuration File Disclosed (/.git/config)",
            severity: Severity::High,
            description: "The Git configuration file (/.git/config) is publicly readable, disclosing internal repository paths, remote Git URLs, branch names, and potentially embedded access tokens.",
            impact: "Leads to source repository mapping, disclosure of private repository hostnames, and potential credential leakage if remote URLs contain embedded HTTP authentication tokens.",
            remediation: "Block all incoming requests targeting /.git/ paths on the reverse proxy or web server.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::GitConfig,
            references: &[
                "https://cwe.mitre.org/data/definitions/538.html",
            ],
        },

        // 3. Database Dumps & Backup Archives
        PathProbeDef {
            path: "/backup.sql",
            id: "exposed-backup-sql",
            title: "Database SQL Dump File Publicly Accessible (/backup.sql)",
            severity: Severity::Critical,
            description: "A raw database SQL dump file is publicly downloadable from the web root.",
            impact: "Malicious users can download the entire application database, containing user tables, password hashes, personal identifiable information (PII), session tokens, and business records.",
            remediation: "Remove the SQL dump file immediately from the web server. Store all database backups in secure, access-controlled offline or private cloud storage (e.g. S3 with private ACL).",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::SqlDump,
            references: &[
                "https://owasp.org/www-project-top-ten/2017/A3_2017-Sensitive_Data_Exposure",
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },
        PathProbeDef {
            path: "/dump.sql",
            id: "exposed-dump-sql",
            title: "Database Dump File Publicly Accessible (/dump.sql)",
            severity: Severity::Critical,
            description: "A database export file (/dump.sql) is accessible without authentication.",
            impact: "Permits complete database extraction, leading to customer credential compromise, privacy violations, and regulatory penalties.",
            remediation: "Delete public dump files from the web server and audit server directories for lingering database backup files.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::SqlDump,
            references: &[
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },
        PathProbeDef {
            path: "/database.sqlite",
            id: "exposed-sqlite-db",
            title: "SQLite Database File Directly Accessible (/database.sqlite)",
            severity: Severity::Critical,
            description: "An embedded SQLite database file is located directly inside the public document root.",
            impact: "Attackers can download the SQLite database file and read all application tables, credentials, and data using standard SQLite client tools.",
            remediation: "Move the SQLite database file outside the public web root directory (e.g. into /var/data/ or a dedicated non-served storage directory).",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::SqliteDb,
            references: &[
                "https://cwe.mitre.org/data/definitions/538.html",
            ],
        },
        PathProbeDef {
            path: "/wp-config.php.bak",
            id: "exposed-wp-config-bak",
            title: "WordPress Configuration Backup File Exposed (/wp-config.php.bak)",
            severity: Severity::Critical,
            description: "A backup of the WordPress configuration file is publicly downloadable. Because the .bak extension is not processed by PHP engines, the server serves the raw source code containing database credentials and authentication keys.",
            impact: "Attackers obtain DB_NAME, DB_USER, DB_PASSWORD, and WordPress auth salt keys, facilitating direct database compromise and session forgery.",
            remediation: "Delete all .bak, .old, and .save files from the web server. Configure the web server to deny requests to backup extensions.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::ConfigBackup,
            references: &[
                "https://cwe.mitre.org/data/definitions/538.html",
            ],
        },
        PathProbeDef {
            path: "/docker-compose.yml",
            id: "exposed-docker-compose",
            title: "Docker Compose Infrastructure Definition Exposed",
            severity: Severity::High,
            description: "A Docker Compose orchestration file is exposed at the web root, detailing container services, volume mounts, internal network ports, and environment passwords.",
            impact: "Provides an attacker with a complete blueprint of backend microservices, database container names, internal subnet IPs, and hardcoded container credentials.",
            remediation: "Remove docker-compose.yml from the public web root and restrict web deployments to compiled production assets only.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::DockerCompose,
            references: &[
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },

        // 4. Debuggers, Profilers & Diagnostic Consoles
        PathProbeDef {
            path: "/phpinfo.php",
            id: "exposed-phpinfo",
            title: "PHP Diagnostic Info Page Publicly Exposed (phpinfo.php)",
            severity: Severity::High,
            description: "The phpinfo() diagnostic function is accessible over HTTP without authentication. It exposes PHP version, loaded extensions, file paths, OS details, and environment variables.",
            impact: "Discloses internal server architecture and path mappings, which attackers leverage to identify vulnerable modules, construct local file inclusion exploits, and bypass security filters.",
            remediation: "Remove phpinfo diagnostic files from production environments and disable sensitive information functions in php.ini.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::PhpInfo,
            references: &[
                "https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/02-Fingerprint_Web_Server",
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },
        PathProbeDef {
            path: "/_profiler/",
            id: "exposed-symfony-profiler",
            title: "Symfony Web Profiler Development Console Exposed",
            severity: Severity::High,
            description: "The Symfony Web Profiler is active and accessible on a production route. The profiler logs HTTP headers, user session tokens, database queries with parameters, and internal exception traces.",
            impact: "Enables unauthenticated users to inspect sensitive requests made by other users, steal session cookies, and review SQL queries.",
            remediation: "Disable the Symfony Web Profiler in production configuration (ensure APP_ENV=prod and APP_DEBUG=0).",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::SymfonyProfiler,
            references: &[
                "https://symfony.com/doc/current/profiler.html",
                "https://cwe.mitre.org/data/definitions/489.html",
            ],
        },
        PathProbeDef {
            path: "/_debugbar/assets/stylesheets",
            id: "exposed-laravel-debugbar",
            title: "Laravel Debugbar Development Utility Exposed",
            severity: Severity::Medium,
            description: "Laravel Debugbar assets are publicly accessible. When active, Laravel Debugbar exposes database execution times, query parameters, route definitions, and auth states.",
            impact: "Discloses internal application routing, query performance, and configuration details to potential adversaries.",
            remediation: "Ensure 'DEBUGBAR_ENABLED=false' and 'APP_DEBUG=false' in production .env configuration.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::LaravelDebugbar,
            references: &[
                "https://cwe.mitre.org/data/definitions/489.html",
            ],
        },
        PathProbeDef {
            path: "/telescope/requests",
            id: "exposed-laravel-telescope",
            title: "Laravel Telescope Diagnostic Dashboard Exposed",
            severity: Severity::High,
            description: "Laravel Telescope dashboard is accessible without proper authorization gates, exposing incoming requests, queued jobs, logs, database queries, and notifications.",
            impact: "Unauthenticated actors can monitor real-time application requests, view user payloads, and read sensitive job parameters.",
            remediation: "Implement strict authorization gates in TelescopeServiceProvider or disable Telescope in production environments.",
            owasp_category: "A01:2021-Broken Access Control",
            cve_id: None,
            check_type: CheckType::LaravelTelescope,
            references: &[
                "https://laravel.com/docs/telescope",
                "https://cwe.mitre.org/data/definitions/284.html",
            ],
        },
        PathProbeDef {
            path: "/elmah.axd",
            id: "exposed-elmah-console",
            title: "ASP.NET ELMAH Error Log Console Exposed",
            severity: Severity::High,
            description: "The ASP.NET ELMAH (Error Logging Modules and Handlers) console is accessible without authentication, exposing detailed unhandled server exception logs and stack traces.",
            impact: "Leaking stack traces, internal SQL errors, and request payloads allows attackers to understand code paths and target unhandled failure states.",
            remediation: "Restrict access to elmah.axd in web.config using authorization rules and ensure remote access is disabled (security.allowRemoteAccess=0).",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::ElmahLog,
            references: &[
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },

        // 5. API Documentation & OpenAPI Specifications
        PathProbeDef {
            path: "/swagger.json",
            id: "exposed-swagger-json",
            title: "OpenAPI / Swagger JSON Specification Publicly Accessible (/swagger.json)",
            severity: Severity::Medium,
            description: "A complete OpenAPI/Swagger JSON specification is publicly exposed at /swagger.json, describing all application API routes, input parameters, schemas, and authorization requirements.",
            impact: "Enables attackers to map the full API surface, locate undocumented internal endpoints, and automate targeted fuzzing and Broken Object Level Authorization (BOLA/IDOR) exploits.",
            remediation: "Disable or protect public access to Swagger/OpenAPI documentation endpoints in production environments unless the API is intentionally public.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::SwaggerJson,
            references: &[
                "https://owasp.org/www-project-api-security/",
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },
        PathProbeDef {
            path: "/openapi.json",
            id: "exposed-openapi-json",
            title: "OpenAPI JSON Specification Exposed (/openapi.json)",
            severity: Severity::Medium,
            description: "The OpenAPI 3.0 specification is publicly accessible at /openapi.json.",
            impact: "Aids adversaries in discovering internal APIs, request payloads, and parameter types for targeted exploitation.",
            remediation: "Require authentication or restrict IP access for API documentation endpoints in production.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::SwaggerJson,
            references: &[
                "https://owasp.org/www-project-api-security/",
            ],
        },
        PathProbeDef {
            path: "/v2/api-docs",
            id: "exposed-springfox-api-docs",
            title: "SpringFox Swagger API Documentation Exposed (/v2/api-docs)",
            severity: Severity::Medium,
            description: "The SpringFox Swagger endpoint (/v2/api-docs) exposes internal Spring REST controller definitions.",
            impact: "Discloses internal endpoints, data transfer objects, and controller methods to unauthenticated callers.",
            remediation: "Disable SpringFox Swagger in production profiles or secure the /v2/api-docs endpoint with Spring Security.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::SwaggerJson,
            references: &[
                "https://owasp.org/www-project-api-security/",
            ],
        },

        // 6. Administrative Portals Exposed to Public Internet
        PathProbeDef {
            path: "/wp-admin/",
            id: "exposed-wp-admin",
            title: "WordPress Administrative Portal Exposed (/wp-admin/)",
            severity: Severity::Medium,
            description: "The WordPress administrative portal (/wp-admin/) is reachable from the public internet without network restrictions.",
            impact: "Permits unauthenticated attackers to perform automated brute-force password guessing, credential stuffing, and user enumeration attacks against administrative accounts.",
            remediation: "Restrict /wp-admin/ and /wp-login.php access to trusted IP ranges or a corporate VPN, and enforce multi-factor authentication (MFA) on all admin users.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::AdminPortal("user_login"),
            references: &[
                "https://owasp.org/www-community/controls/Blocking_Brute_Force_Attacks",
                "https://cwe.mitre.org/data/definitions/284.html",
            ],
        },
        PathProbeDef {
            path: "/phpmyadmin/",
            id: "exposed-phpmyadmin",
            title: "phpMyAdmin Database Management Interface Exposed",
            severity: Severity::High,
            description: "The phpMyAdmin web interface is accessible on a public route without firewall restrictions.",
            impact: "Exposes database login credentials to brute-force attacks and exposes the server to known phpMyAdmin software vulnerabilities that can lead to remote code execution.",
            remediation: "Never expose phpMyAdmin directly to the public internet. Restrict access via VPN, SSH tunnel, or strict web server IP allowlists.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::AdminPortal("phpMyAdmin"),
            references: &[
                "https://www.phpmyadmin.net/security/",
                "https://cwe.mitre.org/data/definitions/284.html",
            ],
        },

        // 7. Telemetry & Metrics
        PathProbeDef {
            path: "/metrics",
            id: "exposed-prometheus-metrics",
            title: "Prometheus / Infrastructure Metrics Telemetry Exposed (/metrics)",
            severity: Severity::Low,
            description: "An operational Prometheus metrics endpoint (/metrics) is publicly accessible without authentication.",
            impact: "Leaks server process statistics, active JVM thread pools, HTTP request counts, internal microservice hostnames, and database connection numbers, assisting attackers in reconnaissance.",
            remediation: "Restrict access to the /metrics endpoint to internal monitoring scrapers (Prometheus) via firewall rules or reverse proxy authentication.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::MetricsPrometheus,
            references: &[
                "https://prometheus.io/docs/operating/security/",
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },
        PathProbeDef {
            path: "/actuator/health",
            id: "exposed-actuator-health",
            title: "Spring Boot Actuator Health Telemetry Exposed (/actuator/health)",
            severity: Severity::Low,
            description: "The Spring Boot Actuator /actuator/health endpoint is accessible without authentication.",
            impact: "Discloses internal system component health, database connectivity status, and disk space details.",
            remediation: "Set 'management.endpoint.health.show-details=never' in application.properties or secure actuator endpoints with Spring Security.",
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type: CheckType::SpringActuatorHealth,
            references: &[
                "https://docs.spring.io/spring-boot/docs/current/reference/html/actuator.html",
                "https://cwe.mitre.org/data/definitions/200.html",
            ],
        },
    ]
}
