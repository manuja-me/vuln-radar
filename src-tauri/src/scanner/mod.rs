pub mod cookies;
pub mod dependencies;
pub mod headers;
pub mod leaks;

use crate::models::{Finding, ScanReport, Severity};
use chrono::Utc;
use reqwest::Client;
use std::time::{Duration, Instant};
use url::Url;

pub async fn run_scan(target_url: &str) -> Result<ScanReport, String> {
    // 1. Normalize and parse URL
    let formatted_url = if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        format!("https://{}", target_url)
    } else {
        target_url.to_string()
    };

    let parsed_url = Url::parse(&formatted_url).map_err(|e| format!("Invalid URL format: {}", e))?;
    let is_https = parsed_url.scheme() == "https";

    // 2. Build HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 VulnRadar/1.0")
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| format!("Failed to initialize HTTP client: {}", e))?;

    let start_time = Instant::now();
    let response = client
        .get(parsed_url.as_str())
        .send()
        .await
        .map_err(|e| format!("Failed to connect to target {}: {}", formatted_url, e))?;

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

    let html_body = response.text().await.unwrap_or_default();

    // 4. Run Analysis Modules
    let mut all_findings: Vec<Finding> = Vec::new();
    let mut detected_tech: Vec<String> = Vec::new();

    if let Some(srv) = &server_info {
        detected_tech.push(format!("Server: {}", srv));
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

    // 5. Calculate Metrics & Security Score
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

    Ok(ScanReport {
        id: scan_id,
        target_url: formatted_url,
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

