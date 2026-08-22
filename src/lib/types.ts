export type Severity = "critical" | "high" | "medium" | "low" | "info";

export type Category =
  | "security_headers"
  | "cookie_security"
  | "vulnerable_dependency"
  | "information_disclosure"
  | "tls_ssl"
  | "cors_misconfiguration"
  | "insecure_form"
  | "dom_security";

export interface Finding {
  id: string;
  title: string;
  severity: Severity;
  category: Category;
  description: string;
  impact: string;
  remediation: string;
  evidence?: string | null;
  owasp_category: string;
  cve_id?: string | null;
  references: string[];
}

export interface ScanSummary {
  id: string;
  target_url: string;
  scanned_at: string;
  status_code: number;
  security_score: number;
  total_findings: number;
  critical_count: number;
  high_count: number;
  medium_count: number;
  low_count: number;
  info_count: number;
}

export interface ScanReport {
  id: string;
  target_url: string;
  scanned_at: string;
  status_code: number;
  response_time_ms: number;
  security_score: number;
  total_findings: number;
  critical_count: number;
  high_count: number;
  medium_count: number;
  low_count: number;
  info_count: number;
  findings: Finding[];
  server_info?: string | null;
  technologies_detected: string[];
  response_headers: [string, string][];
}
