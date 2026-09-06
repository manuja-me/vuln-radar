use crate::models::{Category, Finding, Severity};
use reqwest::header::HeaderMap;

fn sec_finding(
    id: &str,
    title: &str,
    severity: Severity,
    desc: impl Into<String>,
    impact: &str,
    remediation: &str,
    ref_url: &str,
) -> Finding {
    Finding::new(
        id,
        title,
        severity,
        Category::SecurityHeaders,
        desc,
        impact,
        remediation,
        "A05:2021-Security Misconfiguration",
    )
    .with_refs(&[ref_url])
}

fn tls_finding(
    id: &str,
    title: &str,
    severity: Severity,
    desc: impl Into<String>,
    impact: &str,
    remediation: &str,
    ref_url: &str,
) -> Finding {
    Finding::new(
        id,
        title,
        severity,
        Category::TlsSsl,
        desc,
        impact,
        remediation,
        "A02:2021-Cryptographic Failures",
    )
    .with_refs(&[ref_url])
}

fn info_finding(
    id: &str,
    title: &str,
    desc: impl Into<String>,
    impact: &str,
    remediation: &str,
    evidence: String,
    ref_url: &str,
) -> Finding {
    Finding::new(
        id,
        title,
        Severity::Low,
        Category::InformationDisclosure,
        desc,
        impact,
        remediation,
        "A05:2021-Security Misconfiguration",
    )
    .with_evidence(evidence)
    .with_refs(&[ref_url])
}

pub fn analyze_headers(headers: &HeaderMap, is_https: bool) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. Content-Security-Policy (CSP)
    if let Some(csp_val) = headers.get("content-security-policy").and_then(|h| h.to_str().ok()) {
        if csp_val.contains("'unsafe-inline'") && !csp_val.contains("'nonce-") && !csp_val.contains("'sha256-") {
            findings.push(
                sec_finding(
                    "csp-unsafe-inline",
                    "CSP Contains 'unsafe-inline' Without Nonce/Hash",
                    Severity::Medium,
                    "The Content-Security-Policy includes 'unsafe-inline', allowing inline scripts to execute and weakening XSS protection.",
                    "Attackers who successfully inject HTML can execute malicious JavaScript in victim browsers.",
                    "Use cryptographic nonces (nonce-...) or SHA-256 hashes instead of 'unsafe-inline' in script-src directives.",
                    "https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html",
                )
                .with_evidence(format!("Content-Security-Policy: {}", csp_val)),
            );
        }
        if csp_val.contains("'unsafe-eval'") {
            findings.push(
                sec_finding(
                    "csp-unsafe-eval",
                    "CSP Contains 'unsafe-eval'",
                    Severity::Low,
                    "The CSP includes 'unsafe-eval', allowing string-to-code execution functions like eval() or Function().",
                    "Increases exposure to DOM-based XSS when user input reaches dynamic evaluation sinks.",
                    "Refactor dynamic code execution and remove 'unsafe-eval' from your CSP directives.",
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy/script-src",
                )
                .with_evidence(format!("Content-Security-Policy: {}", csp_val)),
            );
        }
        if csp_val.contains("default-src *") || csp_val.contains("script-src *") {
            findings.push(
                sec_finding(
                    "csp-wildcard",
                    "CSP Directives Use Wildcard Domain",
                    Severity::Medium,
                    "The CSP specifies wildcard '*' for script-src or default-src, effectively allowing scripts from any external source.",
                    "Defeats the domain whitelisting protection of CSP against malicious script inclusion.",
                    "Restrict sources to 'self' or explicit, trusted domain origins.",
                    "https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html",
                )
                .with_evidence(format!("Content-Security-Policy: {}", csp_val)),
            );
        }
    } else {
        findings.push(sec_finding(
            "missing-csp",
            "Missing Content-Security-Policy (CSP) Header",
            Severity::High,
            "No Content-Security-Policy header was detected. CSP prevents Cross-Site Scripting (XSS), data injection, and clickjacking attacks.",
            "Leaves the application significantly more vulnerable to Cross-Site Scripting (XSS) and code injection.",
            "Implement a Content-Security-Policy header (e.g. default-src 'self'; script-src 'self'; object-src 'none';).",
            "https://owasp.org/www-project-secure-headers/#content-security-policy",
        ));
    }

    // 2. Strict-Transport-Security (HSTS)
    if is_https {
        if let Some(hsts_val) = headers.get("strict-transport-security").and_then(|h| h.to_str().ok()) {
            let mut max_age = 0u64;
            for part in hsts_val.split(';') {
                let trimmed = part.trim();
                if let Some(age_str) = trimmed.strip_prefix("max-age=") {
                    if let Ok(age) = age_str.parse::<u64>() {
                        max_age = age;
                    }
                }
            }

            if max_age < 15552000 {
                findings.push(
                    tls_finding(
                        "hsts-short-duration",
                        "HSTS Max-Age Duration Is Too Short",
                        Severity::Low,
                        format!("HSTS max-age is set to {} seconds, which is under recommended minimum 180 days (15,552,000s) or 1 year (31,536,000s).", max_age),
                        "Users visiting after a short lapse could be exposed to SSL stripping and man-in-the-middle downgrade attacks.",
                        "Set Strict-Transport-Security to 'max-age=31536000; includeSubDomains; preload'.",
                        "https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Strict_Transport_Security_Cheat_Sheet.html",
                    )
                    .with_evidence(format!("Strict-Transport-Security: {}", hsts_val)),
                );
            }

            if !hsts_val.contains("includeSubDomains") {
                findings.push(
                    tls_finding(
                        "hsts-missing-subdomains",
                        "HSTS Missing 'includeSubDomains' Directive",
                        Severity::Low,
                        "HSTS is active on the root domain but does not enforce HTTPS on all subdomains.",
                        "Subdomains may still be vulnerable to man-in-the-middle downgrade attacks.",
                        "Add 'includeSubDomains' to the Strict-Transport-Security header value.",
                        "https://hstspreload.org/",
                    )
                    .with_evidence(format!("Strict-Transport-Security: {}", hsts_val)),
                );
            }
        } else {
            findings.push(tls_finding(
                "missing-hsts",
                "Missing Strict-Transport-Security (HSTS) Header",
                Severity::Medium,
                "The HTTPS response does not include an HSTS header. Browsers won't enforce HTTPS connections on subsequent visits.",
                "Vulnerable to SSL stripping and active Man-in-the-Middle (MitM) attacks during initial HTTP requests.",
                "Add 'Strict-Transport-Security: max-age=31536000; includeSubDomains' to your web server configuration.",
                "https://owasp.org/www-project-secure-headers/#http-strict-transport-security",
            ));
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
        findings.push(sec_finding(
            "missing-clickjacking-protection",
            "Missing Clickjacking Protection (X-Frame-Options / frame-ancestors)",
            Severity::Medium,
            "Neither X-Frame-Options nor CSP frame-ancestors header is configured to prevent iframe embedding.",
            "Attackers can embed this website inside a transparent iframe on a malicious site to trick users into unintended clicks (Clickjacking).",
            "Add 'X-Frame-Options: DENY' or 'X-Frame-Options: SAMEORIGIN', or set 'frame-ancestors 'self'' in CSP.",
            "https://cheatsheetseries.owasp.org/cheatsheets/Clickjacking_Defense_Cheat_Sheet.html",
        ));
    }

    // 4. X-Content-Type-Options
    if let Some(xcto) = headers.get("x-content-type-options").and_then(|h| h.to_str().ok()) {
        if xcto.trim().to_lowercase() != "nosniff" {
            findings.push(
                sec_finding(
                    "invalid-x-content-type-options",
                    "Invalid X-Content-Type-Options Header Value",
                    Severity::Low,
                    format!("X-Content-Type-Options is set to '{}' instead of 'nosniff'.", xcto),
                    "Browsers may perform MIME-type sniffing, transforming non-executable MIME types into executable ones.",
                    "Set 'X-Content-Type-Options: nosniff'.",
                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Content-Type-Options",
                )
                .with_evidence(format!("X-Content-Type-Options: {}", xcto)),
            );
        }
    } else {
        findings.push(sec_finding(
            "missing-x-content-type-options",
            "Missing X-Content-Type-Options Header",
            Severity::Low,
            "Missing 'X-Content-Type-Options: nosniff'. Without this, browsers may attempt to guess the MIME type of a file.",
            "Can lead to drive-by downloads or execute user-uploaded files as HTML/JavaScript.",
            "Configure the server to return 'X-Content-Type-Options: nosniff'.",
            "https://owasp.org/www-project-secure-headers/#x-content-type-options",
        ));
    }

    // 5. Referrer-Policy
    if let Some(rp) = headers.get("referrer-policy").and_then(|h| h.to_str().ok()) {
        if rp.contains("unsafe-url") || rp.contains("no-referrer-when-downgrade") {
            findings.push(
                Finding::new(
                    "insecure-referrer-policy",
                    "Insecure Referrer-Policy Configuration",
                    Severity::Low,
                    Category::SecurityHeaders,
                    format!("The Referrer-Policy header is configured with '{}', which may leak full URL parameters.", rp),
                    "Sensitive query parameters, user tokens, or session IDs in the URL might be leaked to external third-party domains in the Referer header.",
                    "Use 'Referrer-Policy: strict-origin-when-cross-origin' or 'no-referrer'.",
                    "A01:2021-Broken Access Control",
                )
                .with_evidence(format!("Referrer-Policy: {}", rp))
                .with_refs(&["https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy"]),
            );
        }
    } else {
        findings.push(sec_finding(
            "missing-referrer-policy",
            "Missing Referrer-Policy Header",
            Severity::Info,
            "No Referrer-Policy header is set. The browser falls back to default referrer behavior.",
            "May leak URL path and query parameters to cross-origin destinations.",
            "Set 'Referrer-Policy: strict-origin-when-cross-origin'.",
            "https://owasp.org/www-project-secure-headers/#referrer-policy",
        ));
    }

    // 6. Permissions-Policy
    if headers.get("permissions-policy").is_none() && headers.get("feature-policy").is_none() {
        findings.push(sec_finding(
            "missing-permissions-policy",
            "Missing Permissions-Policy Header",
            Severity::Info,
            "No Permissions-Policy header found. This header allows disabling browser hardware features (camera, microphone, geolocation, payment).",
            "Embedded third-party iframes may request access to sensitive browser device features.",
            "Define a Permissions-Policy header (e.g. 'camera=(), microphone=(), geolocation=()').",
            "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Permissions-Policy",
        ));
    }

    // 7. Information Disclosure (Server & Technology Banners)
    if let Some(srv) = headers.get("server").and_then(|h| h.to_str().ok()) {
        if srv.chars().any(|c| c.is_ascii_digit()) {
            findings.push(info_finding(
                "server-version-disclosure",
                "Server Banner Discloses Version Details",
                format!("The 'Server' response header exposes specific server software and version details ('{}').", srv),
                "Helps attackers quickly target known vulnerabilities and exploits associated with that exact version.",
                "Disable or obfuscate detailed server tokens in server configuration (e.g., server_tokens off in Nginx, ServerTokens Prod in Apache).",
                format!("Server: {}", srv),
                "https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/02-Fingerprint_Web_Server",
            ));
        }
    }

    if let Some(powered_by) = headers.get("x-powered-by").and_then(|h| h.to_str().ok()) {
        findings.push(info_finding(
            "x-powered-by-disclosure",
            "X-Powered-By Header Discloses Technology Stack",
            format!("The 'X-Powered-By' header exposes backend runtime/framework info ('{}').", powered_by),
            "Aids reconnaissance and targeted framework exploitation.",
            "Remove the X-Powered-By header in your application framework settings (e.g. app.disable('x-powered-by') in Express).",
            format!("X-Powered-By: {}", powered_by),
            "https://cheatsheetseries.owasp.org/cheatsheets/HTTP_Headers_Cheat_Sheet.html",
        ));
    }

    if let Some(asp_ver) = headers.get("x-aspnet-version").and_then(|h| h.to_str().ok()) {
        findings.push(info_finding(
            "aspnet-version-disclosure",
            "ASP.NET Version Disclosed in Headers",
            format!("The 'X-AspNet-Version' header exposes the underlying framework version ('{}').", asp_ver),
            "Enables targeted exploitation of unpatched framework components.",
            "Disable enableVersionHeader in web.config <httpRuntime enableVersionHeader=\"false\" />.",
            format!("X-AspNet-Version: {}", asp_ver),
            "https://owasp.org/www-project-secure-headers/",
        ));
    }

    // 8. CORS Wildcard & Null Origin Checks
    if let Some(cors_origin) = headers.get("access-control-allow-origin").and_then(|h| h.to_str().ok()) {
        if cors_origin == "*" {
            let allow_creds = headers
                .get("access-control-allow-credentials")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("");
            if allow_creds.eq_ignore_ascii_case("true") {
                findings.push(
                    Finding::new(
                        "cors-wildcard-credentials",
                        "Dangerous CORS Configuration: Wildcard Origin With Credentials",
                        Severity::High,
                        Category::CorsMisconfiguration,
                        "Access-Control-Allow-Origin is set to '*' while Access-Control-Allow-Credentials is true.",
                        "Allows arbitrary malicious websites to read authenticated cross-origin response data.",
                        "Specify explicit, trusted origins rather than wildcard '*' when credentials are required.",
                        "A01:2021-Broken Access Control",
                    )
                    .with_evidence(format!("Access-Control-Allow-Origin: *\nAccess-Control-Allow-Credentials: {}", allow_creds))
                    .with_refs(&["https://portswigger.net/web-security/cors"]),
                );
            }
        } else if cors_origin.trim().eq_ignore_ascii_case("null") {
            findings.push(
                Finding::new(
                    "cors-null-origin-allowed",
                    "Insecure CORS Policy: 'null' Origin Permitted",
                    Severity::High,
                    Category::CorsMisconfiguration,
                    "The server trusts the 'null' origin in Access-Control-Allow-Origin. Sandboxed iframes and local file exploits can generate a 'null' Origin to bypass CORS.",
                    "Attackers can use sandboxed iframes or data: URIs to steal sensitive cross-origin data.",
                    "Avoid trusting the 'null' origin. Validate against an explicit whitelist of trusted HTTPS origins.",
                    "A01:2021-Broken Access Control",
                )
                .with_evidence("Access-Control-Allow-Origin: null")
                .with_refs(&["https://portswigger.net/web-security/cors"]),
            );
        }
    }

    // 9. Cross-Origin-Opener-Policy (COOP)
    if is_https && headers.get("cross-origin-opener-policy").is_none() {
        findings.push(sec_finding(
            "missing-coop",
            "Missing Cross-Origin-Opener-Policy (COOP) Header",
            Severity::Info,
            "No Cross-Origin-Opener-Policy (COOP) header detected. COOP isolates your top-level browsing context from cross-origin documents.",
            "Without COOP, cross-origin popups or window.opener references can interact with your window, facilitating XS-Leaks and Spectre-based attacks.",
            "Set 'Cross-Origin-Opener-Policy: same-origin' or 'same-origin-allow-popups'.",
            "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cross-Origin-Opener-Policy",
        ));
    }

    findings
}
