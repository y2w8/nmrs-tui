use anyhow::Result;
use log::{LevelFilter, logger};
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::fs::File;
use std::panic;
use std::path::PathBuf;
use std::{env, fs};

use crate::tui::Tui;

pub fn init() -> Result<()> {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        Tui::restore_terminal();
        error!("APPLICATION PANIC: {}", panic_info);
        logger().flush();

        default_hook(panic_info);
    }));

    let level = match env::var("NMRS_LOG")
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
    let log_path = get_log_path();
    let log_file = File::create(&log_path).expect("Failed to create log file.");

    CombinedLogger::init(vec![WriteLogger::new(level, Config::default(), log_file)]).unwrap();

    info!("Logger initialized successfully at {:?}", log_path);
    Ok(())
}

pub fn get_log_path() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")); // fallback to cwd

    let dir = base.join("nmrs-tui");
    fs::create_dir_all(&dir).expect("Failed to create log dir");
    dir.join("nmrs-tui.log")
}
