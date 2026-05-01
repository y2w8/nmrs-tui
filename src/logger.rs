use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::fs::File;
use std::io::stdout;
use std::panic;
use std::path::PathBuf;
use std::{env, fs};

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
    let log_path = get_log_path();
    let log_file = File::create(&log_path).expect("Failed to create log file.");

    CombinedLogger::init(vec![WriteLogger::new(level, Config::default(), log_file)]).unwrap();

    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        error!("APPLICATION PANIC: {}", panic_info);

        let _ = disable_raw_mode_internal();

        default_hook(panic_info);
    }));

    info!("Logger initialized successfully at {:?}", log_path);
    Ok(())
}

pub fn get_log_path() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")); // fallback to cwd

    let dir = base.join("nmrs-tui");
    fs::create_dir_all(&dir).expect("Failed to create log dir");
    dir.join("nmrs-tui.log")
}

fn disable_raw_mode_internal() -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
