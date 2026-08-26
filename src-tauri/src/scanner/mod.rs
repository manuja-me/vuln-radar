pub mod cookies;
pub mod dependencies;
pub mod dns;
pub mod endpoints;
pub mod headers;
pub mod leaks;
pub mod ports;
pub mod subdomains;

use crate::models::{Finding, ScanOptions, ScanReport, Severity};
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use std::str::FromStr;
use std::time::{Duration, Instant};
use url::Url;

fn is_local_or_private(host: &str) -> bool {
    let h = host.to_lowercase();
    let hostname = h.split(':').next().unwrap_or(&h);
    hostname == "localhost"
        || hostname == "127.0.0.1"
        || hostname == "0.0.0.0"
        || hostname == "::1"
        || hostname.starts_with("192.168.")
        || hostname.starts_with("10.")
        || hostname.starts_with("172.16.")
        || hostname.starts_with("172.17.")
        || hostname.starts_with("172.18.")
        || hostname.starts_with("172.19.")
        || hostname.starts_with("172.20.")
        || hostname.starts_with("172.21.")
        || hostname.starts_with("172.22.")
        || hostname.starts_with("172.23.")
        || hostname.starts_with("172.24.")
        || hostname.starts_with("172.25.")
        || hostname.starts_with("172.26.")
        || hostname.starts_with("172.27.")
        || hostname.starts_with("172.28.")
        || hostname.starts_with("172.29.")
        || hostname.starts_with("172.30.")
        || hostname.starts_with("172.31.")
        || hostname.ends_with(".local")
        || hostname.ends_with(".internal")
        || hostname.ends_with(".lan")
}

pub async fn run_scan(target_url: &str, options: Option<ScanOptions>) -> Result<ScanReport, String> {
    let opts = options.unwrap_or_default();
    let trimmed = target_url.trim();

    if trimmed.is_empty() {
        return Err("Target URL cannot be empty.".to_string());
    }

    let had_explicit_scheme = trimmed.starts_with("http://") || trimmed.starts_with("https://");

    // 1. Normalize and parse candidate URL
    let mut candidate_url = if !had_explicit_scheme {
        let lower = trimmed.to_lowercase();
        // If it starts with localhost, private IP, or contains port (e.g. :3000, :8000), default to http://
        if is_local_or_private(&lower) || lower.contains(':') {
            format!("http://{}", trimmed)
        } else {
            format!("https://{}", trimmed)
        }
    } else {
        trimmed.to_string()
    };

    let mut parsed_url = Url::parse(&candidate_url).map_err(|e| format!("Invalid URL format: {}", e))?;
    
    // Strict URL scheme restriction - only HTTP and HTTPS are permitted
    let mut scheme = parsed_url.scheme().to_string();
    if scheme != "http" && scheme != "https" {
        return Err(format!("Unsupported URL scheme '{}'. Only HTTP and HTTPS are permitted.", scheme));
    }

    let mut is_https = scheme == "https";
    let domain = parsed_url.host_str().unwrap_or_default().to_string();

    // 2. Build HTTP client with custom options & support self-signed backend certs
    let timeout_secs = opts.timeout_seconds.unwrap_or(15).clamp(2, 120);
    let user_agent_str = opts.user_agent.unwrap_or_else(|| {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 VulnRadar/1.0".to_string()
    });

    let mut custom_header_map = HeaderMap::new();
    if let Some(headers) = &opts.custom_headers {
        for (k, v) in headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
                custom_header_map.insert(name, val);
            }
        }
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(user_agent_str)
        .default_headers(custom_header_map)
        .redirect(reqwest::redirect::Policy::limited(5))
        .danger_accept_invalid_certs(true) // Allows auditing backend microservices and self-signed dev/staging servers
        .build()
        .map_err(|e| format!("Failed to initialize HTTP client: {}", e))?;

    let mut start_time = Instant::now();
    let response_result = client.get(parsed_url.as_str()).send().await;

    let response = match response_result {
        Ok(resp) => resp,
        Err(err) => {
            // If the connection failed on https:// and user did not explicitly force https://,
            // retry on http:// (essential for backend servers, local APIs, and microservices)
            if !had_explicit_scheme && candidate_url.starts_with("https://") {
                candidate_url = format!("http://{}", trimmed);
                if let Ok(fallback_url) = Url::parse(&candidate_url) {
                    parsed_url = fallback_url;
                    scheme = parsed_url.scheme().to_string();
                    is_https = scheme == "https";
                    start_time = Instant::now();

                    client.get(parsed_url.as_str()).send().await.map_err(|e| {
                        format!(
                            "Failed to connect to target {}: {}\n\nTip: For local backend services or REST APIs, ensure the server process is running, listening, and that the port is accessible.",
                            candidate_url, e
                        )
                    })?
                } else {
                    return Err(format!("Failed to connect to target {}: {}", candidate_url, err));
                }
            } else {
                return Err(format!(
                    "Failed to connect to target {}: {}\n\nTip: Verify the host and port are reachable. For local or backend services, specify the protocol (e.g., http://localhost:8000) and ensure the service is currently running.",
                    candidate_url, err
                ));
            }
        }
    };

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let status_code = response.status().as_u16();

    // 3. Extract headers and body
    let resp_headers = response.headers().clone();
    let server_info = resp_headers.get("server").and_then(|h| h.to_str().ok()).map(|s| s.to_string());

    let mut response_headers_list = Vec::new();
    for (k, v) in resp_headers.iter() {
        if let Ok(val_str) = v.to_str() {
            response_headers_list.push((k.as_str().to_string(), val_str.to_string()));
        }
    }

    // Protect against OOM / decompression bombs (Cap HTML body parsing to 5MB)
    let content_len = response.content_length().unwrap_or(0);
    if content_len > 10 * 1024 * 1024 {
        return Err("Target response payload exceeds safe 10MB limit.".to_string());
    }

    let raw_text = response.text().await.unwrap_or_default();
    let html_body = if raw_text.len() > 5 * 1024 * 1024 {
        raw_text[..5 * 1024 * 1024].to_string()
    } else {
        raw_text
    };

    // 4. Run Core Analysis Modules
    let mut all_findings: Vec<Finding> = Vec::new();
    let mut detected_tech: Vec<String> = Vec::new();

    if let Some(srv) = &server_info {
        detected_tech.push(format!("Server: {}", srv));
    }

    // Passive CDN / WAF Edge Detection
    if resp_headers.contains_key("cf-ray") || server_info.as_deref().unwrap_or("").to_lowercase().contains("cloudflare") {
        if !detected_tech.iter().any(|t| t.contains("Cloudflare")) {
            detected_tech.push("WAF/CDN: Cloudflare".to_string());
        }
    }
    if resp_headers.contains_key("x-amz-cf-id") || resp_headers.get("via").and_then(|v| v.to_str().ok()).unwrap_or("").to_lowercase().contains("cloudfront") {
        if !detected_tech.iter().any(|t| t.contains("CloudFront")) {
            detected_tech.push("CDN: AWS CloudFront".to_string());
        }
    }
    if resp_headers.contains_key("x-fastly-request-id") || resp_headers.contains_key("fastly-restarts") {
        if !detected_tech.iter().any(|t| t.contains("Fastly")) {
            detected_tech.push("CDN: Fastly".to_string());
        }
    }
    if resp_headers.contains_key("x-akamai-transformed") {
        if !detected_tech.iter().any(|t| t.contains("Akamai")) {
            detected_tech.push("CDN: Akamai".to_string());
        }
    }
    if resp_headers.contains_key("x-varnish") {
        if !detected_tech.iter().any(|t| t.contains("Varnish")) {
            detected_tech.push("Cache: Varnish".to_string());
        }
    }

    // Security Headers Analysis
    let header_findings = headers::analyze_headers(&resp_headers, is_https);
    all_findings.extend(header_findings);

    // Cookies Analysis
    let cookie_findings = cookies::analyze_cookies(&resp_headers, is_https);
    all_findings.extend(cookie_findings);

    // Dependencies & Known CVEs
    let dep_findings = dependencies::analyze_dependencies(&html_body, &mut detected_tech);
    all_findings.extend(dep_findings);

    // Leaks, Insecure Forms, Secrets
    let leak_findings = leaks::analyze_leaks(&html_body, is_https);
    all_findings.extend(leak_findings);

    // 5. Extended Recon Modules (Subdomains, DNS & Email Security, Endpoint Hunter, Port Scanner)
    let is_private = is_local_or_private(&domain);

    let subdomains_fut = async {
        if opts.include_subdomains.unwrap_or(true) && !domain.is_empty() && !is_private {
            subdomains::discover_subdomains(&client, &domain).await
        } else {
            Vec::new()
        }
    };

    let dns_fut = async {
        if !domain.is_empty() && !is_private {
            dns::audit_dns_and_email_security(&client, &domain).await
        } else {
            (Default::default(), Vec::new())
        }
    };

    let endpoints_fut = async {
        endpoints::audit_endpoints(&client, &parsed_url).await
    };

    let port_scan_enabled = opts.enable_port_scan.unwrap_or(true);
    let port_profile = opts.port_scan_profile.clone().unwrap_or_else(|| "top20".to_string());
    let custom_ports_str = opts.custom_ports.clone();
    let port_timeout = opts.port_timeout_ms.or(Some(600));

    let ports_fut = async {
        if port_scan_enabled && !domain.is_empty() {
            ports::audit_ports(&domain, &port_profile, custom_ports_str.as_deref(), port_timeout).await
        } else {
            (Default::default(), Vec::new())
        }
    };

    // Run async recon concurrently
    let (subdomains_list, (dns_report, dns_findings), (endpoint_report, endpoint_findings), (port_report, port_findings)) =
        tokio::join!(subdomains_fut, dns_fut, endpoints_fut, ports_fut);

    all_findings.extend(dns_findings);
    all_findings.extend(endpoint_findings);
    all_findings.extend(port_findings);

    // 6. Calculate Metrics & Security Score
    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;
    let mut info_count = 0;

    let mut score_deductions = 0u32;

    for finding in &all_findings {
        match finding.severity {
            Severity::Critical => {
                critical_count += 1;
                score_deductions += 25;
            }
            Severity::High => {
                high_count += 1;
                score_deductions += 15;
            }
            Severity::Medium => {
                medium_count += 1;
                score_deductions += 8;
            }
            Severity::Low => {
                low_count += 1;
                score_deductions += 3;
            }
            Severity::Info => {
                info_count += 1;
                score_deductions += 1;
            }
        }
    }

    let security_score = 100u32.saturating_sub(score_deductions);
    let total_findings = all_findings.len();
    let scan_id = format!("scan_{}", Utc::now().timestamp_millis());

    let port_report_opt = if port_scan_enabled {
        Some(port_report)
    } else {
        None
    };

    Ok(ScanReport {
        id: scan_id,
        target_url: parsed_url.to_string(),
        scanned_at: Utc::now().to_rfc3339(),
        status_code,
        response_time_ms: duration_ms,
        security_score,
        total_findings,
        critical_count,
        high_count,
        medium_count,
        low_count,
        info_count,
        findings: all_findings,
        server_info,
        technologies_detected: detected_tech,
        response_headers: response_headers_list,
        subdomains: subdomains_list,
        dns_security: Some(dns_report),
        endpoint_report: Some(endpoint_report),
        port_report: port_report_opt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    fn test_missing_security_headers() {
        let headers = HeaderMap::new();
        let findings = headers::analyze_headers(&headers, true);
        let ids: Vec<String> = findings.into_iter().map(|f| f.id).collect();
        assert!(ids.contains(&"missing-csp".to_string()));
        assert!(ids.contains(&"missing-hsts".to_string()));
        assert!(ids.contains(&"missing-clickjacking-protection".to_string()));
    }

    #[test]
    fn test_insecure_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "set-cookie",
            HeaderValue::from_static("session_id=12345; Path=/; SameSite=Lax"),
        );
        let findings = cookies::analyze_cookies(&headers, true);
        let ids: Vec<String> = findings.into_iter().map(|f| f.id).collect();
        assert!(ids.contains(&"cookie-missing-httponly-session_id".to_string()));
        assert!(ids.contains(&"cookie-missing-secure-session_id".to_string()));
    }

    #[test]
    fn test_vulnerable_jquery_dependency() {
        let html = r#"<html><head><script src="https://code.jquery.com/jquery-1.12.4.min.js"></script></head><body></body></html>"#;
        let mut detected_tech = Vec::new();
        let findings = dependencies::analyze_dependencies(html, &mut detected_tech);
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.title.contains("jQuery v1.12.4")));
        assert!(detected_tech.contains(&"jQuery v1.12.4".to_string()));
    }

    #[test]
    fn test_leaks_and_insecure_forms() {
        let html = r#"<html><body><form method="get" action="/login"><input type="password" name="pwd"/></form><!-- TODO: fix admin db_pass in prod --></body></html>"#;
        let findings = leaks::analyze_leaks(html, true);
        let ids: Vec<String> = findings.into_iter().map(|f| f.id).collect();
        assert!(ids.contains(&"password-form-method-get".to_string()));
        assert!(ids.iter().any(|id| id.contains("sensitive-comment")));
    }
}

