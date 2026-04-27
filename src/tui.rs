use anyhow::{Ok, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use nmrs::Network;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use std::{io::stdout, time::Duration};

use crate::{
    app::App,
    events,
    ui::{
        self,
        input::{Input, InputMode},
        list::StatefulList,
    },
};

#[derive(PartialEq)]
pub enum Tabs {
    KnownNetworks,
    AvailableNetworks,
    Devices,
}

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,

    pub input: Input,
    pub active_tab: Tabs,
    pub selected_network: Option<Network>,
}

impl Tui {
    pub fn new(app: &mut App) -> anyhow::Result<Self> {
        let mut stdout = stdout();
        enable_raw_mode()?;
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        if !app.available_networks.items.is_empty() {
            app.available_networks.state.select(Some(0));
        }
        if !app.known_networks.items.is_empty() {
            app.known_networks.state.select(Some(0));
        }
        if !app.devices.items.is_empty() {
            app.devices.state.select(Some(0));
        }

        Ok(Tui {
            terminal,

            input: Input::new(),
            selected_network: Self::selected_network(&app.available_networks),
            active_tab: Tabs::KnownNetworks,
        })
    }

    pub fn selected_network(list: &StatefulList<Network>) -> Option<Network> {
        list.state
            .selected()
            .and_then(|index| list.items.get(index))
            .cloned()
    }

    pub async fn run(&mut self, app: &mut App) -> Result<()> {
        let mut last_tick = tokio::time::Instant::now();
        let tick_rate = Duration::from_secs(3);

        while !app.should_quit {
            if last_tick.elapsed() >= tick_rate {
                if let Err(e) = app.refresh_networks().await {
                    eprintln!("scan error: {:?}", e);
                }
                last_tick = tokio::time::Instant::now();
            }
            self.terminal.draw(|f| {
                let size = f.area();

                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(0)])
                    .split(size);

                let body_chunks = Layout::new(
                    Direction::Vertical,
                    [
                        Constraint::Percentage(40),
                        Constraint::Percentage(40),
                        Constraint::Percentage(20),
                    ],
                )
                .split(main_chunks[1]);

                ui::render_all_tables(f, body_chunks, app, &self.active_tab);

                if self.input.mode == InputMode::Editing {
                    ui::render_password_popup(f, &self.input, &self.selected_network);
                }
            })?;

            if event::poll(std::time::Duration::from_millis(200))?
                && let Event::Key(key) = event::read()?
            {
                events::handle_events(app, self, key).await?;
            }
        }

        self.cleanup()?;
        Ok(())
    }

    // Helper for terminal cleanup
    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        self.terminal
            .backend_mut()
            .execute(terminal::LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
