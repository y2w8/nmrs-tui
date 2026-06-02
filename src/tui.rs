use anyhow::{Ok, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event},
    terminal::{self, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use std::{
    io::stdout,
    time::{self},
};

use crate::{
    action::{Action, Actions},
    app::{App, Focus, Popups, Selected},
    events,
    ui::{
        help,
        popup, table,
        toast::{self, Urgency},
    },
};

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
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

        Ok(Tui { terminal })
    }

    pub async fn run(&mut self, app: &mut App) -> Result<()> {
        let mut last_tick = time::Instant::now();

        while !app.should_quit {
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

                table::draw_known_network(f, &body_chunks, app);
                table::draw_available_network(f, &body_chunks, app);
                table::draw_devices(f, &body_chunks, app);
                help::draw(f, body_chunks[3], app.focus);

                if let Focus::Popup(popup) = app.focus {
                    match popup {
                        Popups::Password => {
                            if let Some(Selected::Network(net)) = app.selected() {
                                popup::draw_auth(f, &app.input, net, app.input.hidden_password)
                            } else {
                                app.action.send(Actions::ShowToast(
                                    None,
                                    "Can't find selected network".into(),
                                    Urgency::Critical,
                                    None,
                                ));
                                app.action.send(Actions::SetFocus(app.last_focus));
                            }
                        }
                    }
                }

                toast::draw(f, &app.toasts);
            })?;

            let now = time::Instant::now();
            let delta = now.saturating_duration_since(last_tick);
            last_tick = now;

            app.action.send(Actions::Tick(delta));

            Action::handle_actions(app).await?;

            if event::poll(std::time::Duration::from_millis(200))?
                && let Event::Key(key) = event::read()?
            {
                events::handle_events(app, key).await?;
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
