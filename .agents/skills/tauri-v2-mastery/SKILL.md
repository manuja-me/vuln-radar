---
name: tauri-v2-mastery
description: >-
  Architectural patterns, IPC commands, SQLite persistence, and async runtime best practices for Tauri v2 and Rust in VulnRadar.
  Use this skill whenever authoring Tauri IPC commands, managing background Tokio threads, configuring SQLite databases, or handling Tauri events.
---

# ⚡ Tauri v2 & Rust Architecture Guide for VulnRadar

This skill documents critical patterns, runtime safety guidelines, and IPC standards for VulnRadar's Tauri v2 desktop core.

---

## 🧵 Async Runtime & Background Workers

### Critical Rule: Use `tauri::async_runtime::spawn`
Tauri v2 manages its own internal Tokio reactor. In synchronous setup hooks (such as `tauri::Builder::setup`), **never** use `tokio::spawn` directly without an active Tokio runtime, as this triggers:
> `panicked: there is no reactor running, must be called from the context of a Tokio 1.x runtime`

**Correct Pattern**:
```rust
tauri::Builder::default()
    .setup(|app| {
        let app_handle = app.handle().clone();
        let database = Arc::new(Database::new(db_path)?);

        // ALWAYS use tauri::async_runtime::spawn in setup hooks
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                // Periodic task logic
            }
        });

        Ok(())
    })
```

---

## 🔌 IPC Command Handlers Standard

1. **Keep Handlers Non-Blocking**: Always mark commands as `async` if they perform HTTP requests or database operations.
2. **State Injection**: Access shared databases and clients via `tauri::State<'_, AppState>`.
3. **Structured Errors**: Return `Result<T, String>` so frontend catches structured error strings.

```rust
#[tauri::command]
pub async fn scan_target(
    url: String,
    options: Option<ScanOptions>,
    state: tauri::State<'_, AppState>,
) -> Result<ScanReport, String> {
    let report = scanner::run_scan(&url, options)
        .await
        .map_err(|e| format!("Scan failed: {}", e))?;

    let _ = state.db.save_scan(&report);
    Ok(report)
}
```

---

## 💾 Embedded SQLite (`rusqlite`) Best Practices

1. **Enable WAL Mode**: Always execute `PRAGMA journal_mode=WAL;` and `PRAGMA synchronous=NORMAL;` upon connection initialization to enable non-blocking concurrent reads during background scans.
2. **Mutex Wrapping**: Wrap `rusqlite::Connection` in `Arc<Mutex<Connection>>` to safely share across Tauri command threads and background workers.
3. **Database Location**: Always resolve SQLite database paths relative to `app.path().app_data_dir()`, never relative to the current working directory.

---

## 📡 Frontend-Backend Event Emission

To send real-time alerts or progress updates from Rust to Svelte:
```rust
// Rust Backend
app_handle.emit("monitor_alert", serde_json::json!({
    "target_url": url,
    "new_score": score
})).map_err(|e| e.to_string())?;
```

```typescript
// Svelte 5 Frontend
import { listen } from "@tauri-apps/api/event";

onMount(() => {
  let unlisten: (() => void) | undefined;
  (async () => {
    unlisten = await listen<{ target_url: string; new_score: number }>(
      "monitor_alert",
      (event) => {
        console.log("Alert received:", event.payload);
      }
    );
  })();
  return () => { if (unlisten) unlisten(); };
});
```

---

## ✅ Quality Checklist

- [ ] Check Rust builds: `cargo check --manifest-path src-tauri/Cargo.toml`
- [ ] Run clippy linter: `cargo clippy --manifest-path src-tauri/Cargo.toml`
- [ ] Ensure all IPC commands are registered in `tauri::generate_handler![...]`.
