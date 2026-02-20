use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::theme::ThemeColors;

pub fn info_line(label: &str, value: &str, colors: &ThemeColors) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {label:<20} "),
            Style::default()
                .fg(colors.text_dim)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(colors.text)),
    ])
}

pub fn detail_line(label: &str, value: &str, colors: &ThemeColors) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {label:<18} "),
            Style::default()
                .fg(colors.text_dim)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(colors.text)),
    ])
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn shrink_rect(rect: Rect, horizontal: u16, vertical: u16) -> Rect {
    Rect {
        x: rect.x + horizontal,
        y: rect.y + vertical,
        width: rect.width.saturating_sub(horizontal * 2),
        height: rect.height.saturating_sub(vertical * 2),
    }
}
