export type Severity = "critical" | "high" | "medium" | "low" | "info";

export type Category =
  | "security_headers"
  | "cookie_security"
  | "vulnerable_dependency"
  | "information_disclosure"
  | "tls_ssl"
  | "cors_misconfiguration"
  | "insecure_form"
  | "dom_security"
  | "dns_email_security"
  | "endpoint_exposure"
  | "port_exposure"
  | "rce_risk";

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

export type WordlistCategory =
  | "all"
  | "paths"
  | "directories"
  | "files"
  | "api"
  | "backups"
  | "debug"
  | "custom";

export interface WordlistItem {
  id: string;
  name: string;
  category: "paths" | "directories" | "files" | "api" | "backups" | "debug" | "custom";
  description: string;
  paths: string[];
  tags?: string[];
  isCustom?: boolean;
}

export interface WordlistRecord {
  id: string;
  name: string;
  category: string;
  description: string;
  paths: string[];
  item_count: number;
  is_custom: boolean;
  created_at: string;
  updated_at: string;
}

export interface DynamicWordlistParams {
  base_words?: string[];
  directories?: string[];
  extensions?: string[];
  prefixes?: string[];
  include_dotfiles?: boolean;
  include_backups?: boolean;
}

export interface PathProbeResult {
  path: string;
  status: number;
  content_length: number;
  content_type: string;
  response_time_ms: number;
  has_content: boolean;
  is_found: boolean;
  body?: string | null;
}

export interface WordlistConfig {
  selectedIds: string[];
  customPaths: string[];
  activePreset: string;
}

export interface ScanOptions {
  custom_headers?: [string, string][];
  user_agent?: string;
  timeout_seconds?: number;
  include_subdomains?: boolean;
  enable_port_scan?: boolean;
  port_scan_profile?: "top20" | "top100" | "databases" | "custom" | string;
  custom_ports?: string;
  port_timeout_ms?: number;
  wordlist_config?: WordlistConfig;
}

export interface OpenPort {
  port: number;
  protocol: string;
  service: string;
  state: string;
  banner?: string | null;
  is_risky: boolean;
  description: string;
}

export interface PortScanReport {
  host: string;
  ip_address?: string | null;
  scanned_ports_count: number;
  open_ports_count: number;
  open_ports: OpenPort[];
  scan_duration_ms: number;
}

export interface DnsSecurityReport {
  domain: string;
  spf_record?: string | null;
  spf_valid: boolean;
  dmarc_record?: string | null;
  dmarc_valid: boolean;
  dmarc_policy?: string | null;
  dnssec_enabled: boolean;
}

export interface EndpointReport {
  robots_txt_found: boolean;
  disallowed_paths: string[];
  sensitive_disallowed_paths: string[];
  security_txt_found: boolean;
  security_txt_content?: string | null;
  exposed_paths_count?: number;
  scanned_paths_count?: number;
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
  subdomains?: string[];
  dns_security?: DnsSecurityReport | null;
  endpoint_report?: EndpointReport | null;
  port_report?: PortScanReport | null;
}

export interface MonitorTarget {
  id: string;
  target_url: string;
  interval_hours: number;
  last_scanned_at?: string | null;
  next_scan_at: string;
  last_score?: number | null;
  is_active: boolean;
  created_at: string;
}

export interface BatchScanItem {
  url: string;
  status: "pending" | "scanning" | "completed" | "failed";
  report?: ScanReport | null;
  error?: string | null;
}

export type SwissTheme = "swiss-dark" | "swiss-light";

export interface ThemeState {
  current: SwissTheme;
}

