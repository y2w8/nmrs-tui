use anyhow::{Ok, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use nmrs::Network;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::Cell,
};
use ratatui::{layout::Rect, widgets::Row};
use std::{io::stdout, rc::Rc, time::Duration};

use crate::{
    app::{App, InputMode}, events, ui::{
        list::StatefulList,
        table::{self, TableData},
    }
};

#[derive(PartialEq, Clone, Copy)]
pub enum Tabs {
    KnownNetworks,
    AvailableNetworks,
    Devices,
}

#[derive(Clone, Copy)]
pub struct Tui<'a> {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,

    pub active_tab: Tabs,
    pub selected_network: Option<&'a Network>,
}

impl<'a> Tui<'a> {
    pub fn new(app: &'a mut App) -> anyhow::Result<Self> {
        let mut stdout = stdout();
        enable_raw_mode()?;
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        if !app.new_networks.items.is_empty() {
            app.new_networks.state.select(Some(0));
        }
        if !app.known_networks.items.is_empty() {
            app.known_networks.state.select(Some(0));
        }
        if !app.devices.items.is_empty() {
            app.devices.state.select(Some(0));
        }

        Ok(Tui {
            terminal,
            selected_network: Self::selected_network(&app.new_networks),
            active_tab: Tabs::KnownNetworks,
        })
    }

    pub fn selected_network(list: &'a StatefulList<Network>) -> Option<&'a Network> {
        list.state
            .selected()
            .and_then(|index| list.items.get(index)).map(|v| &**v)
    }

    pub async fn run(
        &'a mut self,
        app: &'a mut App<'a>,
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {

        let mut last_tick = tokio::time::Instant::now();
        let tick_rate = Duration::from_secs(3);

        while !app.should_quit {
            if last_tick.elapsed() >= tick_rate {
                // if let Err(e) = *app.scan_networks().await {
                //     eprintln!("scan error: {:?}", e);
                // }
                last_tick = tokio::time::Instant::now();
            }
            terminal.draw(|f| {


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

                self.render_all_tables(f, body_chunks, app);

                if app.input_mode == InputMode::Editing {
                    self.render_password_popup(f, app);
                }

                fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
                    let popup_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage((100 - percent_y) / 2),
                            Constraint::Percentage(percent_y),
                            Constraint::Percentage((100 - percent_y) / 2),
                        ])
                        .split(r);

                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage((100 - percent_x) / 2),
                            Constraint::Percentage(percent_x),
                            Constraint::Percentage((100 - percent_x) / 2),
                        ])
                        .split(popup_layout[1])[1]
                }

                if app.input_mode == InputMode::Editing {
                    let area = centered_rect(60, 20, f.area());
                    let block = ratatui::widgets::Block::bordered()
                        .title(" Enter Password ")
                        .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                    let text =
                        ratatui::widgets::Paragraph::new(app.password_input.clone()).block(block);
                    f.render_widget(ratatui::widgets::Clear, area);
                    f.render_widget(text, area);
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

    fn render_password_popup(&self, f: &mut ratatui::Frame, app: &App) {
        let area = self.centered_rect(50, 3, f.area()); // Height 3 for one line + borders
        f.render_widget(ratatui::widgets::Clear, area); // Clear the table behind it

        // Hide password with asterisks for security
        let display_pass: String = app.password_input.chars().map(|_| '*').collect();

        let block = ratatui::widgets::Block::bordered()
            .title(format!(
                " Password for {} ",
                self.selected_network
                    .as_ref()
                    .map(|n| n.ssid.as_str())
                    .unwrap_or("Network")
            ))
            .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));

        let text = ratatui::widgets::Paragraph::new(display_pass).block(block);
        f.render_widget(text, area);
    }

    // Helper to calculate centered rect
    fn centered_rect(&self, percent_x: u16, height: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((r.height.saturating_sub(height)) / 2),
                Constraint::Length(height),
                Constraint::Min(0),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
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

    fn render_all_tables(&self, f: &mut Frame<'_>, body_chunks: Rc<[Rect]>, app: &mut App) {
        let rows: Vec<Row> = app
            .known_networks
            .items
            .iter()
            .map(|net| -> Row<'_> {
                Row::new(vec![
                    Cell::from(net.ssid.clone()),
                    Cell::from(if net.secured { "Secured" } else { "Open" }),
                    Cell::from(if app.network_manager.is_connected_cached(&net.ssid) {
                        "Connected"
                    } else {
                        "-"
                    }),
                    Cell::from(format!("{}%", net.strength.unwrap_or(0))),
                ])
            })
            .collect();
        table::draw(
            f,
            body_chunks[0],
            &mut TableData {
                title: " Known Networks ",
                header_cols: vec!["Name", "Security", "State", "Signal"],
                constraint: vec![
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ],
                cells: rows,
                list: &mut app.known_networks,
            },
            self.active_tab == Tabs::KnownNetworks,
        );
        let rows: Vec<Row> = app
            .new_networks
            .items
            .iter()
            .map(|net| -> Row<'_> {
                Row::new(vec![
                    Cell::from(net.ssid.clone()),
                    Cell::from(if net.secured { "Secured" } else { "Open" }),
                    Cell::from(format!("{}%", net.strength.unwrap_or(0))),
                ])
            })
            .collect();
        table::draw(
            f,
            body_chunks[1],
            &mut TableData {
                title: " Available Networks ",
                header_cols: vec!["Name", "Security", "Signal"],
                constraint: vec![
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ],
                cells: rows,
                list: &mut app.new_networks,
            },
            self.active_tab == Tabs::AvailableNetworks,
        );

        let rows: Vec<Row> = app
            .devices
            .items
            .iter()
            .map(|net| -> Row<'_> {
                Row::new(vec![
                    Cell::from(net.interface.clone()),
                    Cell::from(if app.network_manager.enabled {
                        "On"
                    } else {
                        "Off"
                    }),
                    // Cell::from(net.dri),
                ])
            })
            .collect();
        table::draw(
            f,
            body_chunks[2],
            &mut TableData {
                title: " Device ",
                header_cols: vec!["Name", "Powered", "Scanning", "Frequency", "Security"],
                constraint: vec![
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ],
                cells: rows,
                list: &mut app.devices,
            },
            self.active_tab == Tabs::Devices,
        );
    }
}
