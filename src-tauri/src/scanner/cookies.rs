use crate::models::{Category, Finding, Severity};
use reqwest::header::HeaderMap;

fn cookie_finding(
    id_prefix: &str,
    name: &str,
    title_suffix: &str,
    severity: Severity,
    desc: String,
    impact: &str,
    remediation: &str,
    owasp: &str,
    cookie_str: &str,
    ref_url: &str,
) -> Finding {
    Finding::new(
        format!("{}-{}", id_prefix, name),
        format!("Cookie '{}' {}", name, title_suffix),
        severity,
        Category::CookieSecurity,
        desc,
        impact,
        remediation,
        owasp,
    )
    .with_evidence(format!("Set-Cookie: {}", cookie_str))
    .with_refs(&[ref_url])
}

pub fn analyze_cookies(headers: &HeaderMap, is_https: bool) -> Vec<Finding> {
    let mut findings = Vec::new();

    for cookie in headers.get_all("set-cookie") {
        let cookie_str = match cookie.to_str() {
            Ok(s) => s,
            Err(_) => continue,
        };

        let cookie_name = cookie_str.split('=').next().unwrap_or("unknown").trim();
        let cookie_lower = cookie_str.to_lowercase();

        // 1. Missing HttpOnly
        if !cookie_lower.contains("httponly") {
            findings.push(cookie_finding(
                "cookie-missing-httponly",
                cookie_name,
                "Missing 'HttpOnly' Flag",
                Severity::Medium,
                format!("The cookie '{}' is set without the HttpOnly attribute.", cookie_name),
                "If the application suffers an XSS vulnerability, attackers can access and exfiltrate this cookie via document.cookie.",
                "Append '; HttpOnly' to the Set-Cookie header directive.",
                "A05:2021-Security Misconfiguration",
                cookie_str,
                "https://owasp.org/www-community/HttpOnly",
            ));
        }

        // 2. Missing Secure Flag on HTTPS
        if is_https && !cookie_lower.contains("secure") {
            findings.push(cookie_finding(
                "cookie-missing-secure",
                cookie_name,
                "Missing 'Secure' Flag on HTTPS",
                Severity::Medium,
                format!("The cookie '{}' does not have the 'Secure' attribute while served over HTTPS.", cookie_name),
                "The browser may transmit this cookie over unencrypted HTTP connections if requested, exposing it to eavesdropping.",
                "Append '; Secure' to the Set-Cookie header directive.",
                "A02:2021-Cryptographic Failures",
                cookie_str,
                "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#secure",
            ));
        }

        // 3. SameSite Attribute
        if !cookie_lower.contains("samesite") {
            findings.push(cookie_finding(
                "cookie-missing-samesite",
                cookie_name,
                "Missing 'SameSite' Attribute",
                Severity::Low,
                format!("The cookie '{}' does not explicitly specify a SameSite policy (Lax, Strict, or None).", cookie_name),
                "Can increase vulnerability to Cross-Site Request Forgery (CSRF) and cross-site tracking.",
                "Set 'SameSite=Lax' or 'SameSite=Strict' for the cookie.",
                "A01:2021-Broken Access Control",
                cookie_str,
                "https://web.dev/articles/samesite-cookies-explained",
            ));
        } else if cookie_lower.contains("samesite=none") && !cookie_lower.contains("secure") {
            findings.push(cookie_finding(
                "cookie-samesite-none-insecure",
                cookie_name,
                "Has SameSite=None Without Secure Flag",
                Severity::High,
                format!("The cookie '{}' specifies SameSite=None without the Secure attribute.", cookie_name),
                "Modern browsers will reject this cookie, or unencrypted transmission will expose cross-site cookies.",
                "Always pair 'SameSite=None' with the 'Secure' attribute.",
                "A05:2021-Security Misconfiguration",
                cookie_str,
                "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#samesitenone",
            ));
        }
    }

    findings
}
