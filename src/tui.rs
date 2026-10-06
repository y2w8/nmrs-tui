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
use std::{io, time::Duration};
use tokio::time;
use tokio_stream::StreamExt;

use crate::{
    action::{Action, ActionHandler, ToastRequest}, app::{App, Focus, Popups, Selected}, events, ui::{
        PanelKind, header, help, popup, table,
        toast::{self, Urgency},
    },
};

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new() -> anyhow::Result<Self> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Tui { terminal })
    }

    pub async fn run(&mut self, app: &mut App) -> anyhow::Result<()> {
        let mut reader = EventStream::new();
        let mut last_tick = time::Instant::now();
        let mut tick_interval = time::interval(Duration::from_millis(16));

        while !app.should_quit {
            self.terminal.draw(|f| {
                Self::draw(f, app);
            })?;

            tokio::select! {
                maybe_event = reader.next() => {

                    if let Some(Ok(Event::Key(key))) = maybe_event
                    {
                        events::handle_events(app, key).await?;
                        trace!("Event: {:?}!", key);
                    }
                }

                _ = tick_interval.tick() => {

                }
            }

            let now = time::Instant::now();
            let delta = now.saturating_duration_since(last_tick);
            last_tick = now;
            app.action.send(Action::Tick(delta));

            ActionHandler::handle_actions(app).await?;
        }

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
                PanelKind::Header => header::draw(f, *area, app),
                PanelKind::KnownNetworks => table::draw_known_network(f, *area, app),
                PanelKind::AvailableNetworks => table::draw_available_network(f, *area, app),
                PanelKind::Devices => table::draw_devices(f, *area, app),
                PanelKind::Help => help::draw(f, *area, app.focus),
            }
        }

        #[allow(clippy::single_match)]
        match app.focus {
            Focus::Popup(popup) => match popup {
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

            },
            _ => {}
        }

        toast::draw(f, &app.config.ui.toast, &app.toasts);
    }

    pub fn restore_terminal(terminal: Option<&mut CrosstermBackend<io::Stdout>>) {
        let _ = disable_raw_mode();
        if let Some(terminal) = terminal {
            let _ = terminal.execute(LeaveAlternateScreen);
            let _ = terminal.execute(crossterm::cursor::Show);
        } else {
            let _ = io::stdout().execute(LeaveAlternateScreen);
            let _ = io::stdout().execute(crossterm::cursor::Show);
        };
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        Self::restore_terminal(Some(self.terminal.backend_mut()));
    }
}
