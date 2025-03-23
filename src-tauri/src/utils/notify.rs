/**
 * notify utilities
 */

use anyhow::Context;
use std::env;
use winrt_toast::{ Header, Text, Toast, ToastManager };
use winrt_toast::content::text::TextPlacement;

/* AUMIDが定義されているファイルを読み込む */
include!(concat!(env!("OUT_DIR"), "/aumid.rs"));

#[tauri::command]
pub fn send_notify(title: &str, body: &str) -> Result<(), tauri::Error>
{
	let manager = ToastManager::new(AUMID);
	let mut toast = Toast::new();
	toast
		.text1(title)
		.text2(Text::new(body));

	manager.show(&toast).context("Failed to show toast")?;

	Ok(())
}
