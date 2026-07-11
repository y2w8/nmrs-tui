use crossterm::{
    ExecutableCommand,
    event::{Event, EventStream},
    terminal::{self, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
};
use std::{io::stdout, time::Duration};
use tokio::time;
use tokio_stream::StreamExt;

use crate::{
    action::{Action, ActionHandler, ToastRequest},
    app::{App, Focus, Popups, Selected},
    events,
    ui::{
        PanelKind, help, popup, table,
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
                        debug!("Event: {:?}!", key);
                        self.should_render = true;
                    }
                }

                // Render when there no action or event
                _ = time::sleep(tick_rate.saturating_sub(last_tick.elapsed())) => {
                    debug!("Ticks!");
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
        let layout = &app.config.ui.layout;

        let panel_kinds: Vec<PanelKind> =
            app.config.ui.layout.panels.iter().map(|p| p.kind).collect();
        let constraints: Vec<Constraint> = layout.panels.iter().map(|p| p.constraint).collect();

        let chunks = Layout::new(layout.direction, constraints).split(size);

        for (kind, area) in panel_kinds.iter().zip(chunks.iter()) {
            match kind {
                PanelKind::KnownNetworks => table::draw_known_network(f, *area, app),
                PanelKind::AvailableNetworks => table::draw_available_network(f, *area, app),
                PanelKind::Devices => table::draw_devices(f, *area, app),
                PanelKind::Help => help::draw(f, *area, app.focus),
            }
        }

        if let Focus::Popup(popup) = app.focus {
            match popup {
                Popups::Password => {
                    if let Some(Selected::Network(net)) = app.selected() {
                        popup::draw_auth(f, &app.input, net, &app.config.ui.password_popup)
                    } else {
                        app.action.send(Action::ShowToast(Box::new(ToastRequest {
                            title: None,
                            msg: "Can't find selected network".into(),
                            urgency: Urgency::Critical,
                            duration: None,
                        })));
                        app.action.send(Action::SetFocus(app.last_focus));
                    }
                }
            }
        }

        toast::draw(f, &app.config.ui.toast, &app.toasts);
    }

    // terminal cleanup
    pub fn cleanup(&mut self) -> anyhow::Result<()> {
        disable_raw_mode()?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
