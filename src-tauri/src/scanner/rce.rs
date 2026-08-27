use crate::models::{Category, Finding, Severity};
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::collections::HashSet;
use std::time::Duration;
use url::Url;

/// Audits URL query parameters, server/technology headers, and high-risk exposed endpoints for RCE risks.
pub async fn audit_rce_risks(
    client: &Client,
    parsed_url: &Url,
    headers: &HeaderMap,
    technologies: &[String],
) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. URL Query Parameter Attack Surface Heuristics
    audit_url_parameters(parsed_url, &mut findings);

    // 2. Server Banner & Technology Version Known RCE CVE Checks
    audit_version_cves(headers, technologies, &mut findings);

    // 3. High-Risk Management & Debug Endpoint Probes
    let endpoint_findings = probe_dangerous_rce_endpoints(client, parsed_url).await;
    findings.extend(endpoint_findings);

    findings
}

/// Evaluates URL query parameters for dangerous command execution, code evaluation, and template/file inclusion patterns.
pub fn audit_url_parameters(parsed_url: &Url, findings: &mut Vec<Finding>) {
    let query_pairs: Vec<(String, String)> = parsed_url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    if query_pairs.is_empty() {
        return;
    }

    let command_param_keys: HashSet<&'static str> = [
        "cmd",
        "exec",
        "command",
        "run",
        "eval",
        "process",
        "cli",
        "shell",
        "system",
        "code",
        "ping",
        "query_exec",
        "func",
        "execute",
    ]
    .into_iter()
    .collect();

    let template_file_param_keys: HashSet<&'static str> = [
        "tpl",
        "template",
        "include",
        "require",
        "page",
        "file",
        "filepath",
        "doc",
        "view",
        "render",
        "load",
        "layout",
        "component",
    ]
    .into_iter()
    .collect();

    let mut matched_command_params = Vec::new();
    let mut matched_template_params = Vec::new();

    for (k, _) in &query_pairs {
        let lower_k = k.to_lowercase();
        if command_param_keys.contains(lower_k.as_str()) {
            matched_command_params.push(k.clone());
        } else if template_file_param_keys.contains(lower_k.as_str()) {
            matched_template_params.push(k.clone());
        }
    }

    if !matched_command_params.is_empty() {
        findings.push(Finding {
            id: "rce-risk-command-param-surface".to_string(),
            title: "Potential Remote Command Execution (RCE) Parameter Surface".to_string(),
            severity: Severity::High,
            category: Category::RceRisk,
            description: format!(
                "The target URL contains query parameter(s) ('{}') that frequently act as command execution or script evaluation entry points. If user input supplied to these parameters is passed into system shells, process spawners, or dynamic code evaluators (e.g., exec, system, eval) without strict validation, attackers could execute arbitrary OS commands.",
                matched_command_params.join("', '")
            ),
            impact: "Complete server compromise, unauthorized access to host file system, interactive shell access, and lateral movement across internal networks.".to_string(),
            remediation: "Avoid passing user input directly to system command execution functions or eval(). Use parameterized APIs, strict allowlists of permissible actions, and isolate backend execution within unprivileged sandboxes.".to_string(),
            evidence: Some(format!(
                "Identified command-oriented parameter(s): {}",
                matched_command_params.join(", ")
            )),
            owasp_category: "A03:2021-Injection".to_string(),
            cve_id: None,
            references: vec![
                "https://owasp.org/www-community/attacks/Command_Injection".to_string(),
                "https://cwe.mitre.org/data/definitions/78.html".to_string(),
            ],
        });
    }

    if !matched_template_params.is_empty() {
        findings.push(Finding {
            id: "rce-risk-template-inclusion-param-surface".to_string(),
            title: "Potential Server-Side Template / File Inclusion Parameter Detected".to_string(),
            severity: Severity::Medium,
            category: Category::RceRisk,
            description: format!(
                "The target URL exposes query parameter(s) ('{}') typically associated with dynamic file loading, template rendering, or view selection. If handled unsafely, these parameters can expose the application to Server-Side Template Injection (SSTI) or Local/Remote File Inclusion (LFI/RFI), which frequently lead to Remote Code Execution.",
                matched_template_params.join("', '")
            ),
            impact: "Arbitrary file reading, source code leakage, or remote code execution via template sandbox escape.".to_string(),
            remediation: "Never allow client-controlled parameters to directly dictate file system paths or raw template strings. Map input to a hardcoded enum/lookup table on the server.".to_string(),
            evidence: Some(format!(
                "Identified file/template parameter(s): {}",
                matched_template_params.join(", ")
            )),
            owasp_category: "A03:2021-Injection".to_string(),
            cve_id: None,
            references: vec![
                "https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/07-Input_Validation_Testing/18-Testing_for_Server-Side_Template_Injection".to_string(),
                "https://cwe.mitre.org/data/definitions/94.html".to_string(),
            ],
        });
    }
}

/// Checks server and framework headers against known high-severity RCE CVEs.
pub fn audit_version_cves(
    headers: &HeaderMap,
    technologies: &[String],
    findings: &mut Vec<Finding>,
) {
    let server_banner = headers
        .get("server")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let powered_by = headers
        .get("x-powered-by")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let jenkins_header = headers
        .get("x-jenkins")
        .or_else(|| headers.get("x-hudson"))
        .and_then(|v| v.to_str().ok());

    // 1. Apache 2.4.49 / 2.4.50 Path Traversal & RCE (CVE-2021-41773 / CVE-2021-42013)
    if server_banner.contains("apache/2.4.49") || server_banner.contains("apache/2.4.50") {
        findings.push(Finding {
            id: "rce-cve-2021-41773-apache".to_string(),
            title: "Critical Apache HTTP Server Path Traversal & RCE (CVE-2021-41773 / CVE-2021-42013)".to_string(),
            severity: Severity::Critical,
            category: Category::RceRisk,
            description: "The server banner indicates Apache HTTP Server version 2.4.49 or 2.4.50. A flaw in path normalization allows unauthenticated attackers to perform path traversal and, if CGI scripts are enabled (e.g., mod_cgi), execute arbitrary commands on the host.".to_string(),
            impact: "Full unauthenticated remote code execution and arbitrary file read.".to_string(),
            remediation: "Upgrade Apache HTTP Server to version 2.4.51 or higher immediately.".to_string(),
            evidence: Some(format!("Server header: {}", server_banner)),
            owasp_category: "A06:2021-Vulnerable and Outdated Components".to_string(),
            cve_id: Some("CVE-2021-41773".to_string()),
            references: vec![
                "https://nvd.nist.gov/vuln/detail/CVE-2021-41773".to_string(),
                "https://httpd.apache.org/security/vulnerabilities_24.html".to_string(),
            ],
        });
    }

    // 2. PHP 8.1.0-dev Backdoor Signature
    if powered_by.contains("php/8.1.0-dev") || server_banner.contains("php/8.1.0-dev") {
        findings.push(Finding {
            id: "rce-php-8-1-0-dev-backdoor".to_string(),
            title: "Critical PHP 8.1.0-dev Backdoor Detected (Remote Code Execution)".to_string(),
            severity: Severity::Critical,
            category: Category::RceRisk,
            description: "The server discloses PHP 8.1.0-dev in its headers. This specific development branch contained an intentional backdoor committed to the PHP repository that allows unauthenticated remote code execution via custom User-Agent headers (User-Agentt: zerodiumsystem(...)).".to_string(),
            impact: "Instant unauthenticated remote root/server takeover.".to_string(),
            remediation: "Immediately replace this compromised PHP binary with an official, cryptographically verified stable release of PHP (8.2+ or 8.3+).".to_string(),
            evidence: Some(format!("X-Powered-By / Server: {} | {}", powered_by, server_banner)),
            owasp_category: "A06:2021-Vulnerable and Outdated Components".to_string(),
            cve_id: Some("CVE-2021-23337".to_string()),
            references: vec![
                "https://github.com/php/php-src/commit/c730a7388901dae070f4e23d6d972b8324a4bb70".to_string(),
                "https://news-web.php.net/php.internals/113838".to_string(),
            ],
        });
    }

    // 3. Outdated PHP CGI Vulnerable to Argument Injection (CVE-2024-4577)
    if (powered_by.contains("php/5.") || powered_by.contains("php/7.") || powered_by.contains("php/8.0"))
        && !powered_by.contains("php/8.1.0-dev")
    {
        findings.push(Finding {
            id: "rce-php-eol-cgi-exposure".to_string(),
            title: "End-of-Life PHP Runtime Detected (High RCE & CGI Injection Risk)".to_string(),
            severity: Severity::High,
            category: Category::RceRisk,
            description: format!(
                "The server reports an End-of-Life (EOL) version of PHP ('{}'). Legacy PHP installations in CGI/FastCGI environments are susceptible to severe command and argument injection flaws such as CVE-2024-4577 and CVE-2012-1823, enabling arbitrary command execution.",
                powered_by
            ),
            impact: "Exposure to unpatched remote code execution vulnerabilities, memory corruption bugs, and security bypasses.".to_string(),
            remediation: "Upgrade PHP to a currently supported version (PHP 8.2+ or 8.3+) and ensure PHP is not executed via legacy mod_cgi.".to_string(),
            evidence: Some(format!("X-Powered-By: {}", powered_by)),
            owasp_category: "A06:2021-Vulnerable and Outdated Components".to_string(),
            cve_id: Some("CVE-2024-4577".to_string()),
            references: vec![
                "https://nvd.nist.gov/vuln/detail/CVE-2024-4577".to_string(),
                "https://www.php.net/supported-versions.php".to_string(),
            ],
        });
    }

    // 4. Jenkins Controller Version Exposure (CVE-2024-23897)
    if let Some(jenkins_ver) = jenkins_header {
        findings.push(Finding {
            id: "rce-jenkins-version-exposed".to_string(),
            title: "Jenkins Automation Server Exposed (Check for CVE-2024-23897 RCE)".to_string(),
            severity: Severity::High,
            category: Category::RceRisk,
            description: format!(
                "A Jenkins instance header ('X-Jenkins: {}') was detected. Unauthenticated Jenkins controllers prior to version 2.442 and LTS 2.426.3 contain a critical vulnerability in the args4j command-line parser (CVE-2024-23897) that allows reading arbitrary files and achieving remote code execution.",
                jenkins_ver
            ),
            impact: "Reading sensitive controller files, SSH keys, credentials, and full controller RCE.".to_string(),
            remediation: "Upgrade Jenkins to version 2.442 / LTS 2.426.3 or higher, or disable the CLI endpoint if not required.".to_string(),
            evidence: Some(format!("X-Jenkins: {}", jenkins_ver)),
            owasp_category: "A06:2021-Vulnerable and Outdated Components".to_string(),
            cve_id: Some("CVE-2024-23897".to_string()),
            references: vec![
                "https://www.jenkins.io/security/advisory/2024-01-24/".to_string(),
                "https://nvd.nist.gov/vuln/detail/CVE-2024-23897".to_string(),
            ],
        });
    }

    // 5. Webmin MiniServ RCE (CVE-2019-15107)
    if server_banner.contains("miniserv") {
        findings.push(Finding {
            id: "rce-webmin-miniserv-detected".to_string(),
            title: "Webmin MiniServ Management Console Detected".to_string(),
            severity: Severity::High,
            category: Category::RceRisk,
            description: "The server banner indicates Webmin MiniServ. Historical versions of Webmin (e.g. < 1.930) contain unauthenticated remote code execution vulnerabilities in password reset logic (CVE-2019-15107).".to_string(),
            impact: "Unauthenticated root command execution on the host operating system.".to_string(),
            remediation: "Ensure Webmin is running the latest release, enforce multi-factor authentication, and restrict access via firewall / VPN.".to_string(),
            evidence: Some(format!("Server header: {}", server_banner)),
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: Some("CVE-2019-15107".to_string()),
            references: vec![
                "https://nvd.nist.gov/vuln/detail/CVE-2019-15107".to_string(),
                "https://www.webmin.com/security.html".to_string(),
            ],
        });
    }

    // 6. Technology list inspection (e.g. Spring Boot)
    let has_spring = technologies.iter().any(|t| t.to_lowercase().contains("spring"));
    let has_actuator_header = headers.contains_key("x-application-context");
    if has_spring || has_actuator_header {
        findings.push(Finding {
            id: "rce-spring-framework-posture".to_string(),
            title: "Spring Framework Environment Detected (Spring4Shell & Actuator Risk)".to_string(),
            severity: Severity::Low,
            category: Category::RceRisk,
            description: "The application exhibits Spring Framework signatures. Verify that the Spring Framework version is >= 5.3.18 / 5.2.20 (patched against Spring4Shell CVE-2022-22965) and that all Spring Boot Actuator endpoints are strictly authenticated.".to_string(),
            impact: "Potential Remote Code Execution via ClassLoader manipulation or exposed actuator beans.".to_string(),
            remediation: "Keep Spring Framework updated, run on Java 17+, and lock down management.endpoints.web.exposure.include settings in application.properties.".to_string(),
            evidence: if has_actuator_header {
                Some("Disclosed 'X-Application-Context' header".to_string())
            } else {
                Some("Detected Spring runtime signature in response".to_string())
            },
            owasp_category: "A06:2021-Vulnerable and Outdated Components".to_string(),
            cve_id: Some("CVE-2022-22965".to_string()),
            references: vec![
                "https://spring.io/blog/2022/03/31/spring-framework-rce-early-announcement".to_string(),
                "https://tanzu.vmware.com/security/cve-2022-22965".to_string(),
            ],
        });
    }
}

/// Safely probes high-risk management and debug endpoints commonly abused for RCE.
pub async fn probe_dangerous_rce_endpoints(client: &Client, base_url: &Url) -> Vec<Finding> {
    let mut findings = Vec::new();

    let probes = [
        (
            "/actuator/env",
            "Spring Boot Actuator Environment Endpoint Publicly Exposed",
            Severity::Critical,
            "The Spring Boot '/actuator/env' endpoint is publicly accessible without authentication. Attackers can view sensitive credentials and manipulate environment properties to achieve Remote Code Execution (e.g. via Spring Cloud Gateway or H2 database console injection).",
            "Full application compromise, credential leakage, and remote code execution.",
            "Set 'management.endpoints.web.exposure.exclude=env' or restrict access to the actuator endpoints using Spring Security.",
            "CVE-2022-22947 / Misconfiguration",
            "propertySources",
        ),
        (
            "/actuator/gateway/routes",
            "Spring Cloud Gateway Dynamic Routes Exposed (SpEL RCE Risk)",
            Severity::Critical,
            "The '/actuator/gateway/routes' endpoint is exposed without authentication. Attackers can inject malicious SpEL expressions via filter definitions, leading directly to unauthenticated remote code execution (CVE-2022-22947).",
            "Unauthenticated Remote Code Execution on the host server.",
            "Disable or secure the gateway actuator routes endpoint in production configuration.",
            "CVE-2022-22947",
            "predicate",
        ),
        (
            "/script",
            "Jenkins Groovy Script Console Exposed",
            Severity::Critical,
            "The Jenkins Groovy script execution console at '/script' is publicly accessible. Anyone can execute arbitrary Groovy code with controller permissions, providing direct OS command execution.",
            "Immediate root/system execution on the Jenkins host machine.",
            "Enable Jenkins matrix-based security and restrict administrative console access to authenticated administrators.",
            "CWE-94: Code Injection",
            "Groovy script",
        ),
        (
            "/solr/admin/cores?wt=json",
            "Apache Solr Administration API Publicly Exposed",
            Severity::High,
            "The Apache Solr administrative API is accessible without authentication. Publicly accessible Solr instances have historically been targeted by multiple unauthenticated RCE exploits via config manipulation.",
            "Remote command execution and arbitrary index manipulation.",
            "Enable Solr authentication (RuleBasedAuthorizationPlugin) and place Solr behind a firewall or reverse proxy.",
            "CVE-2019-17558 / CVE-2019-0193",
            "responseHeader",
        ),
        (
            "/server-status",
            "Apache Server Status Console Disclosed",
            Severity::Medium,
            "The Apache '/server-status' page is publicly viewable, leaking active HTTP requests, client IP addresses, internal URLs, and application state.",
            "Information disclosure aiding targeted exploitation and session hijacking.",
            "Add 'Require ip <trusted_ip>' or 'Require local' in the Apache server-status configuration block.",
            "CWE-200: Information Exposure",
            "Apache Server Status",
        ),
    ];

    for (path, title, severity, desc, impact, remediation, cve_str, match_str) in probes {
        if let Ok(probe_url) = base_url.join(path) {
            let req = client
                .get(probe_url.as_str())
                .timeout(Duration::from_secs(4))
                .send()
                .await;

            if let Ok(resp) = req {
                let status = resp.status();
                if status.is_success() {
                    let status_code = status.as_u16();
                    let content_type = resp
                        .headers()
                        .get("content-type")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_lowercase();

                    if let Ok(body) = resp.text().await {
                        let is_confirmed = if path.contains("actuator") {
                            body.contains(match_str) && (content_type.contains("json") || body.starts_with('{'))
                        } else if path == "/script" {
                            body.contains(match_str) || body.contains("Jenkins.instance")
                        } else if path.contains("solr") {
                            body.contains(match_str) && body.contains("status")
                        } else {
                            body.contains(match_str)
                        };

                        if is_confirmed {
                            let cve_opt = if cve_str.starts_with("CVE-") {
                                Some(cve_str.split(' ').next().unwrap_or(cve_str).to_string())
                            } else {
                                None
                            };

                            findings.push(Finding {
                                id: format!("rce-exposed-endpoint-{}", path.replace('/', "-").trim_matches('-')),
                                title: title.to_string(),
                                severity,
                                category: Category::RceRisk,
                                description: desc.to_string(),
                                impact: impact.to_string(),
                                remediation: remediation.to_string(),
                                evidence: Some(format!(
                                    "Accessible endpoint: {} (Status: {}, Content-Type: {})",
                                    probe_url,
                                    status_code,
                                    content_type
                                )),
                                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                                cve_id: cve_opt,
                                references: vec![
                                    "https://owasp.org/www-project-top-ten/2017/A5_2017-Broken_Access_Control".to_string(),
                                    "https://cwe.mitre.org/data/definitions/284.html".to_string(),
                                ],
                            });
                        }
                    }
                }
            }
        }
    }

    findings
}
