use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{format_bytes, format_duration, App};
use crate::theme::ThemeColors;
use super::helpers::{centered_rect, detail_line};

pub fn draw_help_popup(frame: &mut Frame, colors: &ThemeColors) {
    let area = centered_rect(50, 60, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  General",
            Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    q / Esc    ", Style::default().fg(colors.accent)),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![
            Span::styled("    Tab        ", Style::default().fg(colors.accent)),
            Span::raw("Next tab"),
        ]),
        Line::from(vec![
            Span::styled("    Shift+Tab  ", Style::default().fg(colors.accent)),
            Span::raw("Previous tab"),
        ]),
        Line::from(vec![
            Span::styled("    ?          ", Style::default().fg(colors.accent)),
            Span::raw("Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("    t          ", Style::default().fg(colors.accent)),
            Span::raw("Cycle theme"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Navigation",
            Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    ↑/k ↓/j    ", Style::default().fg(colors.accent)),
            Span::raw("Scroll up/down"),
        ]),
        Line::from(vec![
            Span::styled("    PgUp/PgDn  ", Style::default().fg(colors.accent)),
            Span::raw("Page up/down"),
        ]),
        Line::from(vec![
            Span::styled("    Home/End   ", Style::default().fg(colors.accent)),
            Span::raw("Jump to top/bottom"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Processes",
            Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    /          ", Style::default().fg(colors.accent)),
            Span::raw("Search processes"),
        ]),
        Line::from(vec![
            Span::styled("    s          ", Style::default().fg(colors.accent)),
            Span::raw("Cycle sort (CPU → MEM → Name → PID)"),
        ]),
        Line::from(vec![
            Span::styled("    x          ", Style::default().fg(colors.accent)),
            Span::raw("Kill selected process"),
        ]),
        Line::from(vec![
            Span::styled("    Enter      ", Style::default().fg(colors.accent)),
            Span::raw("View process details"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Press any key to close",
            Style::default().fg(colors.text_dim),
        )]),
    ];

    let help = Paragraph::new(help_text).block(
        Block::bordered()
            .title(" Help ")
            .border_style(Style::default().fg(colors.primary)),
    );
    frame.render_widget(help, area);
}

pub fn draw_kill_confirm(frame: &mut Frame, app: &App, colors: &ThemeColors) {
    let area = centered_rect(40, 20, frame.area());
    frame.render_widget(Clear, area);

    let pid = app.kill_confirm.unwrap_or(0);
    let name = app
        .selected_process()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "?".into());

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Kill process ", Style::default().fg(colors.danger)),
            Span::styled(
                name,
                Style::default()
                    .fg(colors.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" (PID {pid})?"),
                Style::default().fg(colors.danger),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  y",
                Style::default()
                    .fg(colors.danger)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Yes   "),
            Span::styled(
                "n",
                Style::default()
                    .fg(colors.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" No"),
        ]),
    ];

    let popup = Paragraph::new(text).block(
        Block::bordered()
            .title(" Confirm Kill ")
            .border_style(Style::default().fg(colors.danger)),
    );
    frame.render_widget(popup, area);
}

pub fn draw_process_detail(frame: &mut Frame, app: &App, colors: &ThemeColors) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);

    let detail = match &app.process_detail {
        Some(d) => d,
        None => return,
    };

    let lines = vec![
        Line::from(""),
        detail_line("PID", &detail.base.pid.to_string(), colors),
        detail_line("Name", &detail.base.name, colors),
        detail_line("Status", &detail.base.status, colors),
        Line::from(""),
        detail_line("CPU Usage", &format!("{:.1}%", detail.base.cpu), colors),
        detail_line("Memory", &format_bytes(detail.base.memory), colors),
        detail_line("Virtual Memory", &format_bytes(detail.virtual_memory), colors),
        detail_line(
            "Threads",
            &detail
                .threads
                .map(|t| t.to_string())
                .unwrap_or_else(|| "N/A".into()),
            colors,
        ),
        Line::from(""),
        detail_line("Runtime", &format_duration(detail.base.run_time), colors),
        detail_line("Disk Read", &format_bytes(detail.base.disk_read), colors),
        detail_line("Disk Write", &format_bytes(detail.base.disk_write), colors),
        Line::from(""),
        detail_line(
            "Parent PID",
            &detail
                .parent_pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "None".into()),
            colors,
        ),
        detail_line(
            "Executable",
            if detail.exe.is_empty() {
                "N/A"
            } else {
                &detail.exe
            },
            colors,
        ),
        detail_line(
            "Root",
            if detail.root.is_empty() {
                "N/A"
            } else {
                &detail.root
            },
            colors,
        ),
        detail_line(
            "Command",
            if detail.cmd.is_empty() {
                "N/A"
            } else {
                &detail.cmd
            },
            colors,
        ),
        detail_line("Env Variables", &detail.environ_count.to_string(), colors),
        Line::from(""),
        Line::from(Span::styled(
            "  Press any key to close",
            Style::default().fg(colors.text_dim),
        )),
    ];

    let popup = Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(format!(
                    " Process Detail — {} (PID {}) ",
                    detail.base.name, detail.base.pid
                ))
                .border_style(Style::default().fg(colors.primary)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(popup, area);
}
