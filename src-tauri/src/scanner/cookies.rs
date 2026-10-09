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

        let mut parts = cookie_str.split(';');
        let cookie_name = parts.next().and_then(|p| p.split('=').next()).unwrap_or("unknown").trim();
        let attrs: Vec<String> = parts.map(|p| p.trim().to_lowercase()).collect();

        let has_httponly = attrs.iter().any(|a| a == "httponly");
        let has_secure = attrs.iter().any(|a| a == "secure");
        let samesite_attr = attrs.iter().find(|a| a.starts_with("samesite"));

        // 1. Missing HttpOnly
        if !has_httponly {
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
        if is_https && !has_secure {
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
        match samesite_attr {
            None => {
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
            }
            Some(attr) => {
                let clean_samesite = attr.replace(' ', "");
                if clean_samesite == "samesite=none" && !has_secure {
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
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    #[test]
    fn test_cookie_attribute_parsing_exact() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "set-cookie",
            HeaderValue::from_static("notsecure_token=abc; Path=/; SameSite=Lax"),
        );
        let findings = analyze_cookies(&headers, true);
        let ids: Vec<String> = findings.into_iter().map(|f| f.id).collect();
        assert!(ids.contains(&"cookie-missing-secure-notsecure_token".to_string()));
        assert!(ids.contains(&"cookie-missing-httponly-notsecure_token".to_string()));
        assert!(!ids.contains(&"cookie-missing-samesite-notsecure_token".to_string()));
    }
}
