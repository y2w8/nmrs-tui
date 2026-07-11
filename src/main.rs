use anyhow::Result;

#[macro_use]
extern crate log;
extern crate simplelog;

mod action;
mod app;
mod cli;
mod config;
mod events;
mod logger;
mod network_manager;
mod timer;
mod tui;
mod ui;

use app::App;
use log::logger;
use tui::Tui;

use crate::{cli::Cli, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::new();
    if cli.args.config {
        if let Err(e) = Config::create(&Config::default()) {
            error!("Failed to write config: {}", e);
        };
    } else {
        // Initialize
        logger::init()?;
        let config = match Config::load() {
            Ok(cfg) => cfg,
            Err(e) => {
                error!("Failed to load config, using defaults: {}", e);
                Config::default()
            }
        };
        let mut app = App::new(config).await?;
        let mut tui = Tui::new()?;

        // Start TUI
        if let Err(e) = tui.run(&mut app).await {
            tui.cleanup()?;

            error!("Fatal error: {:#}", e);
            logger().flush();
            return Err(e);
        }
    }

    Ok(())
}
