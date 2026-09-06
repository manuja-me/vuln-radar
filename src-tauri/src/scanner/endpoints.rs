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

impl PathProbeDef {
    fn new(
        path: &'static str,
        id: &'static str,
        title: &'static str,
        severity: Severity,
        description: &'static str,
        impact: &'static str,
        remediation: &'static str,
        check_type: CheckType,
        references: &'static [&'static str],
    ) -> Self {
        Self {
            path,
            id,
            title,
            severity,
            description,
            impact,
            remediation,
            owasp_category: "A05:2021-Security Misconfiguration",
            cve_id: None,
            check_type,
            references,
        }
    }

    fn with_owasp(mut self, owasp: &'static str) -> Self {
        self.owasp_category = owasp;
        self
    }
}

pub async fn audit_endpoints(client: &Client, base_url: &Url) -> (EndpointReport, Vec<Finding>) {
    let mut report = EndpointReport::default();
    let mut findings = Vec::new();

    // 1. Run Pre-flight Probes Concurrently: /robots.txt, /.well-known/security.txt, and Soft-404 Baseline
    let robots_fut = async {
        if let Ok(robots_url) = base_url.join("/robots.txt") {
            if let Ok(resp) = client.get(robots_url.as_str()).timeout(Duration::from_secs(4)).send().await {
                if resp.status().is_success() {
                    return resp.text().await.ok();
                }
            }
        }
        None
    };

    let sec_fut = async {
        if let Ok(sec_url) = base_url.join("/.well-known/security.txt") {
            if let Ok(resp) = client.get(sec_url.as_str()).timeout(Duration::from_secs(4)).send().await {
                if resp.status().is_success() {
                    return resp.text().await.ok();
                }
            }
        }
        None
    };

    let (robots_body_opt, sec_body_opt, baseline) = tokio::join!(
        robots_fut,
        sec_fut,
        probe_soft_404_baseline(client, base_url)
    );

    // Process /robots.txt
    if let Some(body) = robots_body_opt {
        report.robots_txt_found = true;
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
            findings.push(
                Finding::new(
                    "robots-sensitive-paths-exposed",
                    "Sensitive Administrative/Internal Endpoints Disclosed in robots.txt",
                    Severity::Medium,
                    Category::EndpointExposure,
                    format!(
                        "The robots.txt file discloses {} potentially sensitive administrative or internal paths (e.g., {}). Attackers actively inspect robots.txt to discover hidden assets.",
                        sensitive_disallowed.len(),
                        sensitive_disallowed.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                    ),
                    "Disclosing hidden directories aids malicious actors during reconnaissance to pinpoint admin portals, API endpoints, and staging environments.",
                    "Do not rely on robots.txt for security or access control. Enforce strong authentication and IP allowlisting on sensitive administrative routes.",
                    "A05:2021-Security Misconfiguration",
                )
                .with_evidence(sensitive_disallowed.join("\n"))
                .with_refs(&["https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/03-Review_Webserver_Metafiles_for_Information_Leakage"]),
            );
        }

        report.disallowed_paths = disallowed;
        report.sensitive_disallowed_paths = sensitive_disallowed;
    }

    // Process /.well-known/security.txt
    if let Some(body) = sec_body_opt {
        if body.to_lowercase().contains("contact:") {
            report.security_txt_found = true;
            report.security_txt_content = Some(body);
        }
    }

    if !report.security_txt_found {
        findings.push(
            Finding::new(
                "missing-security-txt",
                "Missing security.txt Security Disclosure Policy (RFC 9116)",
                Severity::Info,
                Category::EndpointExposure,
                "No RFC 9116 security.txt file was found at /.well-known/security.txt. A security.txt file helps ethical security researchers responsibly report vulnerabilities directly to your security team.",
                "Vulnerability disclosures may be delayed or misdirected if security researchers cannot find your designated disclosure contacts.",
                "Deploy a security.txt file under the /.well-known/ directory specifying Contact, Expires, and Encryption keys according to RFC 9116.",
                "A05:2021-Security Misconfiguration",
            )
            .with_refs(&[
                "https://securitytxt.org/",
                "https://www.rfc-editor.org/rfc/rfc9116",
            ]),
        );
    }

    // 2. Audit High-Risk Exposed Vulnerability Paths
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

            let mut f = Finding::new(
                probe.id,
                probe.title,
                probe.severity,
                Category::EndpointExposure,
                probe.description,
                probe.impact,
                probe.remediation,
                probe.owasp_category,
            )
            .with_evidence(evidence_lines.join("\n"))
            .with_refs(probe.references);
            if let Some(cve) = probe.cve_id {
                f = f.with_cve(cve);
            }
            Some(f)
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
        PathProbeDef::new(
            "/.env", "exposed-env-file", "Critical Environment Configuration File Exposed (.env)", Severity::Critical,
            "A production environment file (.env) is publicly accessible at the web root without authentication. Environment files contain plaintext infrastructure credentials, database passwords, cloud tokens, and application secrets.",
            "Unauthenticated threat actors can download this file to harvest database passwords, AWS/Stripe API keys, JWT secret salts, and mail server credentials, allowing direct backend database takeover, data exfiltration, or lateral movement into cloud infrastructure.",
            "Immediately deny HTTP access to hidden dot-files in your web server configuration (e.g., in Nginx: 'location ~ /\\.(?!well-known).* { deny all; }' or in Apache: '<FilesMatch \"^\\.\"> Require all denied </FilesMatch>'). Move all sensitive environment files outside the document root and immediately rotate all disclosed credentials.",
            CheckType::EnvFile,
            &["https://owasp.org/www-project-top-ten/2017/A6_2017-Security_Misconfiguration", "https://cwe.mitre.org/data/definitions/200.html"],
        ),
        PathProbeDef::new(
            "/.env.local", "exposed-env-local", "Local Environment Configuration File Exposed (.env.local)", Severity::Critical,
            "A local environment file (.env.local) was deployed to the public web root. Local environment files frequently contain developer API keys, database connection strings, and debug passwords.",
            "Attackers can extract private keys, test database passwords, and third-party API credentials, compromising backend services and sensitive records.",
            "Remove .env.local from the production web root and configure your web server to reject requests to all dot-prefixed configuration files.",
            CheckType::EnvFile,
            &["https://owasp.org/www-project-top-ten/2017/A6_2017-Security_Misconfiguration"],
        ),
        PathProbeDef::new(
            "/.env.production", "exposed-env-production", "Production Environment Secrets File Exposed (.env.production)", Severity::Critical,
            "The production-specific environment secrets file (.env.production) is publicly exposed over HTTP.",
            "Allows immediate exfiltration of live production database credentials and third-party payment/cloud access keys.",
            "Block HTTP access to .env files in the web server configuration and rotate all compromised production secrets immediately.",
            CheckType::EnvFile,
            &["https://cwe.mitre.org/data/definitions/200.html"],
        ),

        // 2. Git Version Control Repositories
        PathProbeDef::new(
            "/.git/HEAD", "exposed-git-head", "Exposed Git Version Control Repository (/.git/HEAD)", Severity::High,
            "The application exposes its Git version control metadata directory over HTTP. Using standard tools such as git-dumper, attackers can traverse the object tree and reconstruct the complete application repository source code, commit history, and branches.",
            "Attackers can review the full application source code to discover zero-day business logic flaws, hardcoded credentials, hidden internal API endpoints, and private developer communications.",
            "Deny access to the /.git/ folder in your web server (e.g., Nginx: 'location ~ /\\.git { deny all; }') and ensure production build artifacts do not include development version control folders.",
            CheckType::GitHead,
            &["https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/05-Review_Webpage_Content_for_Information_Leakage", "https://cwe.mitre.org/data/definitions/538.html"],
        ),
        PathProbeDef::new(
            "/.git/config", "exposed-git-config", "Git Repository Configuration File Disclosed (/.git/config)", Severity::High,
            "The Git configuration file (/.git/config) is publicly readable, disclosing internal repository paths, remote Git URLs, branch names, and potentially embedded access tokens.",
            "Leads to source repository mapping, disclosure of private repository hostnames, and potential credential leakage if remote URLs contain embedded HTTP authentication tokens.",
            "Block all incoming requests targeting /.git/ paths on the reverse proxy or web server.",
            CheckType::GitConfig,
            &["https://cwe.mitre.org/data/definitions/538.html"],
        ),

        // 3. Database Dumps & Backup Archives
        PathProbeDef::new(
            "/backup.sql", "exposed-backup-sql", "Database SQL Dump File Publicly Accessible (/backup.sql)", Severity::Critical,
            "A raw database SQL dump file is publicly downloadable from the web root.",
            "Malicious users can download the entire application database, containing user tables, password hashes, personal identifiable information (PII), session tokens, and business records.",
            "Remove the SQL dump file immediately from the web server. Store all database backups in secure, access-controlled offline or private cloud storage (e.g. S3 with private ACL).",
            CheckType::SqlDump,
            &["https://owasp.org/www-project-top-ten/2017/A3_2017-Sensitive_Data_Exposure", "https://cwe.mitre.org/data/definitions/200.html"],
        ),
        PathProbeDef::new(
            "/dump.sql", "exposed-dump-sql", "Database Dump File Publicly Accessible (/dump.sql)", Severity::Critical,
            "A database export file (/dump.sql) is accessible without authentication.",
            "Permits complete database extraction, leading to customer credential compromise, privacy violations, and regulatory penalties.",
            "Delete public dump files from the web server and audit server directories for lingering database backup files.",
            CheckType::SqlDump,
            &["https://cwe.mitre.org/data/definitions/200.html"],
        ),
        PathProbeDef::new(
            "/database.sqlite", "exposed-sqlite-db", "SQLite Database File Directly Accessible (/database.sqlite)", Severity::Critical,
            "An embedded SQLite database file is located directly inside the public document root.",
            "Attackers can download the SQLite database file and read all application tables, credentials, and data using standard SQLite client tools.",
            "Move the SQLite database file outside the public web root directory (e.g. into /var/data/ or a dedicated non-served storage directory).",
            CheckType::SqliteDb,
            &["https://cwe.mitre.org/data/definitions/538.html"],
        ),
        PathProbeDef::new(
            "/wp-config.php.bak", "exposed-wp-config-bak", "WordPress Configuration Backup File Exposed (/wp-config.php.bak)", Severity::Critical,
            "A backup of the WordPress configuration file is publicly downloadable. Because the .bak extension is not processed by PHP engines, the server serves the raw source code containing database credentials and authentication keys.",
            "Attackers obtain DB_NAME, DB_USER, DB_PASSWORD, and WordPress auth salt keys, facilitating direct database compromise and session forgery.",
            "Delete all .bak, .old, and .save files from the web server. Configure the web server to deny requests to backup extensions.",
            CheckType::ConfigBackup,
            &["https://cwe.mitre.org/data/definitions/538.html"],
        ),
        PathProbeDef::new(
            "/docker-compose.yml", "exposed-docker-compose", "Docker Compose Infrastructure Definition Exposed", Severity::High,
            "A Docker Compose orchestration file is exposed at the web root, detailing container services, volume mounts, internal network ports, and environment passwords.",
            "Provides an attacker with a complete blueprint of backend microservices, database container names, internal subnet IPs, and hardcoded container credentials.",
            "Remove docker-compose.yml from the public web root and restrict web deployments to compiled production assets only.",
            CheckType::DockerCompose,
            &["https://cwe.mitre.org/data/definitions/200.html"],
        ),

        // 4. Debuggers, Profilers & Diagnostic Consoles
        PathProbeDef::new(
            "/phpinfo.php", "exposed-phpinfo", "PHP Diagnostic Info Page Publicly Exposed (phpinfo.php)", Severity::High,
            "The phpinfo() diagnostic function is accessible over HTTP without authentication. It exposes PHP version, loaded extensions, file paths, OS details, and environment variables.",
            "Discloses internal server architecture and path mappings, which attackers leverage to identify vulnerable modules, construct local file inclusion exploits, and bypass security filters.",
            "Remove phpinfo diagnostic files from production environments and disable sensitive information functions in php.ini.",
            CheckType::PhpInfo,
            &["https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/02-Fingerprint_Web_Server", "https://cwe.mitre.org/data/definitions/200.html"],
        ),
        PathProbeDef::new(
            "/_profiler/", "exposed-symfony-profiler", "Symfony Web Profiler Development Console Exposed", Severity::High,
            "The Symfony Web Profiler is active and accessible on a production route. The profiler logs HTTP headers, user session tokens, database queries with parameters, and internal exception traces.",
            "Enables unauthenticated users to inspect sensitive requests made by other users, steal session cookies, and review SQL queries.",
            "Disable the Symfony Web Profiler in production configuration (ensure APP_ENV=prod and APP_DEBUG=0).",
            CheckType::SymfonyProfiler,
            &["https://symfony.com/doc/current/profiler.html", "https://cwe.mitre.org/data/definitions/489.html"],
        ),
        PathProbeDef::new(
            "/_debugbar/assets/stylesheets", "exposed-laravel-debugbar", "Laravel Debugbar Development Utility Exposed", Severity::Medium,
            "Laravel Debugbar assets are publicly accessible. When active, Laravel Debugbar exposes database execution times, query parameters, route definitions, and auth states.",
            "Discloses internal application routing, query performance, and configuration details to potential adversaries.",
            "Ensure 'DEBUGBAR_ENABLED=false' and 'APP_DEBUG=false' in production .env configuration.",
            CheckType::LaravelDebugbar,
            &["https://cwe.mitre.org/data/definitions/489.html"],
        ),
        PathProbeDef::new(
            "/telescope/requests", "exposed-laravel-telescope", "Laravel Telescope Diagnostic Dashboard Exposed", Severity::High,
            "Laravel Telescope dashboard is accessible without proper authorization gates, exposing incoming requests, queued jobs, logs, database queries, and notifications.",
            "Unauthenticated actors can monitor real-time application requests, view user payloads, and read sensitive job parameters.",
            "Implement strict authorization gates in TelescopeServiceProvider or disable Telescope in production environments.",
            CheckType::LaravelTelescope,
            &["https://laravel.com/docs/telescope", "https://cwe.mitre.org/data/definitions/284.html"],
        ).with_owasp("A01:2021-Broken Access Control"),
        PathProbeDef::new(
            "/elmah.axd", "exposed-elmah-console", "ASP.NET ELMAH Error Log Console Exposed", Severity::High,
            "The ASP.NET ELMAH (Error Logging Modules and Handlers) console is accessible without authentication, exposing detailed unhandled server exception logs and stack traces.",
            "Leaking stack traces, internal SQL errors, and request payloads allows attackers to understand code paths and target unhandled failure states.",
            "Restrict access to elmah.axd in web.config using authorization rules and ensure remote access is disabled (security.allowRemoteAccess=0).",
            CheckType::ElmahLog,
            &["https://cwe.mitre.org/data/definitions/200.html"],
        ),

        // 5. API Documentation & OpenAPI Specifications
        PathProbeDef::new(
            "/swagger.json", "exposed-swagger-json", "OpenAPI / Swagger JSON Specification Publicly Accessible (/swagger.json)", Severity::Medium,
            "A complete OpenAPI/Swagger JSON specification is publicly exposed at /swagger.json, describing all application API routes, input parameters, schemas, and authorization requirements.",
            "Enables attackers to map the full API surface, locate undocumented internal endpoints, and automate targeted fuzzing and Broken Object Level Authorization (BOLA/IDOR) exploits.",
            "Disable or protect public access to Swagger/OpenAPI documentation endpoints in production environments unless the API is intentionally public.",
            CheckType::SwaggerJson,
            &["https://owasp.org/www-project-api-security/", "https://cwe.mitre.org/data/definitions/200.html"],
        ),
        PathProbeDef::new(
            "/openapi.json", "exposed-openapi-json", "OpenAPI JSON Specification Exposed (/openapi.json)", Severity::Medium,
            "The OpenAPI 3.0 specification is publicly accessible at /openapi.json.",
            "Aids adversaries in discovering internal APIs, request payloads, and parameter types for targeted exploitation.",
            "Require authentication or restrict IP access for API documentation endpoints in production.",
            CheckType::SwaggerJson,
            &["https://owasp.org/www-project-api-security/"],
        ),
        PathProbeDef::new(
            "/v2/api-docs", "exposed-springfox-api-docs", "SpringFox Swagger API Documentation Exposed (/v2/api-docs)", Severity::Medium,
            "The SpringFox Swagger endpoint (/v2/api-docs) exposes internal Spring REST controller definitions.",
            "Discloses internal endpoints, data transfer objects, and controller methods to unauthenticated callers.",
            "Disable SpringFox Swagger in production profiles or secure the /v2/api-docs endpoint with Spring Security.",
            CheckType::SwaggerJson,
            &["https://owasp.org/www-project-api-security/"],
        ),

        // 6. Administrative Portals Exposed to Public Internet
        PathProbeDef::new(
            "/wp-admin/", "exposed-wp-admin", "WordPress Administrative Portal Exposed (/wp-admin/)", Severity::Medium,
            "The WordPress administrative portal (/wp-admin/) is reachable from the public internet without network restrictions.",
            "Permits unauthenticated attackers to perform automated brute-force password guessing, credential stuffing, and user enumeration attacks against administrative accounts.",
            "Restrict /wp-admin/ and /wp-login.php access to trusted IP ranges or a corporate VPN, and enforce multi-factor authentication (MFA) on all admin users.",
            CheckType::AdminPortal("user_login"),
            &["https://owasp.org/www-community/controls/Blocking_Brute_Force_Attacks", "https://cwe.mitre.org/data/definitions/284.html"],
        ),
        PathProbeDef::new(
            "/phpmyadmin/", "exposed-phpmyadmin", "phpMyAdmin Database Management Interface Exposed", Severity::High,
            "The phpMyAdmin web interface is accessible on a public route without firewall restrictions.",
            "Exposes database login credentials to brute-force attacks and exposes the server to known phpMyAdmin software vulnerabilities that can lead to remote code execution.",
            "Never expose phpMyAdmin directly to the public internet. Restrict access via VPN, SSH tunnel, or strict web server IP allowlists.",
            CheckType::AdminPortal("phpMyAdmin"),
            &["https://www.phpmyadmin.net/security/", "https://cwe.mitre.org/data/definitions/284.html"],
        ),

        // 7. Telemetry & Metrics
        PathProbeDef::new(
            "/metrics", "exposed-prometheus-metrics", "Prometheus / Infrastructure Metrics Telemetry Exposed (/metrics)", Severity::Low,
            "An operational Prometheus metrics endpoint (/metrics) is publicly accessible without authentication.",
            "Leaks server process statistics, active JVM thread pools, HTTP request counts, internal microservice hostnames, and database connection numbers, assisting attackers in reconnaissance.",
            "Restrict access to the /metrics endpoint to internal monitoring scrapers (Prometheus) via firewall rules or reverse proxy authentication.",
            CheckType::MetricsPrometheus,
            &["https://prometheus.io/docs/operating/security/", "https://cwe.mitre.org/data/definitions/200.html"],
        ),
        PathProbeDef::new(
            "/actuator/health", "exposed-actuator-health", "Spring Boot Actuator Health Telemetry Exposed (/actuator/health)", Severity::Low,
            "The Spring Boot Actuator /actuator/health endpoint is accessible without authentication.",
            "Discloses internal system component health, database connectivity status, and disk space details.",
            "Set 'management.endpoint.health.show-details=never' in application.properties or secure actuator endpoints with Spring Security.",
            CheckType::SpringActuatorHealth,
            &["https://docs.spring.io/spring-boot/docs/current/reference/html/actuator.html", "https://cwe.mitre.org/data/definitions/200.html"],
        ),
    ]
}
