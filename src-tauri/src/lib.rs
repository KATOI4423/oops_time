// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod utils;

use clap::Parser;
use commands::info;
use commands::license;
use commands::notify;
use utils::keyhook;
use utils::setting;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'd', long)]
    debug: bool,
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
        .invoke_handler(tauri::generate_handler![
            notify::send_notify,
            license::get_license_html,
            info::get_authors, info::get_homepage, info::get_license, info::get_version,
            info::get_rustversion,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
