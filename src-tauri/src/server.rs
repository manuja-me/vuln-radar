use crate::db::Database;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Debug, serde::Deserialize)]
pub struct AnalyzePathsRequest {
    #[serde(alias = "websiteUrl", alias = "target_url", alias = "targetUrl")]
    pub website_url: Option<String>,
    #[serde(alias = "wordList", alias = "paths")]
    pub word_list: Option<Vec<String>>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, serde::Serialize)]
pub struct PathResult {
    pub path: String,
    pub status_code: u16,
    pub content_length: usize,
    pub content_type: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AnalyzePathsResponse {
    pub results: Vec<PathResult>,
    pub total_tested: usize,
    pub non_404_found: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_secs: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub allowed: bool,
    pub limit: usize,
    pub remaining: usize,
    pub retry_after_secs: u64,
}

/// Thread-safe in-memory sliding window rate limiter per IP address.
pub struct EndpointRateLimiter {
    requests: Mutex<HashMap<IpAddr, Vec<Instant>>>,
    max_per_window: usize,
    window_duration: Duration,
}

impl EndpointRateLimiter {
    pub fn new(max_per_window: usize, window_duration: Duration) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_per_window,
            window_duration,
        }
    }

    pub async fn check(&self, ip: IpAddr) -> RateLimitStatus {
        let mut map = self.requests.lock().await;
        let now = Instant::now();
        let timestamps = map.entry(ip).or_default();

        // Expire older timestamps outside sliding window
        timestamps.retain(|&t| now.duration_since(t) < self.window_duration);

        if timestamps.len() >= self.max_per_window {
            let oldest = timestamps.first().copied().unwrap_or(now);
            let elapsed = now.duration_since(oldest);
            let retry_after = self
                .window_duration
                .saturating_sub(elapsed)
                .as_secs()
                .max(1);

            RateLimitStatus {
                allowed: false,
                limit: self.max_per_window,
                remaining: 0,
                retry_after_secs: retry_after,
            }
        } else {
            timestamps.push(now);
            let remaining = self.max_per_window - timestamps.len();

            RateLimitStatus {
                allowed: true,
                limit: self.max_per_window,
                remaining,
                retry_after_secs: 0,
            }
        }
    }
}

/// Sanitizes website URL to prevent SSRF, CRLF injection, credential leakage,
/// and scheme smuggling attacks.
pub fn sanitize_url(raw_url: &str) -> Result<url::Url, &'static str> {
    let trimmed = raw_url.trim();

    if trimmed.is_empty() {
        return Err("URL cannot be empty");
    }
    if trimmed.len() > 2048 {
        return Err("URL exceeds maximum length of 2048 characters");
    }

    // 1. Reject unprintable control characters and raw CRLF injection
    if trimmed.bytes().any(|b| b <= 0x1F || b == 0x7F) {
        return Err("URL contains illegal control characters or line breaks");
    }

    // 2. Reject URL-encoded CRLF and null bytes
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("%0d") || lower.contains("%0a") || lower.contains("%00") {
        return Err("URL contains encoded CRLF or null bytes");
    }

    // 3. Strict scheme validation: must start with http:// or https://
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return Err("URL must start with http:// or https://");
    }

    // 4. Parse with the url crate
    let parsed = url::Url::parse(trimmed).map_err(|_| "Malformed URL syntax")?;

    // 5. Enforce HTTP/HTTPS scheme strictly
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("Only http and https schemes are permitted");
    }

    // 6. Reject userinfo/credentials (http://user:pass@host) to prevent credential leakage
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URLs with embedded authentication credentials are not permitted");
    }

    // 7. Validate Host
    let host = match parsed.host_str() {
        Some(h) if !h.trim().is_empty() => h.trim().to_ascii_lowercase(),
        _ => return Err("URL must include a valid host domain or IP address"),
    };

    if host.len() > 253 {
        return Err("Host name exceeds maximum length of 253 characters");
    }

    // 8. Prevent SSRF targeting Cloud Metadata Services (AWS, GCP, Azure, OpenStack IMDS)
    if host == "169.254.169.254"
        || host == "metadata.google.internal"
        || host == "100.100.100.200"
        || host.starts_with("169.254.")
        || host == "[fd00:ec2::254]"
        || host.starts_with("[fe80:")
    {
        return Err("Access to cloud metadata services is strictly prohibited");
    }

    // 9. Validate port range if specified
    if let Some(port) = parsed.port() {
        if port == 0 {
            return Err("Port 0 is invalid");
        }
    }

    Ok(parsed)
}

/// Sanitizes word list entries to prevent CRLF injection, command injection,
/// open-redirect host overrides, null-byte exploits, and DoS buffer overflow.
pub fn sanitize_word_list(words: &[String]) -> Result<Vec<String>, &'static str> {
    const MAX_WORDLIST_ITEMS: usize = 10_000;
    const MAX_PATH_LENGTH: usize = 512;

    if words.is_empty() {
        return Err("Word list is empty");
    }

    if words.len() > MAX_WORDLIST_ITEMS {
        return Err("Word list exceeds maximum limit of 10,000 entries");
    }

    let mut cleaned = Vec::with_capacity(words.len().min(1000));
    let mut seen = HashSet::new();

    for raw in words {
        let trimmed = raw.trim();

        // Skip empty lines or comment lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Enforce max path length per entry (mitigate buffer exhaustion)
        if trimmed.len() > MAX_PATH_LENGTH {
            continue;
        }

        // Reject null bytes, raw CRLF, and unprintable control characters
        if trimmed.bytes().any(|b| b <= 0x1F || b == 0x7F) {
            continue;
        }

        // Check for URL-encoded CRLF or null bytes
        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("%0d") || lower.contains("%0a") || lower.contains("%00") {
            continue;
        }

        // Reject dangerous characters that could trigger injection vulnerabilities
        let has_disallowed = trimmed.chars().any(|c| matches!(c, '"' | '\'' | '\\' | '<' | '>' | '`' | '|' | '^' | '{' | '}' | ' ' | '\t'));
        if has_disallowed {
            continue;
        }

        // Strip leading slashes to prevent schema-relative host override (e.g. //attacker.com)
        let mut path_part = trimmed.trim_start_matches('/');

        // Strip embedded protocol schemes if present
        if path_part.to_ascii_lowercase().starts_with("http:")
            || path_part.to_ascii_lowercase().starts_with("https:")
        {
            continue;
        }
        path_part = path_part.trim_start_matches('/');

        if path_part.is_empty() {
            continue;
        }

        // Normalize multiple consecutive slashes (e.g. ///admin -> /admin)
        let mut normalized = String::with_capacity(path_part.len() + 1);
        normalized.push('/');
        let mut prev_slash = true;
        for c in path_part.chars() {
            if c == '/' {
                if !prev_slash {
                    normalized.push('/');
                    prev_slash = true;
                }
            } else {
                normalized.push(c);
                prev_slash = false;
            }
        }

        // Preserve unique paths
        if seen.insert(normalized.clone()) {
            cleaned.push(normalized);
        }
    }

    if cleaned.is_empty() {
        return Err("No valid paths found after security sanitization");
    }

    Ok(cleaned)
}

/// Helper function to transmit standardized HTTP responses with custom headers.
async fn send_response<W: AsyncWriteExt + Unpin>(
    stream: &mut W,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &[u8],
    extra_headers: &[(&str, &str)],
) -> Result<(), tokio::io::Error> {
    let mut header = format!(
        "HTTP/1.1 {} {}\r\n\
        Content-Type: {}\r\n\
        Content-Length: {}\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
        Access-Control-Allow-Headers: *\r\n\
        Connection: close\r\n",
        status_code,
        status_text,
        content_type,
        body.len()
    );

    for (k, v) in extra_headers {
        header.push_str(&format!("{}: {}\r\n", k, v));
    }
    header.push_str("\r\n");

    stream.write_all(header.as_bytes()).await?;
    if !body.is_empty() {
        stream.write_all(body).await?;
    }
    stream.flush().await?;
    Ok(())
}

/// Helper function to transmit standardized JSON error responses.
async fn send_error_response<W: AsyncWriteExt + Unpin>(
    stream: &mut W,
    status_code: u16,
    status_text: &str,
    error: &str,
    message: &str,
    extra_headers: &[(&str, &str)],
) {
    let retry_after = extra_headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("Retry-After"))
        .and_then(|(_, v)| v.parse::<u64>().ok());

    let err_obj = ApiErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
        status: status_code,
        retry_after_secs: retry_after,
    };
    let json_bytes = serde_json::to_vec(&err_obj).unwrap_or_default();
    let _ = send_response(
        stream,
        status_code,
        status_text,
        "application/json; charset=utf-8",
        &json_bytes,
        extra_headers,
    )
    .await;
}

/// Reads HTTP headers and content body according to Content-Length.
async fn read_full_request<R: AsyncReadExt + Unpin>(
    stream: &mut R,
) -> Result<(String, String, Vec<u8>), ()> {
    let mut data = Vec::with_capacity(4096);
    let mut buf = [0u8; 2048];
    let mut header_end = None;
    let mut content_length = 0usize;

    loop {
        let n = stream.read(&mut buf).await.map_err(|_| ())?;
        if n == 0 {
            break;
        }
        data.extend_from_slice(&buf[..n]);

        if header_end.is_none() {
            if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                let header_str = String::from_utf8_lossy(&data[..pos]);
                for line in header_str.lines() {
                    let lower = line.to_ascii_lowercase();
                    if lower.starts_with("content-length:") {
                        if let Some(val) = line.split(':').nth(1) {
                            content_length = val.trim().parse::<usize>().unwrap_or(0);
                        }
                    }
                }
            }
        }

        if let Some(hend) = header_end {
            if data.len() >= hend + content_length {
                break;
            }
        }

        if data.len() > 10 * 1024 * 1024 {
            return Err(());
        }
    }

    let hend = header_end.ok_or(())?;
    let header_str = String::from_utf8_lossy(&data[..hend - 4]);
    let body = if content_length > 0 && data.len() >= hend {
        let end = (hend + content_length).min(data.len());
        data[hend..end].to_vec()
    } else {
        Vec::new()
    };

    let request_line = header_str.lines().next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();

    Ok((method, path, body))
}

/// Starts local HTTP server providing wordlist downloads and rate-limited,
/// sanitized path analysis endpoint with comprehensive error handling.
pub async fn start_wordlist_http_server(db: Arc<Database>) -> u16 {
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind wordlist HTTP server: {}", e);
            return 0;
        }
    };

    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(_) => 0,
    };

    // Rate limiter configured: max 60 requests per 1 minute window per IP
    let rate_limiter = Arc::new(EndpointRateLimiter::new(60, Duration::from_secs(60)));

    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, peer_addr)) = listener.accept().await {
                let db_clone = db.clone();
                let limiter = rate_limiter.clone();
                let peer_ip = peer_addr.ip();

                tokio::spawn(async move {
                    let (method, path, body) = match read_full_request(&mut stream).await {
                        Ok(req) => req,
                        Err(_) => {
                            send_error_response(
                                &mut stream,
                                400,
                                "Bad Request",
                                "Invalid Request",
                                "Failed to parse incoming HTTP stream",
                                &[],
                            )
                            .await;
                            return;
                        }
                    };

                    // Handle CORS preflight
                    if method == "OPTIONS" {
                        let _ = send_response(
                            &mut stream,
                            204,
                            "No Content",
                            "text/plain",
                            b"",
                            &[],
                        )
                        .await;
                        return;
                    }

                    // POST /analyze_paths endpoint
                    if method == "POST"
                        && (path == "/analyze_paths"
                            || path == "/api/analyze_paths"
                            || path.starts_with("/analyze_paths?")
                            || path.starts_with("/api/analyze_paths?"))
                    {
                        // 1. Rate Limiting Check
                        let limit_status = limiter.check(peer_ip).await;
                        let limit_str = limit_status.limit.to_string();
                        let remaining_str = limit_status.remaining.to_string();
                        let retry_str = limit_status.retry_after_secs.to_string();

                        if !limit_status.allowed {
                            send_error_response(
                                &mut stream,
                                429,
                                "Too Many Requests",
                                "Rate Limit Exceeded",
                                "Maximum request threshold reached for this endpoint. Please retry later.",
                                &[
                                    ("Retry-After", &retry_str),
                                    ("X-RateLimit-Limit", &limit_str),
                                    ("X-RateLimit-Remaining", "0"),
                                ],
                            )
                            .await;
                            return;
                        }

                        // 2. Request Deserialization with Graceful Error Handling
                        let req: AnalyzePathsRequest = match serde_json::from_slice(&body) {
                            Ok(r) => r,
                            Err(err) => {
                                send_error_response(
                                    &mut stream,
                                    400,
                                    "Bad Request",
                                    "Malformed JSON",
                                    &format!("Failed to parse request JSON payload: {}", err),
                                    &[
                                        ("X-RateLimit-Limit", &limit_str),
                                        ("X-RateLimit-Remaining", &remaining_str),
                                    ],
                                )
                                .await;
                                return;
                            }
                        };

                        let raw_url = req.website_url.unwrap_or_default();
                        let raw_words = req.word_list.unwrap_or_default();

                        // 3. Validation of Required Input Parameters
                        if raw_url.trim().is_empty() || raw_words.is_empty() {
                            send_error_response(
                                &mut stream,
                                400,
                                "Bad Request",
                                "Missing Parameters",
                                "Both 'website_url' and 'word_list' are required and cannot be empty.",
                                &[
                                    ("X-RateLimit-Limit", &limit_str),
                                    ("X-RateLimit-Remaining", &remaining_str),
                                ],
                            )
                            .await;
                            return;
                        }

                        // 4. URL Sanitization Error Handling
                        let base_url = match sanitize_url(&raw_url) {
                            Ok(u) => u,
                            Err(err_msg) => {
                                send_error_response(
                                    &mut stream,
                                    400,
                                    "Bad Request",
                                    "Invalid Website URL",
                                    err_msg,
                                    &[
                                        ("X-RateLimit-Limit", &limit_str),
                                        ("X-RateLimit-Remaining", &remaining_str),
                                    ],
                                )
                                .await;
                                return;
                            }
                        };

                        // 5. Wordlist Sanitization Error Handling
                        let sanitized_words = match sanitize_word_list(&raw_words) {
                            Ok(words) => words,
                            Err(err_msg) => {
                                send_error_response(
                                    &mut stream,
                                    400,
                                    "Bad Request",
                                    "Invalid Word List",
                                    err_msg,
                                    &[
                                        ("X-RateLimit-Limit", &limit_str),
                                        ("X-RateLimit-Remaining", &remaining_str),
                                    ],
                                )
                                .await;
                                return;
                            }
                        };

                        // 6. HTTP Client Initialization Error Handling
                        let timeout_sec = req.timeout_seconds.unwrap_or(5).clamp(1, 30);
                        let client = match reqwest::Client::builder()
                            .timeout(Duration::from_secs(timeout_sec))
                            .redirect(reqwest::redirect::Policy::limited(3))
                            .danger_accept_invalid_certs(true)
                            .build()
                        {
                            Ok(c) => c,
                            Err(err) => {
                                send_error_response(
                                    &mut stream,
                                    500,
                                    "Internal Server Error",
                                    "Client Initialization Failed",
                                    &format!("Failed to initialize backend probe client: {}", err),
                                    &[],
                                )
                                .await;
                                return;
                            }
                        };

                        // 7. Concurrent Path Execution with Per-Probe Exception Handling
                        let semaphore = Arc::new(tokio::sync::Semaphore::new(10));
                        let mut tasks = Vec::new();
                        let total_words = sanitized_words.len();

                        for path in sanitized_words {
                            let sem = semaphore.clone();
                            let client_clone = client.clone();

                            let mut target_url = base_url.clone();
                            let base_prefix = target_url.path().trim_end_matches('/');
                            let combined_path =
                                format!("{}/{}", base_prefix, path.trim_start_matches('/'));
                            target_url.set_path(&combined_path);

                            tasks.push(tokio::spawn(async move {
                                let _permit = sem.acquire().await.ok();
                                // Gracefully catch per-path connection/timeout exceptions
                                let response = match client_clone.get(target_url).send().await {
                                    Ok(resp) => resp,
                                    Err(_) => return None, // Network failure/timeout handled gracefully
                                };

                                let status = response.status().as_u16();
                                if status != 404 {
                                    let content_type = response
                                        .headers()
                                        .get(reqwest::header::CONTENT_TYPE)
                                        .and_then(|v| v.to_str().ok())
                                        .unwrap_or("")
                                        .to_string();
                                    let bytes = response.bytes().await.unwrap_or_default();
                                    Some(PathResult {
                                        path,
                                        status_code: status,
                                        content_length: bytes.len(),
                                        content_type,
                                    })
                                } else {
                                    None
                                }
                            }));
                        }

                        let mut results = Vec::new();
                        for t in tasks {
                            if let Ok(Some(item)) = t.await {
                                results.push(item);
                            }
                        }

                        let non_404_found = results.len();
                        let resp_obj = AnalyzePathsResponse {
                            results,
                            total_tested: total_words,
                            non_404_found,
                        };

                        let json_bytes = serde_json::to_vec(&resp_obj).unwrap_or_default();
                        let _ = send_response(
                            &mut stream,
                            200,
                            "OK",
                            "application/json; charset=utf-8",
                            &json_bytes,
                            &[
                                ("X-RateLimit-Limit", &limit_str),
                                ("X-RateLimit-Remaining", &remaining_str),
                            ],
                        )
                        .await;
                        return;
                    }

                    // GET /api/wordlist/download
                    if method == "GET" && path.starts_with("/api/wordlist/download") {
                        let id = if let Some(q_idx) = path.find('?') {
                            let query = &path[q_idx + 1..];
                            query
                                .split('&')
                                .find_map(|pair| {
                                    let mut kv = pair.split('=');
                                    if kv.next() == Some("id") {
                                        kv.next().map(|v| v.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_default()
                        } else {
                            String::new()
                        };

                        let (filename, content) = if !id.is_empty() {
                            if let Ok(Some(record)) = db_clone.get_wordlist_by_id(&id) {
                                let safe_name = format!(
                                    "{}.txt",
                                    record.id.replace(
                                        |c: char| !c.is_alphanumeric() && c != '_' && c != '-',
                                        "_"
                                    )
                                );
                                (safe_name, record.paths.join("\n"))
                            } else {
                                let defaults = crate::wordlists::get_predefined_records();
                                if let Some(rec) = defaults.into_iter().find(|r| r.id == id) {
                                    (format!("{}.txt", rec.id), rec.paths.join("\n"))
                                } else {
                                    ("wordlist.txt".to_string(), String::new())
                                }
                            }
                        } else {
                            ("wordlist.txt".to_string(), String::new())
                        };

                        let content_bytes = content.as_bytes();
                        let cd_header = format!("attachment; filename=\"{}\"", filename);
                        let _ = send_response(
                            &mut stream,
                            200,
                            "OK",
                            "text/plain; charset=utf-8",
                            content_bytes,
                            &[
                                ("Content-Disposition", &cd_header),
                                ("Access-Control-Expose-Headers", "Content-Disposition"),
                            ],
                        )
                        .await;
                    } else {
                        send_error_response(
                            &mut stream,
                            404,
                            "Not Found",
                            "Resource Not Found",
                            "The requested path does not exist on this server.",
                            &[],
                        )
                        .await;
                    }
                });
            }
        }
    });

    port
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_rate_limiter_allow_and_throttle() {
        let limiter = EndpointRateLimiter::new(3, Duration::from_secs(2));
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First 3 requests should be permitted
        let res1 = limiter.check(ip).await;
        assert!(res1.allowed);
        assert_eq!(res1.remaining, 2);

        let res2 = limiter.check(ip).await;
        assert!(res2.allowed);
        assert_eq!(res2.remaining, 1);

        let res3 = limiter.check(ip).await;
        assert!(res3.allowed);
        assert_eq!(res3.remaining, 0);

        // 4th request must be throttled with positive retry_after
        let res4 = limiter.check(ip).await;
        assert!(!res4.allowed);
        assert!(res4.retry_after_secs > 0);
    }

    #[test]
    fn test_sanitize_url_valid() {
        assert!(sanitize_url("https://example.com").is_ok());
        assert!(sanitize_url("http://localhost:8080/test").is_ok());
        assert!(sanitize_url("https://sub.domain.co.uk:8443/app/").is_ok());
    }

    #[test]
    fn test_sanitize_url_rejections() {
        assert!(sanitize_url("ftp://example.com").is_err());
        assert!(sanitize_url("javascript:alert(1)").is_err());
        assert!(sanitize_url("file:///etc/passwd").is_err());
        assert!(sanitize_url("data:text/html,<html>").is_err());
        assert!(sanitize_url("not a url").is_err());
        assert!(sanitize_url("").is_err());

        assert!(sanitize_url("http://admin:secret@example.com").is_err());
        assert!(sanitize_url("https://attacker@legitimate.com").is_err());

        assert!(sanitize_url("http://169.254.169.254/latest/meta-data").is_err());
        assert!(sanitize_url("http://metadata.google.internal").is_err());
        assert!(sanitize_url("http://100.100.100.200").is_err());

        assert!(sanitize_url("https://example.com\r\nInjected:header").is_err());
        assert!(sanitize_url("https://example.com%0d%0aInjected:header").is_err());
        assert!(sanitize_url("https://example.com%00").is_err());
    }

    #[test]
    fn test_sanitize_word_list() {
        let input = vec![
            "admin".to_string(),
            "# comment line".to_string(),
            "".to_string(),
            "login/page.php".to_string(),
            "bad\"char".to_string(),
            "bad'quote".to_string(),
            "<script>alert(1)</script>".to_string(),
            "`whoami`".to_string(),
            "path\r\nCRLF".to_string(),
            "path%0d%0aInjected".to_string(),
            "path%00null".to_string(),
            "///multiple///slashes".to_string(),
            "//attacker.com/override".to_string(),
            "valid-path_123".to_string(),
            "admin".to_string(), // duplicate
        ];

        let cleaned = sanitize_word_list(&input).unwrap();
        assert_eq!(
            cleaned,
            vec![
                "/admin".to_string(),
                "/login/page.php".to_string(),
                "/multiple/slashes".to_string(),
                "/attacker.com/override".to_string(),
                "/valid-path_123".to_string()
            ]
        );
    }

    #[test]
    fn test_host_preservation_against_override() {
        let base = sanitize_url("https://victim.com").unwrap();
        let paths = vec!["//attacker.com/evil".to_string()];
        let sanitized = sanitize_word_list(&paths).unwrap();

        let mut target = base.clone();
        let base_prefix = target.path().trim_end_matches('/');
        let combined = format!("{}/{}", base_prefix, sanitized[0].trim_start_matches('/'));
        target.set_path(&combined);

        assert_eq!(target.host_str().unwrap(), "victim.com");
        assert_eq!(target.scheme(), "https");
        assert_eq!(target.path(), "/attacker.com/evil");
    }

    #[test]
    fn test_request_payload_deserialization() {
        let json = r#"{"website_url": "https://example.com", "word_list": ["admin", "api"]}"#;
        let req: AnalyzePathsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.website_url, Some("https://example.com".to_string()));
        assert_eq!(
            req.word_list,
            Some(vec!["admin".to_string(), "api".to_string()])
        );
    }

    #[test]
    fn test_error_response_serialization() {
        let err = ApiErrorResponse {
            error: "Bad Request".to_string(),
            message: "Missing parameter".to_string(),
            status: 400,
            retry_after_secs: None,
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("Bad Request"));
        assert!(json.contains("Missing parameter"));
        assert!(!json.contains("retry_after_secs"));
    }
}
