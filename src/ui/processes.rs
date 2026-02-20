use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::{format_bytes, format_duration, App, InputMode};
use crate::theme::ThemeColors;

pub fn draw_processes(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    draw_search_bar(frame, app, colors, chunks[0]);

    let sort_label = app.sort_by.label();
    let total = app.filtered_processes.len();

    let header = Row::new(vec![
        Cell::from("PID"),
        Cell::from("Name"),
        Cell::from("CPU%"),
        Cell::from("Memory"),
        Cell::from("Runtime"),
        Cell::from("Disk R/W"),
        Cell::from("Status"),
    ])
    .style(
        Style::default()
            .fg(colors.primary)
            .add_modifier(Modifier::BOLD),
    );

    let visible_rows = chunks[1].height.saturating_sub(4) as usize;
    let rows: Vec<Row> = app
        .filtered_processes
        .iter()
        .skip(app.process_scroll)
        .take(visible_rows)
        .enumerate()
        .filter_map(|(i, &idx)| {
            let p = app.processes.get(idx)?;
            let is_selected = i == 0;
            let style = if is_selected {
                Style::default().bg(colors.highlight_bg)
            } else {
                Style::default()
            };
            Some(
                Row::new(vec![
                    Cell::from(p.pid.to_string()),
                    Cell::from(p.name.clone()),
                    Cell::from(format!("{:.1}", p.cpu))
                        .style(Style::default().fg(colors.cpu_usage_color(p.cpu as f64))),
                    Cell::from(format_bytes(p.memory)),
                    Cell::from(format_duration(p.run_time)),
                    Cell::from(format!(
                        "{}/{}",
                        format_bytes(p.disk_read),
                        format_bytes(p.disk_write)
                    )),
                    Cell::from(p.status.clone()),
                ])
                .style(style),
            )
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Min(16),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(
        Block::bordered()
            .title(format!(
                " Processes ({total}) — Sort: {sort_label} — [{}/{}] ",
                app.process_scroll + 1,
                total
            ))
            .border_style(Style::default().fg(colors.primary)),
    );

    frame.render_widget(table, chunks[1]);
}

fn draw_search_bar(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let (label, style) = match app.input_mode {
        InputMode::Search => (
            format!(" Search: {}█ ", app.search_query),
            Style::default().fg(colors.accent),
        ),
        InputMode::Normal => {
            if app.search_query.is_empty() {
                (
                    " Press / to search ".to_string(),
                    Style::default().fg(colors.text_dim),
                )
            } else {
                (
                    format!(" Filter: {} (Esc to clear) ", app.search_query),
                    Style::default().fg(colors.accent),
                )
            }
        }
    };

    let search = Paragraph::new(label)
        .style(style)
        .block(
            Block::bordered()
                .title(" Search ")
                .border_style(Style::default().fg(colors.border)),
        );
    frame.render_widget(search, area);
}
