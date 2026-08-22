use crate::models::{ScanReport, ScanSummary};
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
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

        let rows = stmt.query_map([], |row| {
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
        })?;

        let mut history = Vec::new();
        for row in rows {
            if let Ok(item) = row {
                history.push(item);
            }
        }
        Ok(history)
    }

    pub fn get_scan_report(&self, scan_id: &str) -> Result<Option<ScanReport>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT report_json FROM scans WHERE id = ?1")?;
        let mut rows = stmt.query(params![scan_id])?;

        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            if let Ok(report) = serde_json::from_str::<ScanReport>(&json_str) {
                return Ok(Some(report));
            }
        }

        Ok(None)
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
}
