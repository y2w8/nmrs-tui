use anyhow::Result;

#[macro_use] extern crate log;
extern crate simplelog;

mod app;
mod events;
mod logger;
mod network_manager;
mod tui;
mod ui;

use app::App;
use tui::Tui;

use crate::network_manager::NetworkManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize
    logger::init()?;
    let network_manager = NetworkManager::new().await?;
    let mut app = App::new(network_manager).await?;
    let mut tui = Tui::new(&mut app)?;

    // Start TUI
    tui.run(&mut app).await?;

    Ok(())
}
