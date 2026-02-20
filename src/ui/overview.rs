use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Gauge, Paragraph, Sparkline},
    Frame,
};

use crate::app::{format_bytes, App};
use crate::theme::ThemeColors;

pub fn draw_overview(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let has_gpu = app.has_gpu();

    let rows = if has_gpu {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(35),
                Constraint::Percentage(30),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    };

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(rows[0]);

    let mid_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    draw_cpu(frame, app, colors, top_cols[0]);
    draw_memory(frame, app, colors, top_cols[1]);
    draw_network_overview(frame, app, colors, mid_cols[0]);
    draw_disks(frame, app, colors, mid_cols[1]);

    if has_gpu {
        draw_gpu(frame, app, colors, rows[2]);
    }
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
                format!(
                    "{} / {} ({:.0}%)",
                    format_bytes(used),
                    format_bytes(total),
                    pct
                ),
                Style::default().fg(colors.text_dim),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_gpu(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let gpu_cols: Vec<Constraint> = app
        .gpus
        .iter()
        .map(|_| Constraint::Ratio(1, app.gpus.len() as u32))
        .collect();

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(gpu_cols)
        .split(area);

    for (i, gpu) in app.gpus.iter().enumerate() {
        let mem_pct = if gpu.memory_total > 0 {
            ((gpu.memory_used as f64 / gpu.memory_total as f64) * 100.0) as u16
        } else {
            0
        };

        let power_str = match (gpu.power_usage, gpu.power_limit) {
            (Some(usage), Some(limit)) => format!("  Power: {}W/{}W", usage / 1000, limit / 1000),
            (Some(usage), None) => format!("  Power: {}W", usage / 1000),
            _ => String::new(),
        };

        let fan_str = match gpu.fan_speed {
            Some(speed) => format!("  Fan: {speed}%"),
            None => String::new(),
        };

        let block = Block::bordered()
            .title(format!(
                " {} — {}°C  {}%{}{} ",
                gpu.name, gpu.temperature, gpu.utilization, fan_str, power_str
            ))
            .border_style(Style::default().fg(colors.accent));

        let inner = block.inner(cols[i]);
        frame.render_widget(block, cols[i]);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner);

        let util_gauge = Gauge::default()
            .gauge_style(Style::default().fg(colors.cpu_usage_color(gpu.utilization as f64)))
            .percent(gpu.utilization.min(100) as u16)
            .label(format!("GPU: {}%", gpu.utilization));
        frame.render_widget(util_gauge, chunks[0]);

        let vram_gauge = Gauge::default()
            .gauge_style(Style::default().fg(colors.memory))
            .percent(mem_pct.min(100))
            .label(format!(
                "VRAM: {} / {}",
                format_bytes(gpu.memory_used),
                format_bytes(gpu.memory_total)
            ));
        frame.render_widget(vram_gauge, chunks[1]);

        if let Some(history) = app.gpu_util_history.get(i) {
            let data: Vec<u64> = history.iter().map(|v| *v as u64).collect();
            let sparkline = Sparkline::default()
                .data(&data)
                .max(100)
                .style(Style::default().fg(colors.accent));
            frame.render_widget(sparkline, chunks[2]);
        }
    }
}
