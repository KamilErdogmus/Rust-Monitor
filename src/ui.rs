use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Cell, Clear, Gauge, Paragraph, Row, Sparkline, Table, Tabs, Wrap},
    Frame,
};

use crate::app::{format_bytes, format_duration, App, InputMode, Tab};
use crate::theme::ThemeColors;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let colors = ThemeColors::from_theme(app.theme);
    let size = frame.area();

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    draw_tabs(frame, app, &colors, main_layout[0]);

    match app.active_tab {
        Tab::Overview => draw_overview(frame, app, &colors, main_layout[1]),
        Tab::Processes => draw_processes(frame, app, &colors, main_layout[1]),
        Tab::SystemInfo => draw_system_info(frame, app, &colors, main_layout[1]),
        Tab::NetworkDetail => draw_network_detail(frame, app, &colors, main_layout[1]),
    }

    draw_footer(frame, app, &colors, main_layout[2]);

    if app.show_help {
        draw_help_popup(frame, &colors);
    }
    if app.kill_confirm.is_some() {
        draw_kill_confirm(frame, app, &colors);
    }
}

fn draw_tabs(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let titles: Vec<&str> = Tab::all().iter().map(|t| t.label()).collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::bordered()
                .title(format!(" RustMonitor — {} ", colors_theme_label(app)))
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

fn colors_theme_label(app: &App) -> String {
    app.theme.label().to_string()
}


fn draw_overview(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[0]);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    draw_cpu(frame, app, colors, top_cols[0]);
    draw_memory(frame, app, colors, top_cols[1]);
    draw_network_overview(frame, app, colors, bottom_cols[0]);
    draw_disks(frame, app, colors, bottom_cols[1]);
}

fn draw_cpu(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let block = Block::bordered()
        .title(format!(
            " CPU — {:.1}% ({} cores) ",
            app.global_cpu, app.cpu_count
        ))
        .border_style(Style::default().fg(colors.cpu));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.cpu_count == 0 || inner.height == 0 {
        return;
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(inner);

    let global_data: Vec<u64> = app.global_cpu_history.iter().map(|v| *v as u64).collect();
    let sparkline = Sparkline::default()
        .data(&global_data)
        .max(100)
        .style(Style::default().fg(colors.cpu));
    frame.render_widget(sparkline, sections[0]);

    let cores_to_show = app.cpu_count.min(sections[1].height as usize);
    if cores_to_show == 0 {
        return;
    }

    let constraints: Vec<Constraint> = (0..cores_to_show)
        .map(|_| Constraint::Length(1))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let core_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(sections[1]);

    for i in 0..cores_to_show {
        let usage = app.cpu_history[i].back().copied().unwrap_or(0.0);
        let label = format!("Core {:>2}: {:>5.1}%", i, usage);
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(colors.cpu_usage_color(usage)))
            .percent(usage.min(100.0) as u16)
            .label(label);
        frame.render_widget(gauge, core_rows[i]);
    }
}

fn draw_memory(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let block = Block::bordered()
        .title(" Memory ")
        .border_style(Style::default().fg(colors.memory));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(inner);

    // RAM
    let ram_pct = if app.total_memory > 0 {
        ((app.used_memory as f64 / app.total_memory as f64) * 100.0) as u16
    } else {
        0
    };
    let ram_label = format!(
        "RAM: {} / {} ({ram_pct}%)",
        format_bytes(app.used_memory),
        format_bytes(app.total_memory)
    );
    let ram_gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.memory))
        .percent(ram_pct.min(100))
        .label(ram_label);
    frame.render_widget(ram_gauge, chunks[0]);

    let swap_pct = if app.total_swap > 0 {
        ((app.used_swap as f64 / app.total_swap as f64) * 100.0) as u16
    } else {
        0
    };
    let swap_label = format!(
        "Swap: {} / {} ({swap_pct}%)",
        format_bytes(app.used_swap),
        format_bytes(app.total_swap)
    );
    let swap_gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.secondary))
        .percent(swap_pct.min(100))
        .label(swap_label);
    frame.render_widget(swap_gauge, chunks[1]);

    let data: Vec<u64> = app.mem_history.iter().map(|v| *v as u64).collect();
    let sparkline = Sparkline::default()
        .data(&data)
        .max(100)
        .style(Style::default().fg(colors.memory));
    frame.render_widget(sparkline, chunks[2]);
}

fn draw_network_overview(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let block = Block::bordered()
        .title(format!(
            " Network — ↓{}/s  ↑{}/s ",
            format_bytes(app.net_rx),
            format_bytes(app.net_tx)
        ))
        .border_style(Style::default().fg(colors.network));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    let rx_label = Paragraph::new(Line::from(vec![
        Span::styled("↓ RX ", Style::default().fg(colors.success)),
    ]));
    frame.render_widget(rx_label, chunks[0]);

    let rx_data: Vec<u64> = app.net_rx_history.iter().map(|v| *v as u64).collect();
    let rx_spark = Sparkline::default()
        .data(&rx_data)
        .style(Style::default().fg(colors.success));
    frame.render_widget(rx_spark, chunks[1]);

    let tx_label = Paragraph::new(Line::from(vec![
        Span::styled("↑ TX ", Style::default().fg(colors.warning)),
    ]));
    frame.render_widget(tx_label, chunks[2]);

    let tx_data: Vec<u64> = app.net_tx_history.iter().map(|v| *v as u64).collect();
    let tx_spark = Sparkline::default()
        .data(&tx_data)
        .style(Style::default().fg(colors.warning));
    frame.render_widget(tx_spark, chunks[3]);
}

fn draw_disks(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let block = Block::bordered()
        .title(" Disks ")
        .border_style(Style::default().fg(colors.disk));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    for disk in app.disks.iter() {
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        let pct = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        let mount = disk.mount_point().to_string_lossy();
        let fs = disk.file_system().to_string_lossy();
        let bar_width = 16;
        let filled = ((pct / 100.0) * bar_width as f64) as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);

        let color = colors.disk_usage_color(pct);

        lines.push(Line::from(vec![
            Span::styled(format!("{:<4}", mount), Style::default().fg(colors.text)),
            Span::styled(format!(" [{fs}] "), Style::default().fg(colors.text_dim)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(format!("  {bar} "), Style::default().fg(color)),
            Span::styled(
                format!("{} / {} ({:.0}%)", format_bytes(used), format_bytes(total), pct),
                Style::default().fg(colors.text_dim),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_processes(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
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


fn draw_system_info(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: System details
    let info_lines = vec![
        info_line("Hostname", &app.hostname, colors),
        info_line("OS", &format!("{} {}", app.os_name, app.os_version), colors),
        info_line("Kernel", &app.kernel_version, colors),
        info_line("Architecture", &app.cpu_arch, colors),
        info_line("Boot Time", &app.boot_time_str(), colors),
        Line::from(""),
        info_line("CPU", &app.cpu_brand, colors),
        info_line("Cores", &app.cpu_count.to_string(), colors),
        info_line("CPU Usage", &format!("{:.1}%", app.global_cpu), colors),
        Line::from(""),
        info_line("Total RAM", &format_bytes(app.total_memory), colors),
        info_line("Used RAM", &format_bytes(app.used_memory), colors),
        info_line("Total Swap", &format_bytes(app.total_swap), colors),
        info_line("Used Swap", &format_bytes(app.used_swap), colors),
        Line::from(""),
        info_line("System Uptime", &app.uptime_str(), colors),
        info_line("Monitor Uptime", &app.monitor_uptime_str(), colors),
        info_line("Processes", &app.processes.len().to_string(), colors),
        info_line("Network Interfaces", &app.network_interfaces.len().to_string(), colors),
        info_line("Disks", &app.disks.iter().count().to_string(), colors),
    ];

    let info = Paragraph::new(info_lines)
        .block(
            Block::bordered()
                .title(" System Information ")
                .border_style(Style::default().fg(colors.primary)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(info, cols[0]);

    // Right: Resource summary with big gauges
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(cols[1]);

    let cpu_block = Block::bordered()
        .title(" CPU ")
        .border_style(Style::default().fg(colors.cpu));
    let cpu_inner = cpu_block.inner(right_chunks[0]);
    frame.render_widget(cpu_block, right_chunks[0]);
    let cpu_gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.cpu_usage_color(app.global_cpu as f64)))
        .percent((app.global_cpu as u16).min(100))
        .label(format!("{:.1}%", app.global_cpu));
    frame.render_widget(cpu_gauge, shrink_rect(cpu_inner, 1, 0));

    let ram_pct = if app.total_memory > 0 {
        ((app.used_memory as f64 / app.total_memory as f64) * 100.0) as u16
    } else {
        0
    };
    let ram_block = Block::bordered()
        .title(" RAM ")
        .border_style(Style::default().fg(colors.memory));
    let ram_inner = ram_block.inner(right_chunks[1]);
    frame.render_widget(ram_block, right_chunks[1]);
    let ram_gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.memory))
        .percent(ram_pct.min(100))
        .label(format!(
            "{} / {}",
            format_bytes(app.used_memory),
            format_bytes(app.total_memory)
        ));
    frame.render_widget(ram_gauge, shrink_rect(ram_inner, 1, 0));

    let swap_pct = if app.total_swap > 0 {
        ((app.used_swap as f64 / app.total_swap as f64) * 100.0) as u16
    } else {
        0
    };
    let swap_block = Block::bordered()
        .title(" Swap ")
        .border_style(Style::default().fg(colors.secondary));
    let swap_inner = swap_block.inner(right_chunks[2]);
    frame.render_widget(swap_block, right_chunks[2]);
    let swap_gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.secondary))
        .percent(swap_pct.min(100))
        .label(format!(
            "{} / {}",
            format_bytes(app.used_swap),
            format_bytes(app.total_swap)
        ));
    frame.render_widget(swap_gauge, shrink_rect(swap_inner, 1, 0));

    let history_block = Block::bordered()
        .title(" CPU History (60s) ")
        .border_style(Style::default().fg(colors.cpu));
    let history_inner = history_block.inner(right_chunks[3]);
    frame.render_widget(history_block, right_chunks[3]);
    let data: Vec<u64> = app.global_cpu_history.iter().map(|v| *v as u64).collect();
    let sparkline = Sparkline::default()
        .data(&data)
        .max(100)
        .style(Style::default().fg(colors.cpu));
    frame.render_widget(sparkline, history_inner);
}

fn info_line(label: &str, value: &str, colors: &ThemeColors) -> Line<'static> {
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


fn draw_network_detail(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let spark_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let rx_block = Block::bordered()
        .title(format!(" ↓ Download — {}/s ", format_bytes(app.net_rx)))
        .border_style(Style::default().fg(colors.success));
    let rx_inner = rx_block.inner(spark_cols[0]);
    frame.render_widget(rx_block, spark_cols[0]);
    let rx_data: Vec<u64> = app.net_rx_history.iter().map(|v| *v as u64).collect();
    let rx_spark = Sparkline::default()
        .data(&rx_data)
        .style(Style::default().fg(colors.success));
    frame.render_widget(rx_spark, rx_inner);

    let tx_block = Block::bordered()
        .title(format!(" ↑ Upload — {}/s ", format_bytes(app.net_tx)))
        .border_style(Style::default().fg(colors.warning));
    let tx_inner = tx_block.inner(spark_cols[1]);
    frame.render_widget(tx_block, spark_cols[1]);
    let tx_data: Vec<u64> = app.net_tx_history.iter().map(|v| *v as u64).collect();
    let tx_spark = Sparkline::default()
        .data(&tx_data)
        .style(Style::default().fg(colors.warning));
    frame.render_widget(tx_spark, tx_inner);

    let header = Row::new(vec![
        Cell::from("Interface"),
        Cell::from("MAC"),
        Cell::from("RX"),
        Cell::from("TX"),
        Cell::from("Pkts In"),
        Cell::from("Pkts Out"),
        Cell::from("Err In"),
        Cell::from("Err Out"),
    ])
    .style(
        Style::default()
            .fg(colors.primary)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .network_interfaces
        .iter()
        .enumerate()
        .map(|(i, iface)| {
            let style = if i == app.network_scroll {
                Style::default().bg(colors.highlight_bg)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(iface.name.clone()).style(Style::default().fg(colors.text)),
                Cell::from(iface.mac_address.clone()).style(Style::default().fg(colors.text_dim)),
                Cell::from(format_bytes(iface.received)).style(Style::default().fg(colors.success)),
                Cell::from(format_bytes(iface.transmitted)).style(Style::default().fg(colors.warning)),
                Cell::from(iface.packets_in.to_string()),
                Cell::from(iface.packets_out.to_string()),
                Cell::from(iface.errors_in.to_string())
                    .style(if iface.errors_in > 0 {
                        Style::default().fg(colors.danger)
                    } else {
                        Style::default().fg(colors.text_dim)
                    }),
                Cell::from(iface.errors_out.to_string())
                    .style(if iface.errors_out > 0 {
                        Style::default().fg(colors.danger)
                    } else {
                        Style::default().fg(colors.text_dim)
                    }),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(14),
            Constraint::Length(18),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::bordered()
            .title(format!(" Interfaces ({}) ", app.network_interfaces.len()))
            .border_style(Style::default().fg(colors.network)),
    );

    frame.render_widget(table, chunks[1]);
}

fn draw_footer(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let mut spans = vec![
        Span::styled(" q", Style::default().fg(colors.danger).add_modifier(Modifier::BOLD)),
        Span::raw(" Quit  "),
        Span::styled("Tab", Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)),
        Span::raw(" Tab  "),
        Span::styled("?", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)),
        Span::raw(" Help  "),
        Span::styled("t", Style::default().fg(colors.secondary).add_modifier(Modifier::BOLD)),
        Span::raw(" Theme  "),
    ];

    if app.active_tab == Tab::Processes {
        spans.extend([
            Span::styled("/", Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)),
            Span::raw(" Search  "),
            Span::styled("s", Style::default().fg(colors.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" Sort  "),
            Span::styled("x", Style::default().fg(colors.danger).add_modifier(Modifier::BOLD)),
            Span::raw(" Kill  "),
        ]);
    }

    if let Some((msg, _)) = &app.status_message {
        spans.push(Span::styled(
            format!("  │ {msg}"),
            Style::default().fg(colors.accent),
        ));
    }

    let footer = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(colors.highlight_bg));
    frame.render_widget(footer, area);
}

fn draw_help_popup(frame: &mut Frame, colors: &ThemeColors) {
    let area = centered_rect(50, 60, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  General", Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)),
        ]),
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
        Line::from(vec![
            Span::styled("  Navigation", Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)),
        ]),
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
        Line::from(vec![
            Span::styled("  Processes", Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)),
        ]),
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
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press any key to close", Style::default().fg(colors.text_dim)),
        ]),
    ];

    let help = Paragraph::new(help_text).block(
        Block::bordered()
            .title(" Help ")
            .border_style(Style::default().fg(colors.primary)),
    );
    frame.render_widget(help, area);
}

fn draw_kill_confirm(frame: &mut Frame, app: &App, colors: &ThemeColors) {
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
            Span::styled(name, Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" (PID {pid})?"), Style::default().fg(colors.danger)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  y", Style::default().fg(colors.danger).add_modifier(Modifier::BOLD)),
            Span::raw(" Yes   "),
            Span::styled("n", Style::default().fg(colors.success).add_modifier(Modifier::BOLD)),
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

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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

fn shrink_rect(rect: Rect, horizontal: u16, vertical: u16) -> Rect {
    Rect {
        x: rect.x + horizontal,
        y: rect.y + vertical,
        width: rect.width.saturating_sub(horizontal * 2),
        height: rect.height.saturating_sub(vertical * 2),
    }
}
