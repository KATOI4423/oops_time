use std::env;

#[tauri::command]
pub fn get_authors() -> String {
    env!("CARGO_PKG_AUTHORS")
        .split(':')
        .collect::<Vec<_>>()
        .join("\n")
}

#[tauri::command]
pub fn get_homepage() -> String {
    env!("CARGO_PKG_HOMEPAGE").to_string()
}

#[tauri::command]
pub fn get_license() -> String {
    env!("CARGO_PKG_LICENSE").to_string()
}

#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn get_rustversion() -> String {
    env!("RUSTC_VERSION").to_string()
}
