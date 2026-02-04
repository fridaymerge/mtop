use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::App;

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar
            Constraint::Min(20),     // Main content
        ])
        .split(f.area());

    // Title
    render_title(f, chunks[0], app);

    // Main layout: 3 panels on left, processes on right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65), // Left side (3 panels)
            Constraint::Percentage(35), // Right side (processes)
        ])
        .split(chunks[1]);

    // Left side: 3 panels stacked
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // CPU
            Constraint::Percentage(30), // Memory
            Constraint::Percentage(30), // Network
        ])
        .split(main_chunks[0]);

    // Render panels
    render_cpu_panel(f, left_chunks[0], app);
    render_memory_panel(f, left_chunks[1], app);
    render_network_panel(f, left_chunks[2], app);

    // Render process table
    render_process_panel(f, main_chunks[1], app);
}

fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let total_cost = app.hw.cpu_cost + app.hw.ram_cost + app.hw.disk_cost;
    let cpu_percent = app.sys.global_cpu_usage();
    let power_data = app.hw.get_power_consumption(cpu_percent);

    // Calculate active cost
    let mem = app.sys.used_memory() as f64 / app.sys.total_memory() as f64;
    let cpu_active = (cpu_percent as f64 / 100.0) * app.hw.cpu_cost;
    let mem_active = mem * app.hw.ram_cost;
    let total_active = cpu_active + mem_active;

    let title = Line::from(vec![
        Span::styled("mtop ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled("Resource Cost Monitor", Style::default().fg(Color::Gray)),
        Span::styled(format!("  │  Hardware: ${:.2}", total_cost), Style::default().fg(Color::White)),
        Span::styled(format!("  │  Active: ${:.2}", total_active), Style::default().fg(Color::Cyan)),
        Span::styled(format!("  │  Power: {:.1}W (${:.2}/hr)", power_data.0, power_data.1 * 3600.0), Style::default().fg(Color::Gray)),
    ]);

    let status_line = if app.stress_running {
        let remaining = app.stress_remaining_secs().unwrap_or(0);
        let mins = remaining / 60;
        let secs = remaining % 60;
        Line::from(vec![
            Span::styled("STRESS TEST RUNNING ", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
            Span::styled(format!("({:01}:{:02} remaining, press 't' to stop)", mins, secs), Style::default().fg(Color::LightRed)),
        ])
    } else {
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("'t'", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" to run 1m stress test", Style::default().fg(Color::DarkGray)),
        ])
    };

    let para = Paragraph::new(vec![title, status_line])
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(para, area);
}

fn render_cpu_panel(f: &mut Frame, area: Rect, app: &App) {
    let cpu_percent = app.sys.global_cpu_usage();
    let used_cost = (cpu_percent as f64 / 100.0) * app.hw.cpu_cost;
    let cost_color = cost_color(used_cost, app.hw.cpu_cost);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(cost_color));

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let cpu_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // title + spacing + total
            Constraint::Min(0),    // per-core list
        ])
        .split(inner);

    let header_lines = vec![
        Line::from(vec![
            Span::styled("CPU COST", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  ${:.2}", used_cost),
                Style::default().fg(cost_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" / ${:.2}", app.hw.cpu_cost),
                Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
    ];

    f.render_widget(Paragraph::new(header_lines), cpu_chunks[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(cpu_chunks[1]);

    let cpus = app.sys.cpus();
    let cost_per_core = app.hw.cpu_cost / cpus.len() as f64;
    let split = (cpus.len() + 1) / 2;

    let spark_len = {
        let col_width = columns[0].width.min(columns[1].width) as usize;
        let fixed = 12usize; // "C00 " + space + cost field
        let raw = col_width.saturating_sub(fixed);
        raw.clamp(8, 20)
    };

    let mut left_lines = Vec::new();
    let mut right_lines = Vec::new();

    for (i, cpu) in cpus.iter().enumerate() {
        let core_usage = cpu.cpu_usage();
        let core_cost = (core_usage as f64 / 100.0) * cost_per_core;
        let color = if core_cost > cost_per_core * 0.75 {
            Color::Red
        } else if core_cost > cost_per_core * 0.5 {
            Color::Yellow
        } else {
            Color::Green
        };

        let sparkline = if i < app.cpu_history.len() {
            render_sparkline(&app.cpu_history[i], spark_len)
        } else {
            " ".repeat(spark_len)
        };

        let line = Line::from(vec![
            Span::styled(format!("C{:02} ", i), Style::default().fg(Color::Cyan)),
            Span::styled(sparkline, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(format!("${:>5.2}", core_cost), Style::default().fg(color)),
        ]);

        if i < split {
            left_lines.push(line);
        } else {
            right_lines.push(line);
        }
    }

    f.render_widget(Paragraph::new(left_lines), columns[0]);
    f.render_widget(Paragraph::new(right_lines), columns[1]);
}

fn render_memory_panel(f: &mut Frame, area: Rect, app: &App) {
    let total_mem = app.sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_mem = app.sys.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let mem_percent = (used_mem / total_mem * 100.0) as f32;
    let used_cost = (mem_percent as f64 / 100.0) * app.hw.ram_cost;
    let cost_color = cost_color(used_cost, app.hw.ram_cost);
    let cost_per_gb = app.hw.ram_cost / total_mem;

    let mut lines = vec![];

    // Title with pricing info
    lines.push(Line::from(vec![
        Span::styled("RAM COST ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(format!("(${:.2}/GB)", cost_per_gb), Style::default().fg(Color::Gray)),
    ]));
    lines.push(Line::from(""));

    // Big dollar amount
    lines.push(Line::from(vec![
        Span::styled(format!("  ${:.2}", used_cost),
            Style::default().fg(cost_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" / ${:.2}", app.hw.ram_cost),
            Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    // Sparkline (full width) + dotted midline
    let sparkline_width = area.width.saturating_sub(2 + 2) as usize; // borders + indent
    let sparkline = render_sparkline(&app.mem_history, sparkline_width);
    let midline = render_dot_line(sparkline_width);
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(sparkline, Style::default().fg(cost_color)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(midline, Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    // Memory stats
    lines.push(Line::from(vec![
        Span::styled(format!("Using: {:.1} GB", used_mem), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(format!("Total: {:.1} GB", total_mem), Style::default().fg(Color::DarkGray)),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(cost_color));

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

fn render_network_panel(f: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![];

    // Current rates
    let current_rx = app.net_rx_history.last().unwrap_or(&0.0);
    let current_tx = app.net_tx_history.last().unwrap_or(&0.0);
    let total_rate = current_rx + current_tx;

    // Title
    lines.push(Line::from(vec![
        Span::styled("NETWORK ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:.2} MB/s", total_rate), Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(""));

    // Sparklines (full width) + dotted midline
    let inner_width = area.width.saturating_sub(2) as usize;
    let rx_value = format!(" {:.2}", current_rx);
    let tx_value = format!(" {:.2}", current_tx);
    let rx_width = inner_width.saturating_sub(3 + rx_value.chars().count()); // "RX " + value
    let tx_width = inner_width.saturating_sub(3 + tx_value.chars().count()); // "TX " + value
    let rx_sparkline = render_sparkline_f64(&app.net_rx_history, rx_width);
    let tx_sparkline = render_sparkline_f64(&app.net_tx_history, tx_width);
    let rx_midline = render_dot_line(rx_width);
    let tx_midline = render_dot_line(tx_width);

    lines.push(Line::from(vec![
        Span::styled("RX ", Style::default().fg(Color::Gray)),
        Span::styled(rx_sparkline, Style::default().fg(Color::Cyan)),
        Span::styled(rx_value, Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("   ", Style::default()),
        Span::styled(rx_midline, Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("TX ", Style::default().fg(Color::Gray)),
        Span::styled(tx_sparkline, Style::default().fg(Color::Blue)),
        Span::styled(tx_value, Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("   ", Style::default()),
        Span::styled(tx_midline, Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    // Top 5 network consumers
    lines.push(Line::from(vec![
        Span::styled("Top Consumers:", Style::default().fg(Color::Gray)),
    ]));

    let mut net_procs: Vec<_> = app.processes.iter()
        .filter(|p| p.net_connections > 0)
        .collect();
    net_procs.sort_by(|a, b| b.net_connections.cmp(&a.net_connections));

    for proc in net_procs.iter().take(5) {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:2} ", proc.net_connections), Style::default().fg(Color::Cyan)),
            Span::styled(format!("{}", proc.name), Style::default().fg(Color::White)),
        ]));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}

fn render_process_panel(f: &mut Frame, area: Rect, app: &App) {
    use ratatui::widgets::{Row, Table};

    let header = Row::new(vec!["Process", "Net", "💰 $/s"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.processes.iter()
        .skip(app.scroll)
        .take(area.height.saturating_sub(3) as usize)
        .map(|p| {
            let cost_color = if p.total_cost > 100.0 {
                Color::Red
            } else if p.total_cost > 10.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            // Network indicator (just number, no emoji)
            let net_indicator = if p.net_connections > 0 {
                format!("{}", p.net_connections)
            } else {
                "".to_string()
            };

            Row::new(vec![
                format!("{}", p.name),
                net_indicator,
                format!("${:.2}", p.total_cost),
            ]).style(Style::default().fg(cost_color))
        })
        .collect();

    let widths = [
        Constraint::Percentage(60),
        Constraint::Percentage(15),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Top Processes by Cost")
            .border_style(Style::default().fg(Color::Blue)));

    f.render_widget(table, area);
}

// Remove the old cost_summary function since it's now in the header

fn cost_color(used: f64, total: f64) -> Color {
    let percent = (used / total) * 100.0;
    if percent > 75.0 {
        Color::LightRed
    } else if percent > 50.0 {
        Color::LightYellow
    } else {
        Color::Cyan
    }
}

fn render_sparkline(history: &[f32], width: usize) -> String {
    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    // Take last 'width' samples
    let start = if history.len() > width {
        history.len() - width
    } else {
        0
    };

    let mut spark: String = history[start..]
        .iter()
        .map(|&usage| {
            let index = ((usage / 100.0) * (bars.len() - 1) as f32) as usize;
            bars[index.min(bars.len() - 1)]
        })
        .collect();

    if spark.len() < width {
        spark.push_str(&"·".repeat(width - spark.len()));
    }

    spark
}

fn render_sparkline_f64(history: &[f64], width: usize) -> String {
    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    // Take last 'width' samples
    let start = if history.len() > width {
        history.len() - width
    } else {
        0
    };

    // Find max for scaling
    let max = history[start..].iter().cloned().fold(0.0f64, f64::max).max(0.1);

    let mut spark: String = history[start..]
        .iter()
        .map(|&value| {
            let normalized = (value / max).min(1.0);
            let index = (normalized * (bars.len() - 1) as f64) as usize;
            bars[index.min(bars.len() - 1)]
        })
        .collect();

    if spark.len() < width {
        spark.push_str(&"·".repeat(width - spark.len()));
    }

    spark
}

fn render_dot_line(width: usize) -> String {
    "·".repeat(width)
}
