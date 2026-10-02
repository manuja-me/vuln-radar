#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod db;
pub mod models;
pub mod scanner;
pub mod server;
pub mod wordlists;

use chrono::{Duration as ChronoDuration, Utc};
use db::Database;
use models::{
    BatchScanItem, DynamicWordlistParams, MonitorTarget, PathProbeResult, ScanOptions, ScanReport,
    ScanSummary, WordlistRecord,
};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager, State};

pub struct AppState {
    pub db: Arc<Database>,
    pub server_port: u16,
}

#[tauri::command]
async fn analyze_paths(
    target_url: String,
    paths: Vec<String>,
    timeout_seconds: Option<u64>,
    concurrency: Option<usize>,
) -> Result<Vec<PathProbeResult>, String> {
    let timeout = Duration::from_secs(timeout_seconds.unwrap_or(8));
    let concurrency_limit = concurrency.unwrap_or(20).clamp(1, 50);

    let client = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::limited(3))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let trimmed = target_url.trim();
    if trimmed.is_empty() {
        return Err("Target URL cannot be empty.".to_string());
    }
    let had_explicit_scheme = trimmed.starts_with("http://") || trimmed.starts_with("https://");
    let candidate_url = if !had_explicit_scheme {
        let lower = trimmed.to_lowercase();
        let scheme = match lower.starts_with("localhost") || lower.starts_with("127.0.0.1") || lower.contains(':') {
            true => "http",
            false => "https",
        };
        format!("{}://{}", scheme, trimmed)
    } else {
        trimmed.to_string()
    };

    let base_url = crate::server::sanitize_url(&candidate_url)?;
    let sanitized_paths = crate::server::sanitize_word_list(&paths)?;

    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency_limit));
    let mut tasks = Vec::new();

    for clean_path in sanitized_paths {
        let sem = semaphore.clone();
        let client_clone = client.clone();
        let mut target_probe_url = base_url.clone();
        let base_prefix = target_probe_url.path().trim_end_matches('/');
        let combined_path = format!("{}/{}", base_prefix, clean_path.trim_start_matches('/'));
        target_probe_url.set_path(&combined_path);

        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.ok();
            let start = std::time::Instant::now();
            let res = client_clone.get(target_probe_url).send().await;
            let elapsed = start.elapsed().as_millis() as u64;

            match res {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let content_type = resp
                        .headers()
                        .get(reqwest::header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();
                    let body_bytes = resp.bytes().await.unwrap_or_default();
                    let content_len = body_bytes.len();
                    let is_found = status != 404 && status != 0;
                    let has_content = content_len > 0 && is_found;
                    let body = if has_content {
                        let max_len = 65536.min(content_len);
                        Some(String::from_utf8_lossy(&body_bytes[..max_len]).to_string())
                    } else {
                        None
                    };

                    Some(PathProbeResult {
                        path: clean_path,
                        status,
                        content_length: content_len,
                        content_type,
                        response_time_ms: elapsed,
                        has_content,
                        is_found,
                        body,
                    })
                }
                Err(_) => None,
            }
        }));
    }

    let mut results = Vec::new();
    for t in tasks {
        if let Ok(Some(item)) = t.await {
            results.push(item);
        }
    }

    Ok(results)
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
    let prof = match profile {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => if custom_ports.is_some() { "custom".to_string() } else { "top20".to_string() },
    };
    let (report, _) = scanner::ports::audit_ports(&host, &prof, custom_ports.as_deref(), timeout_ms).await;
    if report.ip_address.is_none() && report.scanned_ports_count == 0 {
        return Err(format!("Could not resolve network host '{}'. Verify the domain/IP and active connectivity.", host));
    }
    Ok(report)
}

#[tauri::command]
async fn query_dns(domain: String) -> Result<models::DnsSecurityReport, String> {
    let clean = domain
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or(&domain)
        .split(':')
        .next()
        .unwrap_or(&domain)
        .to_string();

    if clean.is_empty() {
        return Err("Target domain cannot be empty.".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let (report, _) = scanner::dns::audit_dns_and_email_security(&client, &clean).await;
    Ok(report)
}

#[tauri::command]
fn get_wordlists(state: State<'_, AppState>) -> Result<Vec<WordlistRecord>, String> {
    state.db.get_wordlists().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_wordlist(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    mut item: WordlistRecord,
) -> Result<WordlistRecord, String> {
    let now = Utc::now().to_rfc3339();
    if item.created_at.is_empty() {
        item.created_at = now.clone();
    }
    item.updated_at = now;

    // Sanitize & normalize paths
    let mut unique_paths = std::collections::BTreeSet::new();
    for p in item.paths {
        let trimmed = p.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let norm = format!("/{}", trimmed.trim_start_matches('/'));
            unique_paths.insert(norm);
        }
    }
    item.paths = unique_paths.into_iter().collect();
    item.item_count = item.paths.len();

    // 1. Store in SQLite Database
    state.db.save_wordlist(&item).map_err(|e| e.to_string())?;

    // 2. Store on File System (.txt)
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        if let Ok(wl_dir) = wordlists::ensure_wordlist_dir(&app_data_dir) {
            let _ = wordlists::save_to_filesystem(&wl_dir, &item.id, &item.paths);
        }
    }

    Ok(item)
}

#[tauri::command]
fn delete_wordlist(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // 1. Delete from SQLite Database
    state.db.delete_wordlist(&id).map_err(|e| e.to_string())?;

    // 2. Delete from File System
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let wl_dir = app_data_dir.join("wordlists");
        let _ = wordlists::delete_from_filesystem(&wl_dir, &id);
    }

    Ok(())
}

#[tauri::command]
fn generate_dynamic_wordlist(params: DynamicWordlistParams) -> Result<Vec<String>, String> {
    Ok(wordlists::generate_dynamic_wordlist(&params))
}

#[tauri::command]
fn export_wordlist_file(
    app: tauri::AppHandle,
    filename: String,
    paths: Vec<String>,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let wl_dir = wordlists::ensure_wordlist_dir(&app_data_dir).map_err(|e| e.to_string())?;

    // Extract filename component only, preventing any path traversal outside wl_dir
    let base_name = std::path::Path::new(&filename)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("wordlist.txt");

    let clean_stem: String = base_name
        .trim_end_matches(".txt")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect();

    let safe_filename = if clean_stem.is_empty() {
        "wordlist.txt".to_string()
    } else {
        format!("{}.txt", clean_stem)
    };

    let file_path = wl_dir.join(&safe_filename);
    let content = paths.join("\n");
    std::fs::write(&file_path, content).map_err(|e| e.to_string())?;
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_server_port(state: State<'_, AppState>) -> u16 {
    state.server_port
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
        let refs = match f.references.is_empty() {
            true => String::new(),
            false => format!("**References**:\n{}\n\n", f.references.iter().map(|r| format!("- {}", r)).collect::<Vec<_>>().join("\n")),
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

#[tauri::command]
async fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_maximize_window(window: tauri::Window) -> Result<bool, String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
async fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
async fn hide_to_tray(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
async fn is_window_maximized(window: tauri::Window) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap());
            std::fs::create_dir_all(&app_data_dir).unwrap_or_default();
            let db_path = app_data_dir.join("vuln_radar.db");
            let database = Arc::new(Database::new(db_path).expect("Failed to initialize SQLite database"));

            let port = tauri::async_runtime::block_on(async {
                server::start_wordlist_http_server(database.clone()).await
            });

            app.manage(AppState {
                db: database.clone(),
                server_port: port,
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
                                    let alert_payload = serde_json::json!({
                                        "target_url": target.target_url,
                                        "new_score": report.security_score,
                                        "previous_score": previous_score,
                                        "critical_count": report.critical_count,
                                    });
                                    let _ = app_handle.emit("monitor_alert", &alert_payload);
                                    let _ = app_handle.emit("watchdog_alert", &alert_payload);
                                }
                            }
                        }
                    }
                }
            });

            // Sync wordlists to filesystem directory app_data_dir/wordlists/
            if let Ok(wl_dir) = wordlists::ensure_wordlist_dir(&app_data_dir) {
                if let Ok(all_wordlists) = database.get_wordlists() {
                    for wl in all_wordlists {
                        let _ = wordlists::save_to_filesystem(&wl_dir, &wl.id, &wl.paths);
                    }
                }
            }

            // Setup System Tray
            let show_i = tauri::menu::MenuItem::with_id(app, "show", "Open VulnRadar", true, None::<&str>)?;
            let hide_i = tauri::menu::MenuItem::with_id(app, "hide", "Hide to Tray", true, None::<&str>)?;
            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "Quit VulnRadar", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

            if let Some(tray_icon) = app.default_window_icon().cloned() {
                let _tray = tauri::tray::TrayIconBuilder::new()
                    .icon(tray_icon)
                    .tooltip("VulnRadar - Web Security Scanner")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.hide();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let tauri::tray::TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Left,
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(w) = app.get_webview_window("main") {
                                if w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false) {
                                    let _ = w.hide();
                                } else {
                                    let _ = w.show();
                                    let _ = w.unminimize();
                                    let _ = w.set_focus();
                                }
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            analyze_paths,
            scan_target,
            scan_batch,
            scan_ports,
            query_dns,
            get_history,
            get_scan_report,
            delete_scan,
            clear_history,
            get_monitors,
            add_monitor,
            delete_monitor,
            toggle_monitor,
            export_report_markdown,
            get_wordlists,
            save_wordlist,
            delete_wordlist,
            generate_dynamic_wordlist,
            export_wordlist_file,
            get_server_port,
            minimize_window,
            toggle_maximize_window,
            close_window,
            hide_to_tray,
            is_window_maximized
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
