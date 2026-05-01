use anyhow::Result;
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::fs::File;
use std::path::PathBuf;
use std::{env, fs};

// FIX: it does not save log file when app panic "such useless logger".
pub fn init() -> Result<()> {
    let level = match env::var("RUST_LOG")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => return Ok(()), // logging disabled
    };
    let log_file = File::create(get_log_path()).expect("Failed to create log file.");

    CombinedLogger::init(vec![WriteLogger::new(level, Config::default(), log_file)]).unwrap();
    Ok(())
}

pub fn get_log_path() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")); // fallback to cwd

    let dir = base.join("nmrs-tui");
    fs::create_dir_all(&dir).expect("Failed to create log dir");
    dir.join("nmrs-tui.log")
}
