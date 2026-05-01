use anyhow::{Ok, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event},
    terminal::{self, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use nmrs::{Device, Network};
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
        Urgency,
        input::{Input, InputMode},
        popup, table,
    },
};

#[derive(PartialEq)]
pub enum Tabs {
    KnownNetworks,
    AvailableNetworks,
    Devices,
}

#[derive(Clone)]
pub enum Selected {
    Network(Network),
    Device(Device),
}

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,

    pub input: Input,
    pub active_tab: Tabs,
    pub selected: Option<Selected>,
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

        app.known_networks.state.select_first();
        app.available_networks.state.select_first();
        app.devices.state.select_first();

        Ok(Tui {
            terminal,

            input: Input::new(),
            selected: None,
            active_tab: Tabs::KnownNetworks,
        })
    }

    pub fn update_selected(&mut self, app: &App) {
        self.selected = match self.active_tab {
            Tabs::AvailableNetworks => app
                .available_networks
                .state
                .selected()
                .and_then(|i| app.available_networks.items.get(i))
                .map(|n| Selected::Network(n.clone())),

            Tabs::KnownNetworks => app
                .known_networks
                .state
                .selected()
                .and_then(|i| app.known_networks.items.get(i))
                .map(|n| Selected::Network(n.clone())),

            Tabs::Devices => app
                .devices
                .state
                .selected()
                .and_then(|i| app.devices.items.get(i))
                .map(|d| Selected::Device(d.clone())),
        };
    }

    pub async fn run(&mut self, app: &mut App) -> Result<()> {
        let mut last_tick = tokio::time::Instant::now();
        let tick_rate = Duration::from_secs(3);
        self.update_selected(app);

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

                table::draw_known_network(f, &body_chunks, app, &self.active_tab);
                table::draw_available_network(f, &body_chunks, app, &self.active_tab);
                table::draw_devices(f, &body_chunks, app, &self.active_tab);

                if self.input.mode == InputMode::Editing {
                    popup::draw_auth(f, &self.input, &self.selected);
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

    // terminal cleanup
    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        self.terminal
            .backend_mut()
            .execute(LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
