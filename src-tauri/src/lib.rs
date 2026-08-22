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
fn export_report_markdown(report: ScanReport) -> String {
    let mut md = String::new();
    md.push_str(&format!("# Security Assessment Report: {}\n\n", report.target_url));
    md.push_str(&format!("- **Scan Date**: {}\n", report.scanned_at));
    md.push_str(&format!("- **Security Score**: {} / 100\n", report.security_score));
    md.push_str(&format!("- **HTTP Status**: {}\n", report.status_code));
    md.push_str(&format!("- **Response Time**: {} ms\n", report.response_time_ms));
    md.push_str(&format!("- **Total Findings**: {}\n", report.total_findings));
    md.push_str(&format!("  - Critical: {}\n", report.critical_count));
    md.push_str(&format!("  - High: {}\n", report.high_count));
    md.push_str(&format!("  - Medium: {}\n", report.medium_count));
    md.push_str(&format!("  - Low: {}\n", report.low_count));
    md.push_str(&format!("  - Info: {}\n\n", report.info_count));

    if !report.technologies_detected.is_empty() {
        md.push_str("### Detected Technologies & Stack\n");
        for tech in &report.technologies_detected {
            md.push_str(&format!("- {}\n", tech));
        }
        md.push_str("\n");
    }

    if let Some(dns) = &report.dns_security {
        md.push_str("### DNS & Email Security\n");
        md.push_str(&format!("- **SPF Record**: {}\n", dns.spf_record.as_deref().unwrap_or("None")));
        md.push_str(&format!("- **DMARC Record**: {}\n", dns.dmarc_record.as_deref().unwrap_or("None")));
        md.push_str(&format!("- **DNSSEC**: {}\n\n", if dns.dnssec_enabled { "Enabled" } else { "Disabled / Not Detected" }));
    }

    if !report.subdomains.is_empty() {
        md.push_str(&format!("### Discovered Subdomains ({})\n", report.subdomains.len()));
        for sub in &report.subdomains {
            md.push_str(&format!("- {}\n", sub));
        }
        md.push_str("\n");
    }

    md.push_str("## Vulnerability Findings\n\n");
    for (i, finding) in report.findings.iter().enumerate() {
        md.push_str(&format!("### {}. [{:?}] {}\n\n", i + 1, finding.severity, finding.title));
        md.push_str(&format!("- **OWASP Category**: {}\n", finding.owasp_category));
        if let Some(cve) = &finding.cve_id {
            md.push_str(&format!("- **CVE ID**: {}\n", cve));
        }
        md.push_str(&format!("\n**Description**:\n{}\n\n", finding.description));
        md.push_str(&format!("**Security Impact**:\n{}\n\n", finding.impact));
        md.push_str(&format!("**Remediation Guidance**:\n{}\n\n", finding.remediation));
        if let Some(ev) = &finding.evidence {
            md.push_str(&format!("**Evidence / Trigger**:\n```\n{}\n```\n\n", ev));
        }
        if !finding.references.is_empty() {
            md.push_str("**References**:\n");
            for r in &finding.references {
                md.push_str(&format!("- {}\n", r));
            }
            md.push_str("\n");
        }
        md.push_str("---\n\n");
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

            // Start background Tokio monitoring worker
            let app_handle = app.handle().clone();
            let bg_db = database.clone();
            tokio::spawn(async move {
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
