use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Gauge, Paragraph, Sparkline, Wrap},
    Frame,
};

use crate::app::{format_bytes, App};
use crate::theme::ThemeColors;
use super::helpers::{info_line, shrink_rect};

pub fn draw_system_info(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

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

    let mut gpu_lines: Vec<Line> = Vec::new();
    if !app.gpus.is_empty() {
        gpu_lines.push(Line::from(""));
        for (i, gpu) in app.gpus.iter().enumerate() {
            let mem_str = format!(
                "{} / {}",
                format_bytes(gpu.memory_used),
                format_bytes(gpu.memory_total)
            );
            gpu_lines.push(info_line(&format!("GPU {i}"), &gpu.name, colors));
            gpu_lines.push(info_line(
                "  Utilization",
                &format!("{}%", gpu.utilization),
                colors,
            ));
            gpu_lines.push(info_line(
                "  Temperature",
                &format!("{}°C", gpu.temperature),
                colors,
            ));
            gpu_lines.push(info_line("  VRAM", &mem_str, colors));
            if let Some(fan) = gpu.fan_speed {
                gpu_lines.push(info_line("  Fan Speed", &format!("{fan}%"), colors));
            }
            if let Some(power) = gpu.power_usage {
                let limit_str = gpu
                    .power_limit
                    .map(|l| format!(" / {}W", l / 1000))
                    .unwrap_or_default();
                gpu_lines.push(info_line(
                    "  Power",
                    &format!("{}W{limit_str}", power / 1000),
                    colors,
                ));
            }
        }
    } else {
        gpu_lines.push(Line::from(""));
        gpu_lines.push(info_line("GPU", "Not detected", colors));
    }

    let mut all_lines = info_lines;
    all_lines.extend(gpu_lines);

    let info = Paragraph::new(all_lines)
        .block(
            Block::bordered()
                .title(" System Information ")
                .border_style(Style::default().fg(colors.primary)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(info, cols[0]);

    // Right: Resource summary with big gauges
    let mut right_constraints = vec![
        Constraint::Length(5), // CPU
        Constraint::Length(5), // RAM
        Constraint::Length(5), // Swap
    ];
    for _ in &app.gpus {
        right_constraints.push(Constraint::Length(5));
    }
    right_constraints.push(Constraint::Min(0)); // CPU History

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(right_constraints)
        .split(cols[1]);

    let mut chunk_idx = 0;

    // CPU
    let cpu_block = Block::bordered()
        .title(" CPU ")
        .border_style(Style::default().fg(colors.cpu));
    let cpu_inner = cpu_block.inner(right_chunks[chunk_idx]);
    frame.render_widget(cpu_block, right_chunks[chunk_idx]);
    let cpu_gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.cpu_usage_color(app.global_cpu as f64)))
        .percent((app.global_cpu as u16).min(100))
        .label(format!("{:.1}%", app.global_cpu));
    frame.render_widget(cpu_gauge, shrink_rect(cpu_inner, 1, 0));
    chunk_idx += 1;

    // RAM
    let ram_pct = if app.total_memory > 0 {
        ((app.used_memory as f64 / app.total_memory as f64) * 100.0) as u16
    } else {
        0
    };
    let ram_block = Block::bordered()
        .title(" RAM ")
        .border_style(Style::default().fg(colors.memory));
    let ram_inner = ram_block.inner(right_chunks[chunk_idx]);
    frame.render_widget(ram_block, right_chunks[chunk_idx]);
    let ram_gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.memory))
        .percent(ram_pct.min(100))
        .label(format!(
            "{} / {}",
            format_bytes(app.used_memory),
            format_bytes(app.total_memory)
        ));
    frame.render_widget(ram_gauge, shrink_rect(ram_inner, 1, 0));
    chunk_idx += 1;

    // Swap
    let swap_pct = if app.total_swap > 0 {
        ((app.used_swap as f64 / app.total_swap as f64) * 100.0) as u16
    } else {
        0
    };
    let swap_block = Block::bordered()
        .title(" Swap ")
        .border_style(Style::default().fg(colors.secondary));
    let swap_inner = swap_block.inner(right_chunks[chunk_idx]);
    frame.render_widget(swap_block, right_chunks[chunk_idx]);
    let swap_gauge = Gauge::default()
        .gauge_style(Style::default().fg(colors.secondary))
        .percent(swap_pct.min(100))
        .label(format!(
            "{} / {}",
            format_bytes(app.used_swap),
            format_bytes(app.total_swap)
        ));
    frame.render_widget(swap_gauge, shrink_rect(swap_inner, 1, 0));
    chunk_idx += 1;

    // GPU gauges
    for gpu in &app.gpus {
        let gpu_mem_pct = if gpu.memory_total > 0 {
            ((gpu.memory_used as f64 / gpu.memory_total as f64) * 100.0) as u16
        } else {
            0
        };
        let gpu_block = Block::bordered()
            .title(format!(" {} — {}°C ", gpu.name, gpu.temperature))
            .border_style(Style::default().fg(colors.accent));
        let gpu_inner = gpu_block.inner(right_chunks[chunk_idx]);
        frame.render_widget(gpu_block, right_chunks[chunk_idx]);

        let gpu_area = shrink_rect(gpu_inner, 1, 0);
        let gpu_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(gpu_area);

        let util_gauge = Gauge::default()
            .gauge_style(Style::default().fg(colors.cpu_usage_color(gpu.utilization as f64)))
            .percent(gpu.utilization.min(100) as u16)
            .label(format!("GPU: {}%", gpu.utilization));
        frame.render_widget(util_gauge, gpu_rows[0]);

        let vram_gauge = Gauge::default()
            .gauge_style(Style::default().fg(colors.memory))
            .percent(gpu_mem_pct.min(100))
            .label(format!(
                "VRAM: {} / {}",
                format_bytes(gpu.memory_used),
                format_bytes(gpu.memory_total)
            ));
        frame.render_widget(vram_gauge, gpu_rows[1]);

        chunk_idx += 1;
    }

    // CPU History
    let history_block = Block::bordered()
        .title(" CPU History (60s) ")
        .border_style(Style::default().fg(colors.cpu));
    let history_inner = history_block.inner(right_chunks[chunk_idx]);
    frame.render_widget(history_block, right_chunks[chunk_idx]);
    let data: Vec<u64> = app.global_cpu_history.iter().map(|v| *v as u64).collect();
    let sparkline = Sparkline::default()
        .data(&data)
        .max(100)
        .style(Style::default().fg(colors.cpu));
    frame.render_widget(sparkline, history_inner);
}
