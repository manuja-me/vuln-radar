use crate::models::{Category, Finding, Severity};
use reqwest::header::HeaderMap;

pub fn analyze_headers(headers: &HeaderMap, is_https: bool) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. Content-Security-Policy (CSP)
    if let Some(csp_val) = headers.get("content-security-policy").and_then(|h| h.to_str().ok()) {
        if csp_val.contains("'unsafe-inline'") && !csp_val.contains("'nonce-") && !csp_val.contains("'sha256-") {
            findings.push(Finding {
                id: "csp-unsafe-inline".to_string(),
                title: "CSP Contains 'unsafe-inline' Without Nonce/Hash".to_string(),
                severity: Severity::Medium,
                category: Category::SecurityHeaders,
                description: "The Content-Security-Policy includes 'unsafe-inline', allowing inline scripts to execute and weakening XSS protection.".to_string(),
                impact: "Attackers who successfully inject HTML can execute malicious JavaScript in victim browsers.".to_string(),
                remediation: "Use cryptographic nonces (nonce-...) or SHA-256 hashes instead of 'unsafe-inline' in script-src directives.".to_string(),
                evidence: Some(format!("Content-Security-Policy: {}", csp_val)),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec![
                    "https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html".to_string(),
                ],
            });
        }
        if csp_val.contains("'unsafe-eval'") {
            findings.push(Finding {
                id: "csp-unsafe-eval".to_string(),
                title: "CSP Contains 'unsafe-eval'".to_string(),
                severity: Severity::Low,
                category: Category::SecurityHeaders,
                description: "The CSP includes 'unsafe-eval', allowing string-to-code execution functions like eval() or Function().".to_string(),
                impact: "Increases exposure to DOM-based XSS when user input reaches dynamic evaluation sinks.".to_string(),
                remediation: "Refactor dynamic code execution and remove 'unsafe-eval' from your CSP directives.".to_string(),
                evidence: Some(format!("Content-Security-Policy: {}", csp_val)),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec![
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy/script-src".to_string(),
                ],
            });
        }
        if csp_val.contains("default-src *") || csp_val.contains("script-src *") {
            findings.push(Finding {
                id: "csp-wildcard".to_string(),
                title: "CSP Directives Use Wildcard Domain".to_string(),
                severity: Severity::Medium,
                category: Category::SecurityHeaders,
                description: "The CSP specifies wildcard '*' for script-src or default-src, effectively allowing scripts from any external source.".to_string(),
                impact: "Defeats the domain whitelisting protection of CSP against malicious script inclusion.".to_string(),
                remediation: "Restrict sources to 'self' or explicit, trusted domain origins.".to_string(),
                evidence: Some(format!("Content-Security-Policy: {}", csp_val)),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec![
                    "https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html".to_string(),
                ],
            });
        }
    } else {
        findings.push(Finding {
            id: "missing-csp".to_string(),
            title: "Missing Content-Security-Policy (CSP) Header".to_string(),
            severity: Severity::High,
            category: Category::SecurityHeaders,
            description: "No Content-Security-Policy header was detected. CSP prevents Cross-Site Scripting (XSS), data injection, and clickjacking attacks.".to_string(),
            impact: "Leaves the application significantly more vulnerable to Cross-Site Scripting (XSS) and code injection.".to_string(),
            remediation: "Implement a Content-Security-Policy header (e.g. default-src 'self'; script-src 'self'; object-src 'none';).".to_string(),
            evidence: None,
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec![
                "https://owasp.org/www-project-secure-headers/#content-security-policy".to_string(),
                "https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP".to_string(),
            ],
        });
    }

    // 2. Strict-Transport-Security (HSTS)
    if is_https {
        if let Some(hsts_val) = headers.get("strict-transport-security").and_then(|h| h.to_str().ok()) {
            let mut max_age = 0u64;
            for part in hsts_val.split(';') {
                let trimmed = part.trim();
                if trimmed.starts_with("max-age=") {
                    if let Ok(age) = trimmed.trim_start_matches("max-age=").parse::<u64>() {
                        max_age = age;
                    }
                }
            }

            if max_age < 15552000 {
                findings.push(Finding {
                    id: "hsts-short-duration".to_string(),
                    title: "HSTS Max-Age Duration Is Too Short".to_string(),
                    severity: Severity::Low,
                    category: Category::TlsSsl,
                    description: format!("HSTS max-age is set to {} seconds, which is under recommended minimum 180 days (15,552,000s) or 1 year (31,536,000s).", max_age),
                    impact: "Users visiting after a short lapse could be exposed to SSL stripping and man-in-the-middle downgrade attacks.".to_string(),
                    remediation: "Set Strict-Transport-Security to 'max-age=31536000; includeSubDomains; preload'.".to_string(),
                    evidence: Some(format!("Strict-Transport-Security: {}", hsts_val)),
                    owasp_category: "A02:2021-Cryptographic Failures".to_string(),
                    cve_id: None,
                    references: vec![
                        "https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Strict_Transport_Security_Cheat_Sheet.html".to_string(),
                    ],
                });
            }

            if !hsts_val.contains("includeSubDomains") {
                findings.push(Finding {
                    id: "hsts-missing-subdomains".to_string(),
                    title: "HSTS Missing 'includeSubDomains' Directive".to_string(),
                    severity: Severity::Low,
                    category: Category::TlsSsl,
                    description: "HSTS is active on the root domain but does not enforce HTTPS on all subdomains.".to_string(),
                    impact: "Subdomains may still be vulnerable to man-in-the-middle downgrade attacks.".to_string(),
                    remediation: "Add 'includeSubDomains' to the Strict-Transport-Security header value.".to_string(),
                    evidence: Some(format!("Strict-Transport-Security: {}", hsts_val)),
                    owasp_category: "A02:2021-Cryptographic Failures".to_string(),
                    cve_id: None,
                    references: vec![
                        "https://hstspreload.org/".to_string(),
                    ],
                });
            }
        } else {
            findings.push(Finding {
                id: "missing-hsts".to_string(),
                title: "Missing Strict-Transport-Security (HSTS) Header".to_string(),
                severity: Severity::Medium,
                category: Category::TlsSsl,
                description: "The HTTPS response does not include an HSTS header. Browsers won't enforce HTTPS connections on subsequent visits.".to_string(),
                impact: "Vulnerable to SSL stripping and active Man-in-the-Middle (MitM) attacks during initial HTTP requests.".to_string(),
                remediation: "Add 'Strict-Transport-Security: max-age=31536000; includeSubDomains' to your web server configuration.".to_string(),
                evidence: None,
                owasp_category: "A02:2021-Cryptographic Failures".to_string(),
                cve_id: None,
                references: vec![
                    "https://owasp.org/www-project-secure-headers/#http-strict-transport-security".to_string(),
                ],
            });
        }
    }

    // 3. X-Frame-Options (Clickjacking)
    let has_xfo = headers.get("x-frame-options").is_some();
    let has_frame_ancestors = headers
        .get("content-security-policy")
        .and_then(|h| h.to_str().ok())
        .map(|csp| csp.contains("frame-ancestors"))
        .unwrap_or(false);

    if !has_xfo && !has_frame_ancestors {
        findings.push(Finding {
            id: "missing-clickjacking-protection".to_string(),
            title: "Missing Clickjacking Protection (X-Frame-Options / frame-ancestors)".to_string(),
            severity: Severity::Medium,
            category: Category::SecurityHeaders,
            description: "Neither X-Frame-Options nor CSP frame-ancestors header is configured to prevent iframe embedding.".to_string(),
            impact: "Attackers can embed this website inside a transparent iframe on a malicious site to trick users into unintended clicks (Clickjacking).".to_string(),
            remediation: "Add 'X-Frame-Options: DENY' or 'X-Frame-Options: SAMEORIGIN', or set 'frame-ancestors 'self'' in CSP.".to_string(),
            evidence: None,
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec![
                "https://cheatsheetseries.owasp.org/cheatsheets/Clickjacking_Defense_Cheat_Sheet.html".to_string(),
            ],
        });
    }

    // 4. X-Content-Type-Options
    if let Some(xcto) = headers.get("x-content-type-options").and_then(|h| h.to_str().ok()) {
        if xcto.trim().to_lowercase() != "nosniff" {
            findings.push(Finding {
                id: "invalid-x-content-type-options".to_string(),
                title: "Invalid X-Content-Type-Options Header Value".to_string(),
                severity: Severity::Low,
                category: Category::SecurityHeaders,
                description: format!("X-Content-Type-Options is set to '{}' instead of 'nosniff'.", xcto),
                impact: "Browsers may perform MIME-type sniffing, transforming non-executable MIME types into executable ones.".to_string(),
                remediation: "Set 'X-Content-Type-Options: nosniff'.".to_string(),
                evidence: Some(format!("X-Content-Type-Options: {}", xcto)),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec!["https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options".to_string()],
            });
        }
    } else {
        findings.push(Finding {
            id: "missing-x-content-type-options".to_string(),
            title: "Missing X-Content-Type-Options Header".to_string(),
            severity: Severity::Low,
            category: Category::SecurityHeaders,
            description: "Missing 'X-Content-Type-Options: nosniff'. Without this, browsers may attempt to guess the MIME type of a file.".to_string(),
            impact: "Can lead to drive-by downloads or execute user-uploaded files as HTML/JavaScript.".to_string(),
            remediation: "Configure the server to return 'X-Content-Type-Options: nosniff'.".to_string(),
            evidence: None,
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec!["https://owasp.org/www-project-secure-headers/#x-content-type-options".to_string()],
        });
    }

    // 5. Referrer-Policy
    if let Some(rp) = headers.get("referrer-policy").and_then(|h| h.to_str().ok()) {
        if rp.contains("unsafe-url") || rp.contains("no-referrer-when-downgrade") {
            findings.push(Finding {
                id: "insecure-referrer-policy".to_string(),
                title: "Insecure Referrer-Policy Configuration".to_string(),
                severity: Severity::Low,
                category: Category::SecurityHeaders,
                description: format!("The Referrer-Policy header is configured with '{}', which may leak full URL parameters.", rp),
                impact: "Sensitive query parameters, user tokens, or session IDs in the URL might be leaked to external third-party domains in the Referer header.".to_string(),
                remediation: "Use 'Referrer-Policy: strict-origin-when-cross-origin' or 'no-referrer'.".to_string(),
                evidence: Some(format!("Referrer-Policy: {}", rp)),
                owasp_category: "A01:2021-Broken Access Control".to_string(),
                cve_id: None,
                references: vec!["https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy".to_string()],
            });
        }
    } else {
        findings.push(Finding {
            id: "missing-referrer-policy".to_string(),
            title: "Missing Referrer-Policy Header".to_string(),
            severity: Severity::Info,
            category: Category::SecurityHeaders,
            description: "No Referrer-Policy header is set. The browser falls back to default referrer behavior.".to_string(),
            impact: "May leak URL path and query parameters to cross-origin destinations.".to_string(),
            remediation: "Set 'Referrer-Policy: strict-origin-when-cross-origin'.".to_string(),
            evidence: None,
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec!["https://owasp.org/www-project-secure-headers/#referrer-policy".to_string()],
        });
    }

    // 6. Permissions-Policy
    if headers.get("permissions-policy").is_none() && headers.get("feature-policy").is_none() {
        findings.push(Finding {
            id: "missing-permissions-policy".to_string(),
            title: "Missing Permissions-Policy Header".to_string(),
            severity: Severity::Info,
            category: Category::SecurityHeaders,
            description: "No Permissions-Policy header found. This header allows disabling browser hardware features (camera, microphone, geolocation, payment).".to_string(),
            impact: "Embedded third-party iframes may request access to sensitive browser device features.".to_string(),
            remediation: "Define a Permissions-Policy header (e.g. 'camera=(), microphone=(), geolocation=()').".to_string(),
            evidence: None,
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec!["https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Permissions-Policy".to_string()],
        });
    }

    // 7. Information Disclosure (Server & Technology Banners)
    if let Some(srv) = headers.get("server").and_then(|h| h.to_str().ok()) {
        // If server header reveals version like nginx/1.18.0 or Apache/2.4.41
        if srv.chars().any(|c| c.is_ascii_digit()) {
            findings.push(Finding {
                id: "server-version-disclosure".to_string(),
                title: "Server Banner Discloses Version Details".to_string(),
                severity: Severity::Low,
                category: Category::InformationDisclosure,
                description: format!("The 'Server' response header exposes specific server software and version details ('{}').", srv),
                impact: "Helps attackers quickly target known vulnerabilities and exploits associated with that exact version.".to_string(),
                remediation: "Disable or obfuscate detailed server tokens in server configuration (e.g., server_tokens off in Nginx, ServerTokens Prod in Apache).".to_string(),
                evidence: Some(format!("Server: {}", srv)),
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec!["https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/02-Fingerprint_Web_Server".to_string()],
            });
        }
    }

    if let Some(powered_by) = headers.get("x-powered-by").and_then(|h| h.to_str().ok()) {
        findings.push(Finding {
            id: "x-powered-by-disclosure".to_string(),
            title: "X-Powered-By Header Discloses Technology Stack".to_string(),
            severity: Severity::Low,
            category: Category::InformationDisclosure,
            description: format!("The 'X-Powered-By' header exposes backend runtime/framework info ('{}').", powered_by),
            impact: "Aids reconnaissance and targeted framework exploitation.".to_string(),
            remediation: "Remove the X-Powered-By header in your application framework settings (e.g. app.disable('x-powered-by') in Express).".to_string(),
            evidence: Some(format!("X-Powered-By: {}", powered_by)),
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec!["https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet.html".to_string()],
        });
    }

    if let Some(asp_ver) = headers.get("x-aspnet-version").and_then(|h| h.to_str().ok()) {
        findings.push(Finding {
            id: "aspnet-version-disclosure".to_string(),
            title: "ASP.NET Version Disclosed in Headers".to_string(),
            severity: Severity::Low,
            category: Category::InformationDisclosure,
            description: format!("The 'X-AspNet-Version' header exposes the underlying framework version ('{}').", asp_ver),
            impact: "Enables targeted exploitation of unpatched framework components.".to_string(),
            remediation: "Disable enableVersionHeader in web.config <httpRuntime enableVersionHeader=\"false\" />.".to_string(),
            evidence: Some(format!("X-AspNet-Version: {}", asp_ver)),
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            cve_id: None,
            references: vec!["https://owasp.org/www-project-secure-headers/".to_string()],
        });
    }

    // 8. CORS Wildcard & Null Origin Checks
    if let Some(cors_origin) = headers.get("access-control-allow-origin").and_then(|h| h.to_str().ok()) {
        if cors_origin == "*" {
            let allow_creds = headers
                .get("access-control-allow-credentials")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");
            if allow_creds.eq_ignore_ascii_case("true") {
                findings.push(Finding {
                    id: "cors-wildcard-credentials".to_string(),
                    title: "Dangerous CORS Configuration: Wildcard Origin With Credentials".to_string(),
                    severity: Severity::High,
                    category: Category::CorsMisconfiguration,
                    description: "Access-Control-Allow-Origin is set to '*' while Access-Control-Allow-Credentials is true.".to_string(),
                    impact: "Allows arbitrary malicious websites to read authenticated cross-origin response data.".to_string(),
                    remediation: "Specify explicit, trusted origins rather than wildcard '*' when credentials are required.".to_string(),
                    evidence: Some(format!("Access-Control-Allow-Origin: *\nAccess-Control-Allow-Credentials: {}", allow_creds)),
                    owasp_category: "A01:2021-Broken Access Control".to_string(),
                    cve_id: None,
                    references: vec!["https://portswigger.net/web-security/cors".to_string()],
                });
            }
        } else if cors_origin.trim().eq_ignore_ascii_case("null") {
            findings.push(Finding {
                id: "cors-null-origin-allowed".to_string(),
                title: "Insecure CORS Policy: 'null' Origin Permitted".to_string(),
                severity: Severity::High,
                category: Category::CorsMisconfiguration,
                description: "The server trusts the 'null' origin in Access-Control-Allow-Origin. Sandboxed iframes and local file exploits can generate a 'null' Origin to bypass CORS.".to_string(),
                impact: "Attackers can use sandboxed iframes or data: URIs to steal sensitive cross-origin data.".to_string(),
                remediation: "Avoid trusting the 'null' origin. Validate against an explicit whitelist of trusted HTTPS origins.".to_string(),
                evidence: Some("Access-Control-Allow-Origin: null".to_string()),
                owasp_category: "A01:2021-Broken Access Control".to_string(),
                cve_id: None,
                references: vec!["https://portswigger.net/web-security/cors".to_string()],
            });
        }
    }

    // 9. Cross-Origin-Opener-Policy (COOP) & Cross-Origin-Embedder-Policy (COEP)
    if is_https {
        let has_coop = headers.get("cross-origin-opener-policy").is_some();
        if !has_coop {
            findings.push(Finding {
                id: "missing-coop".to_string(),
                title: "Missing Cross-Origin-Opener-Policy (COOP) Header".to_string(),
                severity: Severity::Info,
                category: Category::SecurityHeaders,
                description: "No Cross-Origin-Opener-Policy (COOP) header detected. COOP isolates your top-level browsing context from cross-origin documents.".to_string(),
                impact: "Without COOP, cross-origin popups or window.opener references can interact with your window, facilitating XS-Leaks and Spectre-based attacks.".to_string(),
                remediation: "Set 'Cross-Origin-Opener-Policy: same-origin' or 'same-origin-allow-popups'.".to_string(),
                evidence: None,
                owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                cve_id: None,
                references: vec!["https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Opener-Policy".to_string()],
            });
        }
    }

    findings
}
