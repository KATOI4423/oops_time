// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod utils;

use clap::Parser;
use utils::keyhook;
use utils::notify;
use utils::setting;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'd', long)]
    debug: bool,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args = Args::parse();
    if args.debug {
        match notify::send_notify("OopsTime debug mode", "Debug mode is enable") {
            Ok(()) => (),
            Err(err) => panic!("Failed to send notify: {}", err),
        }
    }

    // ロガーの初期化を一番最初に行う
    setting::init_logger(&args.debug);

    keyhook::init_keyhook();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, notify::send_notify,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
