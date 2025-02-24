// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod utils;

use std::thread;
use utils::keyhook;

#[tauri::command]
fn greet(name: &str) -> String {
	format!("Hello, {}! You've been greeted from Rust!", name)
}

use anyhow::Context;
use winrt_toast::{ Header, Text, Toast, ToastManager };
use winrt_toast::content::text::TextPlacement;

#[tauri::command]
fn notify(title: &str, body: &str) -> Result<(), tauri::Error>
{
	const AUM_ID: &str = "com.oopstime.app";
	let manager = ToastManager::new(AUM_ID);
	let mut toast = Toast::new();
	toast
		.text1(title)
		.text2(Text::new(body));

	manager.show(&toast).context("Failed to show toast")?;

	Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	keyhook::keyhook_windows::init_logger();

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
			notify,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
