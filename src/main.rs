mod client;
mod config;
mod stats;
mod tui;

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tokio::signal;
use tokio::sync::mpsc;
use crate::config::LngAppConfig;
use crate::stats::{LngMetrics, StatsCalculator};
use crate::tui::LngTui;
use crate::client::spawn_lng_client;
use crossterm::event::{self, Event, KeyCode};

#[tokio::main]
async fn main() -> Result<()> {
    // Increase File Descriptor Limit for high-concurrency
    increase_rlimit()?;

    // Load Configuration
    let config = Arc::new(LngAppConfig::build().expect("Failed to load config"));
    let metrics = Arc::new(LngMetrics::default());
    let mut stats_calc = StatsCalculator::new(Arc::clone(&metrics));

    // Initialize TUI
    let mut tui = LngTui::new()?;
    
    // Set up panic hook to ensure terminal cleanup
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        disable_raw_mode_terminal();
        original_hook(panic_info);
    }));

    // Setup event channel
    let (tx, mut rx) = mpsc::channel(100);
    std::thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    if tx.blocking_send(key).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Ramp-up connections
    let config_p = Arc::clone(&config);
    let metrics_p = Arc::clone(&metrics);
    tokio::spawn(async move {
        let mut ramp_interval = interval(Duration::from_secs(1));
        let mut spawned = 0;
        
        while spawned < config_p.connections {
            ramp_interval.tick().await;
            let batch = (config_p.connections - spawned).min(config_p.ramp_up_rate);
            
            for _ in 0..batch {
                let id = spawned;
                let c = Arc::clone(&config_p);
                let m = Arc::clone(&metrics_p);
                tokio::spawn(async move {
                    if let Err(e) = spawn_lng_client(id, c, m).await {
                        eprintln!("Failed to spawn client {}: {:?}", id, e);
                    }
                });
                spawned += 1;
            }
        }
    });

    // UI and Event Loop
    let mut ui_interval = interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            _ = ui_interval.tick() => {
                let stats = stats_calc.calculate();
                tui.draw(&stats, &config)?;
            }
            _ = signal::ctrl_c() => {
                break;
            }
            Some(key) = rx.recv() => {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    tui.cleanup()?;
    Ok(())
}

fn increase_rlimit() -> Result<()> {
    unsafe {
        let mut rlim = libc::rlimit {
            rlim_cur: 65535,
            rlim_max: 65535,
        };
        if libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) != 0 {
            // If 65535 is too high, try a more modest increase or just log it
            rlim.rlim_cur = 10240;
            rlim.rlim_max = 10240;
            libc::setrlimit(libc::RLIMIT_NOFILE, &rlim);
        }
    }
    Ok(())
}

fn disable_raw_mode_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    let mut stdout = std::io::stdout();
    let _ = crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen, crossterm::event::DisableMouseCapture);
}
