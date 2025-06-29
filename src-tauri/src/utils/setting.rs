//! apps setting

use chrono::Local;
use flexi_logger::{
    Cleanup, Criterion, FileSpec, LevelFilter, LogSpecBuilder, Logger, Naming, Record,
};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize}; // to edit cargo file
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::utils::keyhook;

fn custom_format(
    w: &mut dyn Write,
    now: &mut flexi_logger::DeferredNow,
    record: &Record,
) -> std::io::Result<()> {
    write!(
        w,
        "{level}\t{time}\t{string}",
        level = record.level(),
        time = now.now().format("%Y/%m/%d-%H:%M:%S"),
        string = &record.args()
    )
}

pub fn init_logger(is_debug_mode: &bool) {
    const MAX_FILE_SIZE: u64 = 1024 * 1024; // ローテションサイズ
    const MAX_FILE_NUM: usize = 10; // ログファイルの最大保存数

    let log_level = if *is_debug_mode {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let mut builder = LogSpecBuilder::new();
    builder.default(log_level);

    let log_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| {
            panic!("Could not determine executable directory");
        })
        .join("logs");

    match Logger::with(builder.build())
        .log_to_file(FileSpec::default().directory(log_dir))
        .append()
        .rotate(
            Criterion::Size(MAX_FILE_SIZE),
            Naming::Numbers,
            Cleanup::KeepLogFiles(MAX_FILE_NUM),
        )
        .format(custom_format)
        .start()
    {
        Ok(_logger) => {
            debug!(
                "Application started at {}",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            );
        }
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct MisstypeConfig {
    threshold: f64,
    count: usize,
    interval: u64,
    afterallow: bool,
}

impl Default for MisstypeConfig {
    fn default() -> Self {
        Self {
            threshold:  0.1,
            count:      100,
            interval:   5,
            afterallow: true,
        }
    }
}

fn config_file_path() -> PathBuf {
    //! 設定ファイルのパスを返す
    Path::new(".").join("config").join("config.toml")
}

impl MisstypeConfig {
    fn load() -> Option<Self> {
        //! 設定値をファイルからロード
        let path = config_file_path();

        match fs::read_to_string(&path) {
            Err(e) => {
                warn!("Failed to read config file {file}: {err}", file=path.display(), err=e);
                info!("Create new config file");
                Self::save(&Self::default());
            },
            Ok(config_str) => match toml::from_str::<Self>(&config_str) {
                Err(e) => error!("Failed to parse config file: {}", e),
                Ok(config) => {
                    debug!("Config loaded successfully from {:?}", path);
                    return Some(config);
                }
            },
        }

        return None;
    }

    pub fn save(&self) {
        let path = config_file_path();

        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create config directory: {}", e);
            }
        }

        match toml::to_string_pretty(self) {
            Err(_) => error!("Failed to serialize config"),
            Ok(config_str) => match fs::write(path, config_str) {
                Err(e) => error!("Failed to save config file: {}", e),
                Ok(_) => debug!("Config saved successfully"),
            },
        }
    }

    pub fn get_threshold(&self) -> f64 {
        //! `threshold` の取得用メソッド
        self.threshold
    }

    pub fn set_threshold(&mut self, value: f64) {
        //! `threshold` を更新
        self.threshold = value;
    }

    pub fn get_count(&self) -> usize {
        //! `count` の取得用メソッド
        self.count
    }

    pub fn set_count(&mut self, value: usize) {
        //! `count` を更新
        self.count = value;
    }

    pub fn get_interval(&self) -> u64 {
        //! `interval` の取得用メソッド
        self.interval
    }

    pub fn set_interval(&mut self, value: u64) {
        //! `interval` を更新
        self.interval = value;
    }

    pub fn get_afterallow(&self) -> bool {
        //! `afterallow` の取得用メソッド
        self.afterallow
    }

    pub fn set_afterallow(&mut self, value: bool) {
        //! `afterallow` を更新
        self.afterallow = value;
    }
}

static CONFIG: Lazy<RwLock<MisstypeConfig>> = Lazy::new(|| {
    RwLock::new(MisstypeConfig::load().unwrap_or_else(MisstypeConfig::default)) // configからの初期化に失敗した場合は default コンストラクタにより初期化する
});

#[tauri::command]
pub fn save_config() {
    //! グローバル変数 `CONFIG` の内容を外部ファイルに保存する関数.
    //! set系関数を呼んだあとは、必ずこの関数を使用して保存してください.
    let cfg = CONFIG.read().expect("CONFIG RwLock poisoned");
    cfg.save();
}

#[tauri::command]
pub fn get_threshold() -> f64 {
    //! グローバル変数 `CONFIG` から `threshold` を取得するメソッド
    let cfg = CONFIG.read().expect("CONFIG RwLock poisoned");
    cfg.get_threshold()
}

#[tauri::command]
pub fn set_threshold(value: f64) {
    //! グローバル変数 `CONFIG` の `threshold` を更新するメソッド.
    //! 外部ファイルへは保存されないため、`save_config` 関数を必ず使用してください.
    let mut cfg = CONFIG.write().expect("CONFIG RwLock poisoned");
    cfg.set_threshold(value);
}

#[tauri::command]
pub fn get_count() -> usize {
    //! グローバル変数 `CONFIG` から `count` を取得するメソッド
    let cfg = CONFIG.read().expect("CONFIG RwLock poisoned");
    cfg.get_count()
}

#[tauri::command]
pub fn set_count(value: usize) {
    //! グローバル変数 `CONFIG` の `count` を更新するメソッド.
    //! グローバル変数 `HISTORY` の `max_hisotry_size` も更新する.
    //! 外部ファイルへは保存されないため、`save_config` 関数を必ず使用してください.
    let mut cfg = CONFIG.write().expect("CONFIG RwLock poisoned");
    cfg.set_count(value);
    keyhook::change_max_history_size(value);
}

#[tauri::command]
pub fn get_interval() -> u64 {
    //! グローバル変数 `CONFIG` から `interval` を取得するメソッド
    let cfg = CONFIG.read().expect("CONFIG RwLock poisoned");
    cfg.get_interval()
}

#[tauri::command]
pub fn set_interval(value: u64) {
    //! グローバル変数 `CONFIG` の `interval` を更新するメソッド.
    //! 外部ファイルへは保存されないため、`save_config` 関数を必ず使用してください.
    let mut cfg = CONFIG.write().expect("CONFIG RwLock poisoned");
    cfg.set_interval(value);
}

#[tauri::command]
pub fn get_afterallow() -> bool {
    //! グローバル変数 `CONFIG` から `afterallow` を取得するメソッド
    let cfg = CONFIG.read().expect("CONFIG RwLock poisoned");
    cfg.get_afterallow()
}

#[tauri::command]
pub fn set_afterallow(value: bool) {
    //! グローバル変数 `CONFIG` の `afterallow` を更新するメソッド.
    //! 外部ファイルへは保存されないため、`save_config` 関数を必ず使用してください.
    let mut cfg = CONFIG.write().expect("CONFIG RwLock poisoned");
    cfg.set_afterallow(value);
}
