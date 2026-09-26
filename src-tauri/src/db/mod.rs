use crate::models::{MonitorTarget, ScanReport, ScanSummary, WordlistRecord};
use chrono::{Duration as ChronoDuration, Utc};
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // High-performance, low-memory SQLite tuning: WAL mode, 2MB cache, incremental vacuum
        let _ = conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -2000;
             PRAGMA busy_timeout = 5000;
             PRAGMA auto_vacuum = INCREMENTAL;",
        );

        conn.execute(
            "CREATE TABLE IF NOT EXISTS scans (
                id TEXT PRIMARY KEY,
                target_url TEXT NOT NULL,
                scanned_at TEXT NOT NULL,
                status_code INTEGER NOT NULL,
                security_score INTEGER NOT NULL,
                total_findings INTEGER NOT NULL,
                critical_count INTEGER NOT NULL,
                high_count INTEGER NOT NULL,
                medium_count INTEGER NOT NULL,
                low_count INTEGER NOT NULL,
                info_count INTEGER NOT NULL,
                report_json TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_scans_scanned_at ON scans (scanned_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS monitors (
                id TEXT PRIMARY KEY,
                target_url TEXT NOT NULL,
                interval_hours INTEGER NOT NULL,
                last_scanned_at TEXT,
                next_scan_at TEXT NOT NULL,
                last_score INTEGER,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_monitors_due ON monitors (is_active, next_scan_at)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS wordlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT NOT NULL,
                paths_json TEXT NOT NULL,
                item_count INTEGER NOT NULL,
                is_custom INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Seed predefined wordlists if table is empty
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM wordlists", [], |r| r.get(0)).unwrap_or(0);
        if count == 0 {
            let defaults = crate::wordlists::get_predefined_records();
            for rec in defaults {
                let paths_json = serde_json::to_string(&rec.paths).unwrap_or_else(|_| "[]".to_string());
                let _ = conn.execute(
                    "INSERT INTO wordlists (id, name, category, description, paths_json, item_count, is_custom, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        rec.id,
                        rec.name,
                        rec.category,
                        rec.description,
                        paths_json,
                        rec.item_count as i64,
                        if rec.is_custom { 1 } else { 0 },
                        rec.created_at,
                        rec.updated_at
                    ],
                );
            }
        }

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn save_scan(&self, report: &ScanReport) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let report_json = serde_json::to_string(report).unwrap_or_default();

        conn.execute(
            "INSERT OR REPLACE INTO scans (
                id, target_url, scanned_at, status_code, security_score,
                total_findings, critical_count, high_count, medium_count, low_count, info_count, report_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                report.id,
                report.target_url,
                report.scanned_at,
                report.status_code,
                report.security_score,
                report.total_findings,
                report.critical_count,
                report.high_count,
                report.medium_count,
                report.low_count,
                report.info_count,
                report_json
            ],
        )?;

        Ok(())
    }

    pub fn get_history(&self) -> Result<Vec<ScanSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, target_url, scanned_at, status_code, security_score,
                    total_findings, critical_count, high_count, medium_count, low_count, info_count
             FROM scans ORDER BY scanned_at DESC LIMIT 50",
        )?;

        let history: Vec<ScanSummary> = stmt.query_map([], |row| {
            Ok(ScanSummary {
                id: row.get(0)?,
                target_url: row.get(1)?,
                scanned_at: row.get(2)?,
                status_code: row.get(3)?,
                security_score: row.get(4)?,
                total_findings: row.get(5)?,
                critical_count: row.get(6)?,
                high_count: row.get(7)?,
                medium_count: row.get(8)?,
                low_count: row.get(9)?,
                info_count: row.get(10)?,
            })
        })?.flatten().collect();
        Ok(history)
    }

    pub fn get_scan_report(&self, scan_id: &str) -> Result<Option<ScanReport>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT report_json FROM scans WHERE id = ?1")?;
        let json_str: Option<String> = stmt.query_row(params![scan_id], |r| r.get(0)).ok();
        Ok(json_str.and_then(|s| serde_json::from_str(&s).ok()))
    }

    pub fn delete_scan(&self, scan_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM scans WHERE id = ?1", params![scan_id])?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM scans", [])?;
        Ok(())
    }

    // Monitoring Operations
    pub fn add_monitor(&self, target_url: &str, interval_hours: u32) -> Result<MonitorTarget> {
        let conn = self.conn.lock().unwrap();
        let id = format!("mon_{}", Utc::now().timestamp_millis());
        let now = Utc::now();
        let next_scan = now + ChronoDuration::hours(interval_hours as i64);
        let created_at = now.to_rfc3339();
        let next_scan_at = next_scan.to_rfc3339();

        conn.execute(
            "INSERT INTO monitors (id, target_url, interval_hours, last_scanned_at, next_scan_at, last_score, is_active, created_at)
             VALUES (?1, ?2, ?3, NULL, ?4, NULL, 1, ?5)",
            params![id, target_url, interval_hours, next_scan_at, created_at],
        )?;

        Ok(MonitorTarget {
            id,
            target_url: target_url.to_string(),
            interval_hours,
            last_scanned_at: None,
            next_scan_at,
            last_score: None,
            is_active: true,
            created_at,
        })
    }

    pub fn get_monitors(&self) -> Result<Vec<MonitorTarget>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, target_url, interval_hours, last_scanned_at, next_scan_at, last_score, is_active, created_at
             FROM monitors ORDER BY created_at DESC",
        )?;
        let monitors: Vec<MonitorTarget> = stmt.query_map([], map_monitor_row)?.flatten().collect();
        Ok(monitors)
    }

    pub fn delete_monitor(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM monitors WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn toggle_monitor(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE monitors SET is_active = CASE WHEN is_active = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        let active_int: i32 = conn.query_row("SELECT is_active FROM monitors WHERE id = ?1", params![id], |r| r.get(0)).unwrap_or(0);
        Ok(active_int == 1)
    }

    pub fn update_monitor_scan(&self, id: &str, last_scanned_at: &str, next_scan_at: &str, last_score: u32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE monitors SET last_scanned_at = ?1, next_scan_at = ?2, last_score = ?3 WHERE id = ?4",
            params![last_scanned_at, next_scan_at, last_score, id],
        )?;
        Ok(())
    }

    pub fn get_due_monitors(&self, now_iso: &str) -> Result<Vec<MonitorTarget>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, target_url, interval_hours, last_scanned_at, next_scan_at, last_score, is_active, created_at
             FROM monitors WHERE is_active = 1 AND next_scan_at <= ?1",
        )?;
        let monitors: Vec<MonitorTarget> = stmt.query_map(params![now_iso], map_monitor_row)?.flatten().collect();
        Ok(monitors)
    }

    pub fn save_wordlist(&self, record: &WordlistRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let paths_json = serde_json::to_string(&record.paths).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT OR REPLACE INTO wordlists (
                id, name, category, description, paths_json, item_count, is_custom, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.id,
                record.name,
                record.category,
                record.description,
                paths_json,
                record.item_count as i64,
                if record.is_custom { 1 } else { 0 },
                record.created_at,
                record.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_wordlists(&self) -> Result<Vec<WordlistRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, category, description, paths_json, item_count, is_custom, created_at, updated_at
             FROM wordlists ORDER BY is_custom ASC, id ASC",
        )?;
        let list: Vec<WordlistRecord> = stmt.query_map([], map_wordlist_row)?.flatten().collect();
        Ok(list)
    }

    pub fn get_wordlist_by_id(&self, id: &str) -> Result<Option<WordlistRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, category, description, paths_json, item_count, is_custom, created_at, updated_at
             FROM wordlists WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], map_wordlist_row)?;
        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_wordlist(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM wordlists WHERE id = ?1", params![id])?;
        Ok(())
    }
}

fn map_monitor_row(row: &rusqlite::Row) -> rusqlite::Result<MonitorTarget> {
    let active_int: i32 = row.get(6)?;
    Ok(MonitorTarget {
        id: row.get(0)?,
        target_url: row.get(1)?,
        interval_hours: row.get(2)?,
        last_scanned_at: row.get(3)?,
        next_scan_at: row.get(4)?,
        last_score: row.get(5)?,
        is_active: active_int == 1,
        created_at: row.get(7)?,
    })
}

fn map_wordlist_row(row: &rusqlite::Row) -> rusqlite::Result<WordlistRecord> {
    let paths_json: String = row.get(4)?;
    let paths: Vec<String> = serde_json::from_str(&paths_json).unwrap_or_default();
    let is_custom_int: i32 = row.get(6)?;

    Ok(WordlistRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(2)?,
        description: row.get(3)?,
        paths,
        item_count: row.get::<_, i64>(5)? as usize,
        is_custom: is_custom_int == 1,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}
