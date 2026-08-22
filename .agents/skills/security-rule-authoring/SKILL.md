---
name: security-rule-authoring
description: >-
  Authoritative guide for implementing new vulnerability checks, CVE rules, security headers, and passive reconnaissance modules in VulnRadar.
  Use this skill whenever adding or modifying scanner rules, OWASP category mappings, remediation guidance, or inspection logic in Rust (src-tauri/src/scanner/).
---

# 🛡️ VulnRadar Security Rule & Vulnerability Authoring Guide

This skill specifies how to author, test, and integrate new security analysis checks into VulnRadar's Rust scanning engine.

---

## 📐 Core Architecture & Rule Standard

All security checks reside under [`src-tauri/src/scanner/`](file:///c:/Users/user/Documents/GitHub/vuln-radar/src-tauri/src/scanner/). Every detected weakness must produce a structured [`Finding`](file:///c:/Users/user/Documents/GitHub/vuln-radar/src-tauri/src/models.rs) object.

### Finding Schema Requirements
1. **`id`**: Unique kebab-case slug (e.g. `missing-hsts`, `exposed-aws-key`, `cve-2020-11022`).
2. **`title`**: Concise, executive-readable title (e.g. *"Missing Strict-Transport-Security Header"*).
3. **`severity`**: Must be one of:
   - `Severity::Critical` (Direct RCE, credential leak, active spoofing vulnerability)
   - `Severity::High` (Clickjacking, severe XSS risk, outdated CVE library with known exploit)
   - `Severity::Medium` (Missing Content-Security-Policy, insecure cookie SameSite flags)
   - `Severity::Low` (Missing Referrer-Policy, Permissions-Policy)
   - `Severity::Info` (Server technology banner disclosure)
4. **`category`**: Semantic classification (e.g., `security_headers`, `cookie_security`, `dns_email_security`, `endpoint_exposure`, `vulnerable_dependency`, `information_disclosure`, `cors_misconfiguration`).
5. **`owasp_category`**: Standard OWASP 2021 mapping (e.g., `A05:2021-Security Misconfiguration`, `A01:2021-Broken Access Control`, `A06:2021-Vulnerable and Outdated Components`).
6. **`impact`**: Explains *what an attacker can achieve* if this flaw is exploited.
7. **`remediation`**: Actionable, exact configuration guidance (e.g. Nginx, Apache, or Cloudflare directive).
8. **`evidence`**: Exact header value, HTML line, or DNS record triggered.
9. **`references`**: Array of authoritative URLs (e.g. MDN, OWASP WSTG, RFC, NVD CVE).

---

## 🛠️ Step-by-Step: Adding a New Check

### 1. Implement Detection Logic in Modular Subfile
Create or edit the relevant submodule in `src-tauri/src/scanner/`:
```rust
use crate::models::{Finding, Severity};

pub fn check_custom_rule(headers: &reqwest::header::HeaderMap) -> Option<Finding> {
    if !headers.contains_key("x-permitted-cross-domain-policies") {
        return Some(Finding {
            id: "missing-cross-domain-policy".to_string(),
            title: "Missing X-Permitted-Cross-Domain-Policies Header".to_string(),
            severity: Severity::Low,
            category: "security_headers".to_string(),
            owasp_category: "A05:2021-Security Misconfiguration".to_string(),
            description: "The application does not restrict Adobe Flash/PDF cross-domain policy access.".to_string(),
            impact: "Legacy plugins may execute cross-domain data access if policies are permissive.".to_string(),
            remediation: "Add 'X-Permitted-Cross-Domain-Policies: none' to HTTP response headers.".to_string(),
            evidence: None,
            references: vec![
                "https://owasp.org/www-project-secure-headers/#x-permitted-cross-domain-policies".to_string(),
            ],
            cve_id: None,
        });
    }
    None
}
```

### 2. Register in Scanner Orchestrator
Wire the new check inside [`src-tauri/src/scanner/mod.rs`](file:///c:/Users/user/Documents/GitHub/vuln-radar/src-tauri/src/scanner/mod.rs) within `run_scan`.

### 3. Update TypeScript Definitions (If adding new Category)
If you add a new category enum variant, ensure [`src/lib/types.ts`](file:///c:/Users/user/Documents/GitHub/vuln-radar/src/lib/types.ts) includes it in `Category`.

---

## 🔒 Passive Safety Rules

1. **Never send intrusive active exploit payloads** (e.g., SQLi `' OR 1=1`, SSTI `{{7*7}}`, or path traversal `../../`).
2. **Handle network failures gracefully** with configurable timeouts (default 15s).
3. **Always sanitize and truncate evidence** to avoid memory leaks or binary crashes.

---

## ✅ Verification Checklist

After adding a security check:
```bash
# 1. Verify Rust compilation
cargo check --manifest-path src-tauri/Cargo.toml

# 2. Run unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# 3. Check frontend types
npm run check
```
