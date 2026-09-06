use crate::models::{Category, Finding, Severity};
use regex::Regex;
use scraper::{Html, Selector};
use std::sync::LazyLock;

static FORM_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("form").unwrap());
static PASSWORD_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("input[type=\"password\"]").unwrap());
static MIXED_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("script[src^=\"http://\"], link[href^=\"http://\"], iframe[src^=\"http://\"]").unwrap()
});

static COMMENT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<!--([\s\S]*?)-->").unwrap());

static AWS_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"AKIA[0-9A-Z]{16}").unwrap());
static GCP_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"AIza[0-9A-Za-z-_]{35}").unwrap());
static SLACK_TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"xox[baprs]-[0-9a-zA-Z]{10,48}").unwrap());
static GITHUB_PAT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"gh[pousr]_[0-9a-zA-Z]{36,255}").unwrap());
static RSA_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-----BEGIN (?:RSA|EC|OPENSSH)? ?PRIVATE KEY-----").unwrap());

struct SecretPattern {
    name: &'static str,
    regex: &'static LazyLock<Regex>,
    severity: Severity,
}

static SECRET_PATTERNS: [SecretPattern; 5] = [
    SecretPattern {
        name: "AWS Access Key ID",
        regex: &AWS_KEY_REGEX,
        severity: Severity::Critical,
    },
    SecretPattern {
        name: "Google Cloud / Maps API Key",
        regex: &GCP_KEY_REGEX,
        severity: Severity::Medium,
    },
    SecretPattern {
        name: "Slack Webhook / Token",
        regex: &SLACK_TOKEN_REGEX,
        severity: Severity::High,
    },
    SecretPattern {
        name: "GitHub Personal Access Token",
        regex: &GITHUB_PAT_REGEX,
        severity: Severity::Critical,
    },
    SecretPattern {
        name: "Generic RSA/EC Private Key Header",
        regex: &RSA_KEY_REGEX,
        severity: Severity::Critical,
    },
];

pub fn analyze_leaks(document: &Html, html_content: &str, is_https: bool) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. Insecure Form Submissions
    for form in document.select(&FORM_SELECTOR) {
        let action = form.value().attr("action").unwrap_or("");
        let method = form.value().attr("method").unwrap_or("get").to_lowercase();

        // Check if form submits via unencrypted HTTP on HTTPS site
        if is_https && action.starts_with("http://") {
            findings.push(
                Finding::new(
                    "insecure-form-action-http",
                    "Insecure Form Action (Submits over Plaintext HTTP)",
                    Severity::High,
                    Category::InsecureForm,
                    format!("Form action points to an unencrypted HTTP destination ('{}').", action),
                    "Form data submitted by users (passwords, PII) will be transmitted in plaintext across the network.",
                    "Ensure all form action URLs use HTTPS (or relative URLs).",
                    "A02:2021-Cryptographic Failures",
                )
                .with_evidence(format!("<form action=\"{}\" method=\"{}\">", action, method))
                .with_refs(&["https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Protection_Cheat_Sheet.html"]),
            );
        }

        // Check if form with password field uses GET method
        if form.select(&PASSWORD_SELECTOR).next().is_some() && method == "get" {
            findings.push(
                Finding::new(
                    "password-form-method-get",
                    "Password Form Uses HTTP GET Method",
                    Severity::High,
                    Category::InsecureForm,
                    "A form containing a password field is configured to submit via HTTP GET.",
                    "Passwords will be appended to the query string, exposing them in browser history, proxy logs, web server access logs, and Referer headers.",
                    "Change the form method to 'POST'.",
                    "A02:2021-Cryptographic Failures",
                )
                .with_evidence(format!("<form method=\"get\" action=\"{}\">", action))
                .with_refs(&["https://owasp.org/www-community/vulnerabilities/Information_exposure_through_query_strings_in_url"]),
            );
        }
    }

    // 2. Mixed Content on HTTPS
    if is_https {
        let mixed_resources: Vec<String> = document
            .select(&MIXED_SELECTOR)
            .filter_map(|el| el.value().attr("src").or_else(|| el.value().attr("href")).map(String::from))
            .collect();

        if !mixed_resources.is_empty() {
            findings.push(
                Finding::new(
                    "mixed-active-content",
                    "Mixed Active Content (HTTP Resources Loaded on HTTPS)",
                    Severity::High,
                    Category::TlsSsl,
                    format!("The HTTPS page loads {} active unencrypted HTTP resources (scripts/stylesheets/iframes).", mixed_resources.len()),
                    "Man-in-the-Middle attackers can modify unencrypted HTTP scripts in transit to execute arbitrary JavaScript in victim browsers.",
                    "Serve all external scripts, stylesheets, and iframes over HTTPS.",
                    "A02:2021-Cryptographic Failures",
                )
                .with_evidence(mixed_resources.join("\n"))
                .with_refs(&["https://developer.mozilla.org/en-US/docs/Web/Security/Mixed_content"]),
            );
        }
    }

    // 3. Exposed Secrets / API Keys in HTML source (Static Zero-Allocation Evaluation)
    for pattern in &SECRET_PATTERNS {
        let re = &pattern.regex;
        if let Some(mat) = re.find(html_content) {
            let matched_text = mat.as_str();
            let masked = if matched_text.len() > 8 {
                format!("{}...{}", &matched_text[..4], &matched_text[matched_text.len() - 4..])
            } else {
                "***".to_string()
            };

            findings.push(
                Finding::new(
                    format!("exposed-secret-{}", pattern.name.to_lowercase().replace(' ', "-")),
                    format!("Potential Hardcoded Secret Exposed: {}", pattern.name),
                    pattern.severity,
                    Category::InformationDisclosure,
                    format!("A pattern matching a {} was detected in the client-accessible source code.", pattern.name),
                    "Exposed API credentials allow unauthorized attackers to access cloud infrastructure, APIs, or internal databases.",
                    "Revoke the exposed key immediately and store credentials in secure server-side environment variables.",
                    "A07:2021-Identification and Authentication Failures",
                )
                .with_evidence(format!("Detected Pattern: {}", masked))
                .with_refs(&["https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html"]),
            );
        }
    }

    // 4. Sensitive Information in HTML Comments
    let sensitive_keywords = ["password", "secret", "internal api", "todo: fix security", "admin login", "db_pass", "mysql://", "postgres://", "mongodb://"];

    for caps in COMMENT_REGEX.captures_iter(html_content) {
        if let Some(comment_match) = caps.get(1) {
            let comment_text = comment_match.as_str().trim();
            let comment_lower = comment_text.to_lowercase();

            for keyword in &sensitive_keywords {
                if comment_lower.contains(keyword) {
                    findings.push(
                        Finding::new(
                            format!("sensitive-comment-{}", keyword.replace(' ', "-")),
                            format!("Sensitive Comment Discovered in HTML Source ('{}')", keyword),
                            Severity::Low,
                            Category::InformationDisclosure,
                            "HTML source comments contain developer notes or credentials that should not be visible to public users.",
                            "Assists attackers in discovering internal logic, test endpoints, or forgotten credentials.",
                            "Strip HTML and code comments during your production build process.",
                            "A05:2021-Security Misconfiguration",
                        )
                        .with_evidence(format!("<!-- {} -->", if comment_text.len() > 120 { &comment_text[..120] } else { comment_text }))
                        .with_refs(&["https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/05-Review_Webpage_Comments_and_Metadata_for_Information_Leakage"]),
                    );
                    break;
                }
            }
        }
    }

    findings
}
