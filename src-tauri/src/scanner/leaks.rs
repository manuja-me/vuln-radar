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

pub fn analyze_leaks(html_content: &str, is_https: bool) -> Vec<Finding> {
    let mut findings = Vec::new();
    let document = Html::parse_document(html_content);

    // 1. Insecure Form Submissions
    for form in document.select(&FORM_SELECTOR) {
        let action = form.value().attr("action").unwrap_or("");
        let method = form.value().attr("method").unwrap_or("get").to_lowercase();

        // Check if form submits via unencrypted HTTP on HTTPS site
        if is_https && action.starts_with("http://") {
            findings.push(Finding {
                id: "insecure-form-action-http".to_string(),
                title: "Insecure Form Action (Submits over Plaintext HTTP)".to_string(),
                severity: Severity::High,
                category: Category::InsecureForm,
                description: format!("Form action points to an unencrypted HTTP destination ('{}').", action),
                impact: "Form data submitted by users (passwords, PII) will be transmitted in plaintext across the network.".to_string(),
                remediation: "Ensure all form action URLs use HTTPS (or relative URLs).".to_string(),
                evidence: Some(format!("<form action=\"{}\" method=\"{}\">", action, method)),
                owasp_category: "A02:2021-Cryptographic Failures".to_string(),
                cve_id: None,
                references: vec!["https://cheatsheetseries.owasp.org/cheatsheets/Transport_Layer_Protection_Cheat_Sheet.html".to_string()],
            });
        }

        // Check if form with password field uses GET method
        if form.select(&PASSWORD_SELECTOR).next().is_some() && method == "get" {
            findings.push(Finding {
                id: "password-form-method-get".to_string(),
                title: "Password Form Uses HTTP GET Method".to_string(),
                severity: Severity::High,
                category: Category::InsecureForm,
                description: "A form containing a password field is configured to submit via HTTP GET.".to_string(),
                impact: "Passwords will be appended to the query string, exposing them in browser history, proxy logs, web server access logs, and Referer headers.".to_string(),
                remediation: "Change the form method to 'POST'.".to_string(),
                evidence: Some(format!("<form method=\"get\" action=\"{}\">", action)),
                owasp_category: "A02:2021-Cryptographic Failures".to_string(),
                cve_id: None,
                references: vec!["https://owasp.org/www-community/vulnerabilities/Information_exposure_through_query_strings_in_url".to_string()],
            });
        }
    }

    // 2. Mixed Content on HTTPS
    if is_https {
        let mut mixed_resources = Vec::new();
        for el in document.select(&MIXED_SELECTOR) {
            if let Some(src) = el.value().attr("src").or_else(|| el.value().attr("href")) {
                mixed_resources.push(src.to_string());
            }
        }

        if !mixed_resources.is_empty() {
            findings.push(Finding {
                id: "mixed-active-content".to_string(),
                title: "Mixed Active Content (HTTP Resources Loaded on HTTPS)".to_string(),
                severity: Severity::High,
                category: Category::TlsSsl,
                description: format!("The HTTPS page loads {} active unencrypted HTTP resources (scripts/stylesheets/iframes).", mixed_resources.len()),
                impact: "Man-in-the-Middle attackers can modify unencrypted HTTP scripts in transit to execute arbitrary JavaScript in victim browsers.".to_string(),
                remediation: "Serve all external scripts, stylesheets, and iframes over HTTPS.".to_string(),
                evidence: Some(mixed_resources.join("\n")),
                owasp_category: "A02:2021-Cryptographic Failures".to_string(),
                cve_id: None,
                references: vec!["https://developer.mozilla.org/en-US/docs/Web/Security/Mixed_content".to_string()],
            });
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

            findings.push(Finding {
                id: format!("exposed-secret-{}", pattern.name.to_lowercase().replace(' ', "-")),
                title: format!("Potential Hardcoded Secret Exposed: {}", pattern.name),
                severity: pattern.severity.clone(),
                category: Category::InformationDisclosure,
                description: format!("A pattern matching a {} was detected in the client-accessible source code.", pattern.name),
                impact: "Exposed API credentials allow unauthorized attackers to access cloud infrastructure, APIs, or internal databases.".to_string(),
                remediation: "Revoke the exposed key immediately and store credentials in secure server-side environment variables.".to_string(),
                evidence: Some(format!("Detected Pattern: {}", masked)),
                owasp_category: "A07:2021-Identification and Authentication Failures".to_string(),
                cve_id: None,
                references: vec!["https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html".to_string()],
            });
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
                    findings.push(Finding {
                        id: format!("sensitive-comment-{}", keyword.replace(' ', "-")),
                        title: format!("Sensitive Comment Discovered in HTML Source ('{}')", keyword),
                        severity: Severity::Low,
                        category: Category::InformationDisclosure,
                        description: "HTML source comments contain developer notes or credentials that should not be visible to public users.".to_string(),
                        impact: "Assists attackers in discovering internal logic, test endpoints, or forgotten credentials.".to_string(),
                        remediation: "Strip HTML and code comments during your production build process.".to_string(),
                        evidence: Some(format!("<!-- {} -->", if comment_text.len() > 120 { &comment_text[..120] } else { comment_text })),
                        owasp_category: "A05:2021-Security Misconfiguration".to_string(),
                        cve_id: None,
                        references: vec!["https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/01-Information_Gathering/05-Review_Webpage_Comments_and_Metadata_for_Information_Leakage".to_string()],
                    });
                    break;
                }
            }
        }
    }

    findings
}
