use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::TableState,
    Terminal,
};
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use sysinfo::System;

mod hardware;
mod ui;
mod prices;
use hardware::{HardwareInfo, ProcessInfo};

pub struct App {
    pub hw: HardwareInfo,
    pub sys: System,
    pub processes: Vec<ProcessInfo>,
    pub table_state: TableState,
    pub scroll: usize,
    pub cpu_history: Vec<Vec<f32>>, // Per-core history
    pub mem_history: Vec<f32>,       // Memory usage history
    pub net_rx_history: Vec<f64>,    // Network receive history (MB/s)
    pub net_tx_history: Vec<f64>,    // Network transmit history (MB/s)
    pub last_net_rx: u64,
    pub last_net_tx: u64,
    pub history_size: usize,
    pub stress_running: bool,
    pub stress_cpu_threads: usize,
    pub stress_mem_bytes: usize,
    pub stress_started_at: Option<Instant>,
    pub stress_max_secs: u64,
    stress_state: Option<StressState>,
}

struct StressState {
    stop: Arc<AtomicBool>,
    handles: Vec<thread::JoinHandle<()>>,
}

impl App {
    fn new() -> Self {
        // Fetch/load prices (will check if we need to update today)
        let _current_prices = prices::HardwarePrices::load_or_fetch();

        let mut sys = System::new_all();
        sys.refresh_all();

        let hw = HardwareInfo::new(&sys);
        let num_cpus = sys.cpus().len();
        let history_size = 60; // Keep 60 samples

        // Initialize history buffers
        let cpu_history = vec![vec![0.0; history_size]; num_cpus];
        let mem_history = vec![0.0; history_size];
        let net_rx_history = vec![0.0; history_size];
        let net_tx_history = vec![0.0; history_size];

        // Get initial network stats
        let networks = sysinfo::Networks::new_with_refreshed_list();
        let (rx, tx) = networks.iter()
            .fold((0u64, 0u64), |(rx_acc, tx_acc), (_, data)| {
                (rx_acc + data.total_received(), tx_acc + data.total_transmitted())
            });

        Self {
            hw,
            sys,
            processes: Vec::new(),
            table_state: TableState::default(),
            scroll: 0,
            cpu_history,
            mem_history,
            net_rx_history,
            net_tx_history,
            last_net_rx: rx,
            last_net_tx: tx,
            history_size,
            stress_running: false,
            stress_cpu_threads: 0,
            stress_mem_bytes: 0,
            stress_started_at: None,
            stress_max_secs: 60,
            stress_state: None,
        }
    }

    fn update(&mut self) {
        if self.stress_running {
            if let Some(remaining) = self.stress_remaining_secs() {
                if remaining == 0 {
                    self.stop_stress();
                }
            }
        }

        // Refresh system info
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.sys.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, sysinfo::ProcessRefreshKind::everything());

        // Update CPU history
        for (i, cpu) in self.sys.cpus().iter().enumerate() {
            if i < self.cpu_history.len() {
                self.cpu_history[i].remove(0);
                self.cpu_history[i].push(cpu.cpu_usage());
            }
        }

        // Update memory history
        let mem_percent = (self.sys.used_memory() as f64 / self.sys.total_memory() as f64 * 100.0) as f32;
        self.mem_history.remove(0);
        self.mem_history.push(mem_percent);

        // Update network history
        let networks = sysinfo::Networks::new_with_refreshed_list();
        let (rx, tx) = networks.iter()
            .fold((0u64, 0u64), |(rx_acc, tx_acc), (_, data)| {
                (rx_acc + data.total_received(), tx_acc + data.total_transmitted())
            });

        // Calculate rate (bytes per second, convert to MB/s)
        let rx_rate = (rx.saturating_sub(self.last_net_rx)) as f64 / 1_048_576.0; // MB/s
        let tx_rate = (tx.saturating_sub(self.last_net_tx)) as f64 / 1_048_576.0; // MB/s

        self.net_rx_history.remove(0);
        self.net_rx_history.push(rx_rate);
        self.net_tx_history.remove(0);
        self.net_tx_history.push(tx_rate);

        self.last_net_rx = rx;
        self.last_net_tx = tx;

        // Update process list
        self.processes = self.hw.get_top_processes(&self.sys, 100);
    }

    fn scroll_down(&mut self) {
        if self.scroll < self.processes.len().saturating_sub(1) {
            self.scroll += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    fn toggle_stress(&mut self) {
        if self.stress_running {
            self.stop_stress();
        } else {
            self.start_stress();
        }
    }

    fn start_stress(&mut self) {
        if self.stress_running {
            return;
        }

        let total_kb = self.sys.total_memory() as u64;
        let total_bytes = total_kb.saturating_mul(1024);
        let mut mem_bytes = (total_bytes / 4) as usize; // 25% of RAM
        let min_bytes = 256usize * 1024 * 1024;
        let max_bytes = 1024usize * 1024 * 1024;
        mem_bytes = mem_bytes.clamp(min_bytes, max_bytes);

        let cpu_threads = self.sys.cpus().len().max(1);
        let stop = Arc::new(AtomicBool::new(false));
        let mut handles = Vec::new();

        for _ in 0..cpu_threads {
            let stop_flag = stop.clone();
            handles.push(thread::spawn(move || {
                let mut x = 0.0f64;
                while !stop_flag.load(Ordering::Relaxed) {
                    x = (x + 1.0).sin().cos();
                    std::hint::black_box(x);
                }
            }));
        }

        let mem = Arc::new(Mutex::new(vec![0u8; mem_bytes]));
        let mem_stop = stop.clone();
        let mem_ref = mem.clone();
        handles.push(thread::spawn(move || {
            let mut idx = 0usize;
            while !mem_stop.load(Ordering::Relaxed) {
                if let Ok(mut buf) = mem_ref.lock() {
                    if buf.is_empty() {
                        drop(buf);
                        thread::yield_now();
                        continue;
                    }
                    let len = buf.len();
                    let chunk = 1024 * 1024;
                    let end = (idx + chunk).min(len);
                    for b in &mut buf[idx..end] {
                        *b = b.wrapping_add(1);
                    }
                    idx = if end >= len { 0 } else { end };
                }
            }
        }));

        self.stress_running = true;
        self.stress_cpu_threads = cpu_threads;
        self.stress_mem_bytes = mem_bytes;
        self.stress_started_at = Some(Instant::now());
        self.stress_state = Some(StressState {
            stop,
            handles,
        });
    }

    fn stop_stress(&mut self) {
        if let Some(state) = self.stress_state.take() {
            state.stop.store(true, Ordering::Relaxed);
            for handle in state.handles {
                let _ = handle.join();
            }
        }
        self.stress_running = false;
        self.stress_cpu_threads = 0;
        self.stress_mem_bytes = 0;
        self.stress_started_at = None;
    }

    pub fn stress_remaining_secs(&self) -> Option<u64> {
        if !self.stress_running {
            return None;
        }
        let start = self.stress_started_at?;
        let elapsed = start.elapsed().as_secs();
        if elapsed >= self.stress_max_secs {
            Some(0)
        } else {
            Some(self.stress_max_secs - elapsed)
        }
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();
    app.update();

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let mut last_update = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                        KeyCode::Char('r') => app.update(),
                        KeyCode::Char('t') => app.toggle_stress(),
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
        }

        // Auto-update every second
        if last_update.elapsed() >= Duration::from_secs(1) {
            app.update();
            last_update = std::time::Instant::now();
        }
    }
}
