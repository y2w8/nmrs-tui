use anyhow::Result;
use std::io::stdout;

#[macro_use] extern crate log;
extern crate simplelog;

mod app;
mod logger;
mod tui;
mod ui;
mod nm;

use tui::Tui;
use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize app state
    let mut app = App::new().await?;
    let mut tui = Tui::new()?;
    // Initialize
    logger::init()?;

    // Start TUI
    tui.run(&mut app).await?;

    Ok(())
}
