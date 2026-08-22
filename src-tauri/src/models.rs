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
}
