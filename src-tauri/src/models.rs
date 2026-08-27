use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
pub struct ScanOptions {
    pub custom_headers: Option<Vec<(String, String)>>,
    pub user_agent: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub include_subdomains: Option<bool>,
    pub enable_port_scan: Option<bool>,
    pub port_scan_profile: Option<String>,
    pub custom_ports: Option<String>,
    pub port_timeout_ms: Option<u64>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EndpointReport {
    pub robots_txt_found: bool,
    pub disallowed_paths: Vec<String>,
    pub sensitive_disallowed_paths: Vec<String>,
    pub security_txt_found: bool,
    pub security_txt_content: Option<String>,
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
