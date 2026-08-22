use crate::models::{Category, Finding, Severity};
use reqwest::header::HeaderMap;

pub fn analyze_cookies(headers: &HeaderMap, is_https: bool) -> Vec<Finding> {
    let mut findings = Vec::new();

    let cookie_headers = headers.get_all("set-cookie");

    for cookie in cookie_headers {
        let cookie_str = match cookie.to_str() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let cookie_name = cookie_str.split('=').next().unwrap_or("unknown").trim();
        let cookie_lower = cookie_str.to_lowercase();

        // 1. Missing HttpOnly
        if !cookie_lower.contains("httponly") {
            findings.push(Finding {
                id: format!("cookie-missing-httponly-{}", cookie_name),
                title: format!("Cookie '{}' Missing 'HttpOnly' Flag", cookie_name),
                severity: Severity::Medium,
                category: Category::CookieSecurity,
                description: format!("The cookie '{}' is set without the HttpOnly attribute.", cookie_name),
                impact: "If the application suffers an XSS vulnerability, attackers can access and exfiltrate this cookie via document.cookie.".to_string(),
                remediation: "Append '; HttpOnly' to the Set-Cookie header directive.".to_string(),
                evidence: Some(format!("Set-Cookie: {}", cookie_str)),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec![
                    "https://owasp.org/www-community/HttpOnly".to_string(),
                ],
            });
        }

        // 2. Missing Secure Flag on HTTPS
        if is_https && !cookie_lower.contains("secure") {
            findings.push(Finding {
                id: format!("cookie-missing-secure-{}", cookie_name),
                title: format!("Cookie '{}' Missing 'Secure' Flag on HTTPS", cookie_name),
                severity: Severity::Medium,
                category: Category::CookieSecurity,
                description: format!("The cookie '{}' does not have the 'Secure' attribute while served over HTTPS.", cookie_name),
                impact: "The browser may transmit this cookie over unencrypted HTTP connections if requested, exposing it to eavesdropping.".to_string(),
                remediation: "Append '; Secure' to the Set-Cookie header directive.".to_string(),
                evidence: Some(format!("Set-Cookie: {}", cookie_str)),
                owasp_category: "A02:2021-Cryptographic Failures".to_string(),
                cve_id: None,
                references: vec![
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#secure".to_string(),
                ],
            });
        }

        // 3. SameSite Attribute
        if !cookie_lower.contains("samesite") {
            findings.push(Finding {
                id: format!("cookie-missing-samesite-{}", cookie_name),
                title: format!("Cookie '{}' Missing 'SameSite' Attribute", cookie_name),
                severity: Severity::Low,
                category: Category::CookieSecurity,
                description: format!("The cookie '{}' does not explicitly specify a SameSite policy (Lax, Strict, or None).", cookie_name),
                impact: "Can increase vulnerability to Cross-Site Request Forgery (CSRF) and cross-site tracking.".to_string(),
                remediation: "Set 'SameSite=Lax' or 'SameSite=Strict' for the cookie.".to_string(),
                evidence: Some(format!("Set-Cookie: {}", cookie_str)),
                owasp_category: "A01:2021-Broken Access Control".to_string(),
                cve_id: None,
                references: vec![
                    "https://web.dev/articles/samesite-cookies-explained".to_string(),
                ],
            });
        } else if cookie_lower.contains("samesite=none") && !cookie_lower.contains("secure") {
            findings.push(Finding {
                id: format!("cookie-samesite-none-insecure-{}", cookie_name),
                title: format!("Cookie '{}' Has SameSite=None Without Secure Flag", cookie_name),
                severity: Severity::High,
                category: Category::CookieSecurity,
                description: format!("The cookie '{}' specifies SameSite=None without the Secure attribute.", cookie_name),
                impact: "Modern browsers will reject this cookie, or unencrypted transmission will expose cross-site cookies.".to_string(),
                remediation: "Always pair 'SameSite=None' with the 'Secure' attribute.".to_string(),
                evidence: Some(format!("Set-Cookie: {}", cookie_str)),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec![
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#samesitenone".to_string(),
                ],
            });
        }
    }

    findings
}
