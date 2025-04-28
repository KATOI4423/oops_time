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
pub struct MisstypeConfig {
    threshold: f64,
    count: usize,
    interval: u64,
    afterallow: bool,
}

impl MisstypeConfig {
    fn new() -> Self {
        //! `MisstypeConfig`のデフォルト値を設定
        Self {
            threshold: 0.1,
            count: 100,
            interval: 5,
            afterallow: true,
        }
    }

    fn get_config_file_path() -> Result<PathBuf, &'static str> {
        //! 設定ファイルのパスを取得
        let root_path = Path::new(".");
        let config_file_path = root_path.join("config").join("config.toml");

        if config_file_path.to_str().is_some() {
            return Ok(config_file_path);
        } else {
            return Err("Invalid UTF-8 in path");
        }
    }

    fn load() -> Option<Self> {
        //! 設定値をファイルからロード
        match Self::get_config_file_path() {
            Err(e) => error!("Failed to get config file path: {}", e),
            Ok(path) => match fs::read_to_string(&path) {
                Err(e) => {
                    warn!("Failed to read config file {file}: {err}", file=path.display(), err=e);
                    info!("Create new config file");
                    Self::save(&Self::new());
                },
                Ok(config_str) => match toml::from_str::<Self>(&config_str) {
                    Err(e) => error!("Failed to parse config file: {}", e),
                    Ok(config) => {
                        debug!("Config loaded successfully from {:?}", path);
                        return Some(config);
                    }
                },
            },
        }

        return None;
    }

    pub fn save(&self) {
        match Self::get_config_file_path() {
            Err(e) => error!("Failed to get config file path: {}", e),
            Ok(path) => {
                if let Some(parent) = path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        error!("Failed to create config directory: {}", e);
                        return;
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
        }
    }

    pub fn get_threshold() -> f64 {
        //! `threshold` の取得用メソッド
        return CONFIG.read().unwrap().threshold;
    }

    pub fn get_count() -> usize {
        //! `count` の取得用メソッド
        return CONFIG.read().unwrap().count;
    }

    pub fn get_interval() -> u64 {
        //! `interval` の取得用メソッド
        return CONFIG.read().unwrap().interval;
    }

    pub fn get_afterallow() -> bool {
        //! `afterallow` の取得用メソッド
        return CONFIG.read().unwrap().afterallow;
    }

    pub fn set_threshold(value: f64) {
        //! `threshold` を更新
        let mut config = CONFIG.write().unwrap();
        config.threshold = value;
        config.save();
    }

    pub fn set_count(value: usize) {
        //! `count` を更新
        let mut config = CONFIG.write().unwrap();
        config.count = value;
        config.save();
    }

    pub fn set_interval(value: u64) {
        //! `interval` を更新
        let mut config = CONFIG.write().unwrap();
        config.interval = value;
        config.save();
    }

    pub fn set_afterallow(value: bool) {
        //! `afterallow` を更新
        let mut config = CONFIG.write().unwrap();
        config.afterallow = value;
        config.save();
    }
}

static CONFIG: Lazy<RwLock<MisstypeConfig>> = Lazy::new(|| {
    RwLock::new(MisstypeConfig::load().unwrap_or_else(MisstypeConfig::new)) // configからの初期化に失敗した場合は new 関数により初期化する
});
