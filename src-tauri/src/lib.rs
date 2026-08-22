pub mod db;
pub mod models;
pub mod scanner;

use db::Database;
use models::{ScanReport, ScanSummary};
use std::sync::Arc;
use tauri::{Manager, State};

pub struct AppState {
    pub db: Arc<Database>,
}

#[tauri::command]
async fn scan_target(state: State<'_, AppState>, url: String) -> Result<ScanReport, String> {
    let report = scanner::run_scan(&url).await?;
    let _ = state.db.save_scan(&report);
    Ok(report)
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
            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
            std::fs::create_dir_all(&app_data_dir).unwrap_or_default();
            let db_path = app_data_dir.join("vuln_radar.db");
            let database = Database::new(db_path).expect("Failed to initialize SQLite database");
            app.manage(AppState {
                db: Arc::new(database),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_target,
            get_history,
            get_scan_report,
            delete_scan,
            clear_history,
            export_report_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
