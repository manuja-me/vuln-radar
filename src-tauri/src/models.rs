use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn deduction(&self) -> u32 {
        match self {
            Severity::Critical => 25,
            Severity::High => 15,
            Severity::Medium => 8,
            Severity::Low => 3,
            Severity::Info => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    SecurityHeaders,
    CookieSecurity,
    VulnerableDependency,
    InformationDisclosure,
    TlsSsl,
    CorsMisconfiguration,
    InsecureForm,
    DomSecurity,
    DnsEmailSecurity,
    EndpointExposure,
    PortExposure,
    RceRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub category: Category,
    pub description: String,
    pub impact: String,
    pub remediation: String,
    pub evidence: Option<String>,
    pub owasp_category: String,
    pub cve_id: Option<String>,
    pub references: Vec<String>,
}

impl Finding {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        severity: Severity,
        category: Category,
        description: impl Into<String>,
        impact: impl Into<String>,
        remediation: impl Into<String>,
        owasp_category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            severity,
            category,
            description: description.into(),
            impact: impact.into(),
            remediation: remediation.into(),
            evidence: None,
            owasp_category: owasp_category.into(),
            cve_id: None,
            references: Vec::new(),
        }
    }

    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = Some(evidence.into());
        self
    }

    pub fn with_cve(mut self, cve: impl Into<String>) -> Self {
        self.cve_id = Some(cve.into());
        self
    }

    pub fn with_refs(mut self, refs: &[&str]) -> Self {
        self.references = refs.iter().map(|s| s.to_string()).collect();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSummary {
    pub id: String,
    pub target_url: String,
    pub scanned_at: String,
    pub status_code: u16,
    pub security_score: u32,
    pub total_findings: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WordlistConfig {
    #[serde(default, alias = "selectedIds", alias = "selected_ids")]
    pub selected_ids: Vec<String>,
    #[serde(default, alias = "customPaths", alias = "custom_paths")]
    pub custom_paths: Vec<String>,
    #[serde(default, alias = "activePreset", alias = "active_preset")]
    pub active_preset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanOptions {
    #[serde(default, alias = "customHeaders", alias = "custom_headers")]
    pub custom_headers: Option<Vec<(String, String)>>,
    #[serde(default, alias = "userAgent", alias = "user_agent")]
    pub user_agent: Option<String>,
    #[serde(default, alias = "timeoutSeconds", alias = "timeout_seconds")]
    pub timeout_seconds: Option<u64>,
    #[serde(default, alias = "includeSubdomains", alias = "include_subdomains")]
    pub include_subdomains: Option<bool>,
    #[serde(default, alias = "enablePortScan", alias = "enable_port_scan")]
    pub enable_port_scan: Option<bool>,
    #[serde(default, alias = "portScanProfile", alias = "port_scan_profile")]
    pub port_scan_profile: Option<String>,
    #[serde(default, alias = "customPorts", alias = "custom_ports")]
    pub custom_ports: Option<String>,
    #[serde(default, alias = "portTimeoutMs", alias = "port_timeout_ms")]
    pub port_timeout_ms: Option<u64>,
    #[serde(default, alias = "wordlistConfig", alias = "wordlist_config")]
    pub wordlist_config: Option<WordlistConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub name: String,
    pub ttl: u32,
    pub class: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub data: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPort {
    pub port: u16,
    pub protocol: String,
    pub service: String,
    pub state: String,
    pub banner: Option<String>,
    pub is_risky: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortScanReport {
    pub host: String,
    pub ip_address: Option<String>,
    pub scanned_ports_count: usize,
    pub open_ports_count: usize,
    pub open_ports: Vec<OpenPort>,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsSecurityReport {
    pub domain: String,
    pub spf_record: Option<String>,
    pub spf_valid: bool,
    pub dmarc_record: Option<String>,
    pub dmarc_valid: bool,
    pub dmarc_policy: Option<String>,
    pub dnssec_enabled: bool,
    #[serde(default)]
    pub nameservers: Vec<String>,
    #[serde(default)]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub records: Vec<DnsRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EndpointReport {
    pub robots_txt_found: bool,
    pub disallowed_paths: Vec<String>,
    pub sensitive_disallowed_paths: Vec<String>,
    pub security_txt_found: bool,
    pub security_txt_content: Option<String>,
    #[serde(default)]
    pub exposed_paths_count: usize,
    #[serde(default)]
    pub scanned_paths_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub id: String,
    pub target_url: String,
    pub scanned_at: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub security_score: u32,
    pub total_findings: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub findings: Vec<Finding>,
    pub server_info: Option<String>,
    pub technologies_detected: Vec<String>,
    pub response_headers: Vec<(String, String)>,
    #[serde(default)]
    pub subdomains: Vec<String>,
    #[serde(default)]
    pub dns_security: Option<DnsSecurityReport>,
    #[serde(default)]
    pub endpoint_report: Option<EndpointReport>,
    #[serde(default)]
    pub port_report: Option<PortScanReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorTarget {
    pub id: String,
    pub target_url: String,
    pub interval_hours: u32,
    pub last_scanned_at: Option<String>,
    pub next_scan_at: String,
    pub last_score: Option<u32>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchScanItem {
    pub url: String,
    pub status: String,
    pub report: Option<ScanReport>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordlistRecord {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub paths: Vec<String>,
    #[serde(default, alias = "itemCount", alias = "item_count")]
    pub item_count: usize,
    #[serde(default, alias = "isCustom", alias = "is_custom")]
    pub is_custom: bool,
    #[serde(default, alias = "createdAt", alias = "created_at")]
    pub created_at: String,
    #[serde(default, alias = "updatedAt", alias = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicWordlistParams {
    #[serde(default, alias = "baseWords", alias = "base_words")]
    pub base_words: Vec<String>,
    #[serde(default)]
    pub directories: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub prefixes: Vec<String>,
    #[serde(default, alias = "includeDotfiles", alias = "include_dotfiles")]
    pub include_dotfiles: bool,
    #[serde(default, alias = "includeBackups", alias = "include_backups")]
    pub include_backups: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathProbeResult {
    pub path: String,
    pub status: u16,
    pub content_length: usize,
    pub content_type: String,
    pub response_time_ms: u64,
    pub has_content: bool,
    pub is_found: bool,
    #[serde(default)]
    pub body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wordlist_record_camel_case_deserialization() {
        let json = r#"{
            "id": "wl_123",
            "name": "Custom Admin",
            "category": "custom",
            "description": "Admin test paths",
            "paths": ["/admin", "/login"],
            "itemCount": 2,
            "isCustom": true,
            "createdAt": "2026-09-27T00:00:00Z",
            "updatedAt": "2026-09-27T00:00:00Z"
        }"#;

        let rec: WordlistRecord = serde_json::from_str(json).expect("Failed to deserialize WordlistRecord");
        assert_eq!(rec.id, "wl_123");
        assert_eq!(rec.item_count, 2);
        assert!(rec.is_custom);
        assert_eq!(rec.created_at, "2026-09-27T00:00:00Z");
    }

    #[test]
    fn test_dynamic_wordlist_params_camel_case() {
        let json = r#"{
            "baseWords": ["admin", "api"],
            "directories": ["v1"],
            "extensions": ["json"],
            "includeDotfiles": true,
            "includeBackups": true
        }"#;

        let params: DynamicWordlistParams = serde_json::from_str(json).expect("Failed to deserialize DynamicWordlistParams");
        assert_eq!(params.base_words, vec!["admin", "api"]);
        assert!(params.include_dotfiles);
        assert!(params.include_backups);
    }
}
