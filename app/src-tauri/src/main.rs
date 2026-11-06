// Prevents an extra console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if !tempa_app_lib::mcp_mode(&args) {
        tempa_app_lib::run()
    }
}
