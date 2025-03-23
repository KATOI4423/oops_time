// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod utils;

use std::thread;
use utils::keyhook;
use utils::notify;

#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	keyhook::init_logger();

	tauri::Builder::default()
		.setup(|_app| {
			thread::spawn(|| {
				keyhook::set_keyboard_hook();
			});
			Ok(())
		})
		.plugin(tauri_plugin_opener::init())
		.invoke_handler(tauri::generate_handler![
			greet,
			notify::send_notify,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
