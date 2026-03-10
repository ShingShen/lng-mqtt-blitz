use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};
use std::io;
use crate::stats::ThroughputStats;
use crate::config::LngAppConfig;

pub struct LngTui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl LngTui {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn draw(&mut self, stats: &ThroughputStats, config: &LngAppConfig) -> io::Result<()> {
        self.terminal.draw(|f| {
            render_ui(f, stats, config);
        })?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

fn render_ui(f: &mut Frame, stats: &ThroughputStats, config: &LngAppConfig) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.size());

    // Connection Progress
    let progress = if config.connections > 0 {
        (stats.active_conns as f64 / config.connections as f64).min(1.0)
    } else {
        1.0
    };
    let gauge = Gauge::default()
        .block(Block::default().title("Connection Progress").borders(Borders::ALL))
        .gauge_style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
        .percent((progress * 100.0) as u16);
    f.render_widget(gauge, chunks[0]);

    // Throughput stats
    let text = format!(
        "Sent: {} | Recv: {} | Errors: {} | Msg/sec: {:.2} | Active: {}/{}",
        stats.sent_total, stats.recv_total, stats.errors_total, stats.msg_per_sec, stats.active_conns, config.connections
    );
    let p = Paragraph::new(text)
        .block(Block::default().title("Throughput Dashboard").borders(Borders::ALL));
    f.render_widget(p, chunks[1]);

    // Footer info
    let footer = Paragraph::new("Press 'q' to quit | 'lng-mqtt-blitz' production-ready MQTT Stress Tester")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
