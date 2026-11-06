//! tempa desktop shell: terminals per project and the contract panel, on tempa-core.

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_version])
        .run(tauri::generate_context!())
        .expect("tempa app failed to start");
}

#[cfg(test)]
mod tests {
    #[test]
    fn app_version_matches_package() {
        assert_eq!(super::app_version(), "0.1.0");
    }
}
