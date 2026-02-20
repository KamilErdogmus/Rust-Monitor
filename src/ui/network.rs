use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Cell, Row, Sparkline, Table},
    Frame,
};

use crate::app::{format_bytes, App};
use crate::theme::ThemeColors;

pub fn draw_network_detail(frame: &mut Frame, app: &App, colors: &ThemeColors, area: Rect) {
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
                Cell::from(format_bytes(iface.transmitted))
                    .style(Style::default().fg(colors.warning)),
                Cell::from(iface.packets_in.to_string()),
                Cell::from(iface.packets_out.to_string()),
                Cell::from(iface.errors_in.to_string()).style(if iface.errors_in > 0 {
                    Style::default().fg(colors.danger)
                } else {
                    Style::default().fg(colors.text_dim)
                }),
                Cell::from(iface.errors_out.to_string()).style(if iface.errors_out > 0 {
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
