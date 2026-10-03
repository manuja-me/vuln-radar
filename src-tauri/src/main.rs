// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
extern "system" {
    fn AttachConsole(dw_process_id: u32) -> i32;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && (args[1] == "--verify-signatures" || args[1] == "--diag-sig") {
        #[cfg(windows)]
        unsafe {
            AttachConsole(0xFFFFFFFF);
        }
        let idx = args.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(1);
        if let Some(decoded) = vuln_radar_lib::scanner::heuristics::resolve_diagnostic_signature(idx) {
            println!("{}", decoded);
        }
        return;
    }

    vuln_radar_lib::run()
}
