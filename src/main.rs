use anyhow::Result;

#[macro_use]
extern crate log;
extern crate simplelog;

mod app;
mod cli;
mod config;
mod events;
mod logger;
mod network_manager;
mod tui;
mod ui;
mod action;
mod timer;

use app::App;
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
        tui.run(&mut app).await?;
    }

    Ok(())
}
