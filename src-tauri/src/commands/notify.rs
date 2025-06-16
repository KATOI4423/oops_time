/**
 * notify utilities
 */
use anyhow::Context;
use std::env;
use winrt_toast::{Text, Toast, ToastManager};


#[tauri::command]
pub fn send_notify(title: &str, body: &str) -> Result<(), tauri::Error> {
    let aumid = env!("AUMID");
    let manager = ToastManager::new(aumid);
    let mut toast = Toast::new();
    toast.text1(title).text2(Text::new(body));

    manager.show(&toast).context("Failed to show toast")?;

    Ok(())
}
