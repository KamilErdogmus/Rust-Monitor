mod helpers;
mod network;
mod overview;
mod popups;
mod processes;
mod system;

use ratatui::Frame;

use crate::app::{App, Tab};
use crate::theme::ThemeColors;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let colors = ThemeColors::from_theme(app.theme);
    let size = frame.area();

    let main_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Min(0),
            ratatui::layout::Constraint::Length(1),
        ])
        .split(size);

    draw_tabs(frame, app, &colors, main_layout[0]);

    match app.active_tab {
        Tab::Overview => overview::draw_overview(frame, app, &colors, main_layout[1]),
        Tab::Processes => processes::draw_processes(frame, app, &colors, main_layout[1]),
        Tab::SystemInfo => system::draw_system_info(frame, app, &colors, main_layout[1]),
        Tab::NetworkDetail => network::draw_network_detail(frame, app, &colors, main_layout[1]),
    }

    draw_footer(frame, app, &colors, main_layout[2]);

    if app.show_help {
        popups::draw_help_popup(frame, &colors);
    }
    if app.kill_confirm.is_some() {
        popups::draw_kill_confirm(frame, app, &colors);
    }
    if app.show_process_detail {
        popups::draw_process_detail(frame, app, &colors);
    }
}

fn draw_tabs(frame: &mut Frame, app: &App, colors: &ThemeColors, area: ratatui::layout::Rect) {
    use ratatui::{
        style::{Modifier, Style},
        widgets::{Block, Tabs},
    };

    let titles: Vec<&str> = Tab::all().iter().map(|t| t.label()).collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::bordered()
                .title(format!(" RustMonitor — {} ", app.theme.label()))
                .border_style(Style::default().fg(colors.border)),
        )
        .select(app.active_tab.index())
        .style(Style::default().fg(colors.text_dim))
        .highlight_style(
            Style::default()
                .fg(colors.tab_active)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

fn draw_footer(frame: &mut Frame, app: &App, colors: &ThemeColors, area: ratatui::layout::Rect) {
    use ratatui::{
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::Paragraph,
    };

    let mut spans = vec![
        Span::styled(
            " q",
            Style::default()
                .fg(colors.danger)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Quit  "),
        Span::styled(
            "Tab",
            Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Tab  "),
        Span::styled(
            "?",
            Style::default()
                .fg(colors.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Help  "),
        Span::styled(
            "t",
            Style::default()
                .fg(colors.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Theme  "),
    ];

    if app.active_tab == Tab::Processes {
        spans.extend([
            Span::styled(
                "/",
                Style::default()
                    .fg(colors.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Search  "),
            Span::styled(
                "s",
                Style::default()
                    .fg(colors.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Sort  "),
            Span::styled(
                "x",
                Style::default()
                    .fg(colors.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Kill  "),
            Span::styled(
                "⏎",
                Style::default()
                    .fg(colors.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Detail  "),
        ]);
    }

    if let Some((msg, _)) = &app.status_message {
        spans.push(Span::styled(
            format!("  │ {msg}"),
            Style::default().fg(colors.accent),
        ));
    }

    let footer = Paragraph::new(Line::from(spans)).style(Style::default().bg(colors.highlight_bg));
    frame.render_widget(footer, area);
}
