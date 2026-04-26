use anyhow::Result;
use std::io::stdout;

#[macro_use] extern crate log;
extern crate simplelog;

mod app;
mod events;
mod logger;
mod network;
mod tui;
mod ui;

use app::App;
use crossterm::ExecutableCommand;
use crossterm::terminal::{self, enable_raw_mode};
use ratatui::{Terminal, prelude::CrosstermBackend};
use tui::Tui;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize
    logger::init()?;
    let network_manager = network::Manager::new().await?;
    let mut app = App::new(network_manager).await?;
    let mut tui = Tui::new(&mut app)?;

    let mut stdout = stdout();
    enable_raw_mode()?;
    stdout
        .execute(terminal::EnterAlternateScreen)?
        .execute(terminal::Clear(terminal::ClearType::All))?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Start TUI
    tui.run(&mut app, terminal).await?;

    Ok(())
}
