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
        let _ = Config::create(&Config::default());
    } else {
        // Initialize
        logger::init()?;
        let config = Config::load().unwrap_or_default();
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
