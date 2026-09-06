pub mod db;
pub mod models;
pub mod scanner;

use chrono::{Duration as ChronoDuration, Utc};
use db::Database;
use models::{BatchScanItem, MonitorTarget, ScanOptions, ScanReport, ScanSummary};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager, State};

pub struct AppState {
    pub db: Arc<Database>,
}

#[tauri::command]
async fn scan_target(
    state: State<'_, AppState>,
    url: String,
    options: Option<ScanOptions>,
) -> Result<ScanReport, String> {
    let report = scanner::run_scan(&url, options).await?;
    let _ = state.db.save_scan(&report);
    Ok(report)
}

#[tauri::command]
async fn scan_batch(
    state: State<'_, AppState>,
    urls: Vec<String>,
    options: Option<ScanOptions>,
) -> Result<Vec<BatchScanItem>, String> {
    let mut results = Vec::new();

    for raw_url in urls {
        let url = raw_url.trim().to_string();
        if url.is_empty() {
            continue;
        }

        match scanner::run_scan(&url, options.clone()).await {
            Ok(report) => {
                let _ = state.db.save_scan(&report);
                results.push(BatchScanItem {
                    url,
                    status: "completed".to_string(),
                    report: Some(report),
                    error: None,
                });
            }
            Err(e) => {
                results.push(BatchScanItem {
                    url,
                    status: "failed".to_string(),
                    report: None,
                    error: Some(e),
                });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
fn get_history(state: State<'_, AppState>) -> Result<Vec<ScanSummary>, String> {
    state.db.get_history().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scan_report(state: State<'_, AppState>, id: String) -> Result<Option<ScanReport>, String> {
    state.db.get_scan_report(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_scan(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_scan(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    state.db.clear_all().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_monitors(state: State<'_, AppState>) -> Result<Vec<MonitorTarget>, String> {
    state.db.get_monitors().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_monitor(
    state: State<'_, AppState>,
    url: String,
    interval_hours: u32,
) -> Result<MonitorTarget, String> {
    state.db.add_monitor(&url, interval_hours).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_monitor(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.db.delete_monitor(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_monitor(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    state.db.toggle_monitor(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_ports(
    host: String,
    profile: Option<String>,
    custom_ports: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<models::PortScanReport, String> {
    let prof = profile.unwrap_or_else(|| "top20".to_string());
    let (report, _) = scanner::ports::audit_ports(&host, &prof, custom_ports.as_deref(), timeout_ms).await;
    Ok(report)
}

#[tauri::command]
fn export_report_markdown(report: ScanReport) -> String {
    let mut md = format!(
        "# Security Assessment Report: {}\n\n\
        - **Scan Date**: {}\n\
        - **Security Score**: {} / 100\n\
        - **HTTP Status**: {}\n\
        - **Response Time**: {} ms\n\
        - **Total Findings**: {}\n\
          - Critical: {}\n\
          - High: {}\n\
          - Medium: {}\n\
          - Low: {}\n\
          - Info: {}\n\n",
        report.target_url, report.scanned_at, report.security_score,
        report.status_code, report.response_time_ms, report.total_findings,
        report.critical_count, report.high_count, report.medium_count,
        report.low_count, report.info_count
    );

    if !report.technologies_detected.is_empty() {
        md.push_str("### Detected Technologies & Stack\n");
        for tech in &report.technologies_detected {
            md.push_str(&format!("- {}\n", tech));
        }
        md.push('\n');
    }

    if let Some(port_rep) = &report.port_report {
        let ip_line = port_rep.ip_address.as_deref().map(|ip| format!("- **Resolved IP**: {}\n", ip)).unwrap_or_default();
        md.push_str(&format!(
            "### Discovered Open Ports & Services\n\
            {}\
            - **Ports Scanned**: {}\n\
            - **Open Ports Found**: {}\n\
            - **Scan Duration**: {} ms\n\n",
            ip_line, port_rep.scanned_ports_count, port_rep.open_ports_count, port_rep.scan_duration_ms
        ));

        if !port_rep.open_ports.is_empty() {
            md.push_str("| Port | Protocol | Service | State | Risk | Banner / Details |\n| --- | --- | --- | --- | --- | --- |\n");
            for p in &port_rep.open_ports {
                let risk = if p.is_risky { "⚠️ EXPOSED / RISKY" } else { "STANDARD" };
                let banner = p.banner.as_deref().unwrap_or(p.description.as_str());
                md.push_str(&format!("| {} | {} | {} | {} | {} | {} |\n", p.port, p.protocol.to_uppercase(), p.service, p.state.to_uppercase(), risk, banner));
            }
            md.push('\n');
        }
    }

    if let Some(dns) = &report.dns_security {
        let dnssec = if dns.dnssec_enabled { "Enabled" } else { "Disabled / Not Detected" };
        md.push_str(&format!(
            "### DNS & Email Security\n\
            - **SPF Record**: {}\n\
            - **DMARC Record**: {}\n\
            - **DNSSEC**: {}\n\n",
            dns.spf_record.as_deref().unwrap_or("None"),
            dns.dmarc_record.as_deref().unwrap_or("None"),
            dnssec
        ));
    }

    if !report.subdomains.is_empty() {
        md.push_str(&format!("### Discovered Subdomains ({})\n", report.subdomains.len()));
        for sub in &report.subdomains {
            md.push_str(&format!("- {}\n", sub));
        }
        md.push('\n');
    }

    md.push_str("## Vulnerability Findings\n\n");
    for (i, f) in report.findings.iter().enumerate() {
        let cve_line = f.cve_id.as_deref().map(|c| format!("- **CVE ID**: {}\n", c)).unwrap_or_default();
        let ev_block = f.evidence.as_deref().map(|e| format!("**Evidence / Trigger**:\n```\n{}\n```\n\n", e)).unwrap_or_default();
        let refs = if f.references.is_empty() {
            String::new()
        } else {
            format!("**References**:\n{}\n\n", f.references.iter().map(|r| format!("- {}", r)).collect::<Vec<_>>().join("\n"))
        };

        md.push_str(&format!(
            "### {}. [{:?}] {}\n\n\
            - **OWASP Category**: {}\n\
            {}\
            \n**Description**:\n{}\n\n\
            **Security Impact**:\n{}\n\n\
            **Remediation Guidance**:\n{}\n\n\
            {}{}\
            ---\n\n",
            i + 1, f.severity, f.title, f.owasp_category, cve_line, f.description, f.impact, f.remediation, ev_block, refs
        ));
    }

    md
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap());
            std::fs::create_dir_all(&app_data_dir).unwrap_or_default();
            let db_path = app_data_dir.join("vuln_radar.db");
            let database = Arc::new(Database::new(db_path).expect("Failed to initialize SQLite database"));

            app.manage(AppState {
                db: database.clone(),
            });

            // Start background monitoring worker using Tauri async runtime
            let app_handle = app.handle().clone();
            let bg_db = database.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    let now = Utc::now();
                    let now_iso = now.to_rfc3339();

                    if let Ok(due_targets) = bg_db.get_due_monitors(&now_iso) {
                        for target in due_targets {
                            if let Ok(report) = scanner::run_scan(&target.target_url, None).await {
                                let _ = bg_db.save_scan(&report);
                                let next_scan = Utc::now() + ChronoDuration::hours(target.interval_hours as i64);
                                let _ = bg_db.update_monitor_scan(
                                    &target.id,
                                    &Utc::now().to_rfc3339(),
                                    &next_scan.to_rfc3339(),
                                    report.security_score,
                                );

                                // If previous score was known and score decreased or critical issues found, emit alert
                                let previous_score = target.last_score.unwrap_or(report.security_score);
                                if report.security_score < previous_score || report.critical_count > 0 {
                                    let _ = app_handle.emit(
                                        "monitor_alert",
                                        serde_json::json!({
                                            "target_url": target.target_url,
                                            "new_score": report.security_score,
                                            "previous_score": previous_score,
                                            "critical_count": report.critical_count,
                                        }),
                                    );
                                }
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_target,
            scan_batch,
            scan_ports,
            get_history,
            get_scan_report,
            delete_scan,
            clear_history,
            get_monitors,
            add_monitor,
            delete_monitor,
            toggle_monitor,
            export_report_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
