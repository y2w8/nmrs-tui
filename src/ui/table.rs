use ratatui::{
    Frame, 
    layout::{Constraint, Rect}, 
    style::{Style, Stylize}, 
    widgets::{Block, Row, Table}
};
use crate::{ui::list::StatefulList};

pub struct TableData<'a, T> {
    pub title: &'a str,
    pub header_cols: Vec<&'a str>,
    pub constraint: Vec<Constraint>,
    pub cells: Vec<Row<'a>>,
    pub list: &'a mut StatefulList<T>
}

pub fn draw<Any>(f: &mut Frame, area: Rect, table_data: &mut TableData<Any>, is_active: bool) {
    let border_style = if is_active { Style::new().bold().green() } else { Style::new() };
    let row_style = if is_active { Style::new().on_blue().black() } else { Style::new() };
    let table = Table::new(
        table_data.cells.clone(),
        table_data.constraint.clone(),
    )
    .header(Row::new(table_data.header_cols.clone()).bold())
    .block(
        Block::bordered()
        .title(table_data.title)
        .border_style(border_style)
        )
    .row_highlight_style(row_style);

    f.render_stateful_widget(table, area, &mut table_data.list.state);
}
