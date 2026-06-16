use crossterm::{
    ExecutableCommand,
    event::{Event, EventStream},
    terminal::{self, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use std::{io::stdout, time::Duration};
use tokio::time;
use tokio_stream::StreamExt;

use crate::{
    action::{Action, ActionHandler},
    app::{App, Focus, Popups, Selected},
    events,
    ui::{
        help, popup, table,
        toast::{self, Urgency},
    },
};

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    pub should_render: bool,
}

impl Tui {
    pub fn new() -> anyhow::Result<Self> {
        let mut stdout = stdout();
        enable_raw_mode()?;
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Tui {
            terminal,
            should_render: true,
        })
    }

    pub async fn run(&mut self, app: &mut App) -> anyhow::Result<()> {
        let mut reader = EventStream::new();
        let mut last_tick = time::Instant::now();
        let tick_rate = Duration::from_secs(1);

        while !app.should_quit {
            if self.should_render {
                self.terminal.draw(|f| {
                    Self::draw(f, app);
                })?;
                self.should_render = false;
            }

            // Render when needed
            tokio::select! {
                // Render & Handle events
                maybe_event = reader.next() => {
                    if let Some(Ok(Event::Key(key))) = maybe_event {
                        events::handle_events(app, key).await?;
                        info!("Event: {:?}!", key);
                        self.should_render = true;
                    }
                }

                // Render when there no action or event
                _ = time::sleep(tick_rate.saturating_sub(last_tick.elapsed())) => {
                    info!("Ticks!");
                    self.should_render = true;
                }
            }

            let now = time::Instant::now();
            let delta = now.saturating_duration_since(last_tick);
            last_tick = now;
            app.action.send(Action::Tick(delta));

            ActionHandler::handle_actions(app).await?;
        }

        self.cleanup()?;
        Ok(())
    }

    fn draw(f: &mut Frame, app: &mut App) {
        let size = f.area();

        let chunks = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(50), // Known Networks
                Constraint::Percentage(50), // Available Networks
                Constraint::Min(5),         // Devices
                Constraint::Length(1),      // Help
            ],
        )
        .split(size);

        table::draw_known_network(f, chunks[0], app);
        table::draw_available_network(f, chunks[1], app);
        table::draw_devices(f, chunks[2], app);
        help::draw(f, chunks[3], app.focus);

        if let Focus::Popup(popup) = app.focus {
            match popup {
                Popups::Password => {
                    if let Some(Selected::Network(net)) = app.selected() {
                        popup::draw_auth(f, &app.input, net)
                    } else {
                        app.action.send(Action::ShowToast(
                            None,
                            "Can't find selected network".into(),
                            Urgency::Critical,
                            None,
                        ));
                        app.action.send(Action::SetFocus(app.last_focus));
                    }
                }
            }
        }

        toast::draw(f, &app.toasts);
    }

    // terminal cleanup
    pub fn cleanup(&mut self) -> anyhow::Result<()> {
        disable_raw_mode()?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
