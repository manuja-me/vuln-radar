use crate::models::{Category, EndpointReport, Finding, Severity};
use reqwest::Client;
use std::time::Duration;
use url::Url;

pub async fn audit_endpoints(base_url: &Url) -> (EndpointReport, Vec<Finding>) {
    let mut report = EndpointReport::default();
    let mut findings = Vec::new();

    let client = match Client::builder()
        .timeout(Duration::from_secs(6))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) VulnRadar/1.0")
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return (report, findings),
    };

    // 1. Audit /robots.txt
    if let Ok(robots_url) = base_url.join("/robots.txt") {
        if let Ok(resp) = client.get(robots_url.as_str()).send().await {
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
                            remediation: "Do not rely on robots.txt for security or access control. Enforce strong authentication and IP whitelisting on sensitive administrative routes.".to_string(),
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
        if let Ok(resp) = client.get(sec_url.as_str()).send().await {
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
            references: vec!["https://securitytxt.org/".to_string(), "https://www.rfc-editor.org/rfc/rfc9116".to_string()],
        });
    }

    (report, findings)
}
