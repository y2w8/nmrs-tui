use anyhow::{Ok, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event},
    terminal::{self, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use nmrs::{Network, WifiDevice};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use std::{
    io::stdout,
    time::{self, Duration},
};

use crate::{
    app::{App, AppEvent},
    events,
    ui::{
        help,
        input::{Input},
        popup, table,
        toast::{self, Toast},
    },
};

#[derive(Clone, Copy, PartialEq)]
pub enum Tabs {
    KnownNetworks,
    AvailableNetworks,
    Devices,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Popups {
    Password,
}

#[derive(Clone, Copy)]
pub enum Focus {
    Tab(Tabs),
    Popup(Popups),
}

#[derive(Clone)]
pub enum Selected {
    Network(Network),
    Device(WifiDevice),
}

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,

    pub input: Input,
    pub focus: Focus,
    pub last_focus: Focus,
    pub selected: Option<Selected>,
    pub toasts: Vec<Toast>,

    pub scan: bool,
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
            focus: Focus::Tab(Tabs::KnownNetworks),
            last_focus: Focus::Tab(Tabs::KnownNetworks),
            selected: None,
            toasts: Vec::new(),

            scan: true,
        })
    }

    pub fn update_selected(&mut self, app: &App) {
        self.selected = match self.focus {
            Focus::Tab(tab) => match tab {
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
            },
            Focus::Popup(_) => None,
        };
    }

    pub fn change_focus(&mut self, new_focus: Focus) {
        self.last_focus = self.focus;
        self.focus = new_focus;
    }

    pub async fn run(&mut self, app: &mut App) -> Result<()> {
        let mut last_tick = time::Instant::now();
        let mut rescan_timer = Duration::from_secs(10);
        self.update_selected(app);

        while !app.should_quit {
            let now = time::Instant::now();
            let delta = now.saturating_duration_since(last_tick);
            last_tick = now;

            // Toast duration timer
            self.toasts.iter_mut().for_each(|toast| {
                toast.duration = toast.duration.saturating_sub(delta);
            });
            self.toasts.retain(|toast| !toast.duration.is_zero());

            // Rescan timer
            rescan_timer = rescan_timer.saturating_sub(delta);
            if rescan_timer.is_zero() {
                app.event_sender.send(AppEvent::Refresh)?;
                rescan_timer = Duration::from_secs(10);
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
                        Constraint::Percentage(45),
                        Constraint::Percentage(45),
                        Constraint::Percentage(10),
                        Constraint::Length(1),
                    ],
                )
                .split(main_chunks[1]);

                table::draw_known_network(f, &body_chunks, app, &self.focus);
                table::draw_available_network(f, &body_chunks, app, &self.focus);
                table::draw_devices(f, &body_chunks, app, &self.focus);
                help::draw(f, body_chunks[3], &self.focus);

                if let Focus::Popup(popup) = self.focus {
                    match popup {
                        Popups::Password => popup::draw_auth(
                            f,
                            &self.input,
                            &self.selected,
                            self.input.hidden_password,
                        ),
                    }
                }

                toast::draw(f, &self.toasts);
            })?;

            while let std::result::Result::Ok(event) = app.event_receiver.try_recv() {
                match event {
                    AppEvent::Toast(title, msg, urgency, duration) => {
                        self.toasts.push(Toast::new(title, msg, urgency, duration));
                    }
                    AppEvent::Refresh => {
                        if self.scan {
                            app.refresh_networks().await?;
                            self.update_selected(app);
                        }
                    }
                }
            }

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
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
