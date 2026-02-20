use sysinfo::{Disks, Networks, Pid, Signal, System};
use std::collections::VecDeque;
use std::time::Instant;

const HISTORY_LEN: usize = 60;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Overview,
    Processes,
    SystemInfo,
    NetworkDetail,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Overview, Tab::Processes, Tab::SystemInfo, Tab::NetworkDetail]
    }

    pub fn index(self) -> usize {
        match self {
            Tab::Overview => 0,
            Tab::Processes => 1,
            Tab::SystemInfo => 2,
            Tab::NetworkDetail => 3,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Tab::Overview => " Overview ",
            Tab::Processes => " Processes ",
            Tab::SystemInfo => " System ",
            Tab::NetworkDetail => " Network ",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SortBy {
    Cpu,
    Memory,
    Name,
    Pid,
}

impl SortBy {
    pub fn label(self) -> &'static str {
        match self {
            SortBy::Cpu => "CPU ▼",
            SortBy::Memory => "MEM ▼",
            SortBy::Name => "NAME ▼",
            SortBy::Pid => "PID ▼",
        }
    }

    pub fn next(self) -> Self {
        match self {
            SortBy::Cpu => SortBy::Memory,
            SortBy::Memory => SortBy::Name,
            SortBy::Name => SortBy::Pid,
            SortBy::Pid => SortBy::Cpu,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Theme {
    Default,
    Ocean,
    Forest,
    Sunset,
}

impl Theme {
    pub fn next(self) -> Self {
        match self {
            Theme::Default => Theme::Ocean,
            Theme::Ocean => Theme::Forest,
            Theme::Forest => Theme::Sunset,
            Theme::Sunset => Theme::Default,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Theme::Default => "Default",
            Theme::Ocean => "Ocean",
            Theme::Forest => "Forest",
            Theme::Sunset => "Sunset",
        }
    }
}

pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory: u64,
    pub status: String,
    pub run_time: u64,
    pub disk_read: u64,
    pub disk_write: u64,
}

pub struct NetworkInterface {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub packets_in: u64,
    pub packets_out: u64,
    pub errors_in: u64,
    pub errors_out: u64,
    pub mac_address: String,
}

pub struct App {
    pub system: System,
    pub disks: Disks,
    pub networks: Networks,

    // History data
    pub cpu_history: Vec<VecDeque<f64>>,
    pub global_cpu_history: VecDeque<f64>,
    pub mem_history: VecDeque<f64>,
    pub net_rx_history: VecDeque<f64>,
    pub net_tx_history: VecDeque<f64>,

    // Current stat
    pub processes: Vec<ProcessInfo>,
    pub network_interfaces: Vec<NetworkInterface>,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub cpu_count: usize,
    pub global_cpu: f32,
    pub net_rx: u64,
    pub net_tx: u64,

    // System info
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_brand: String,
    pub cpu_arch: String,
    pub boot_time: u64,
    pub start_time: Instant,

    // UI state
    pub active_tab: Tab,
    pub sort_by: SortBy,
    pub process_scroll: usize,
    pub network_scroll: usize,
    pub input_mode: InputMode,
    pub search_query: String,
    pub filtered_processes: Vec<usize>,
    pub theme: Theme,
    pub show_help: bool,
    pub kill_confirm: Option<u32>,
    pub status_message: Option<(String, Instant)>,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let cpu_count = system.cpus().len();

        let cpu_brand = system
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".into());

        let mut app = Self {
            hostname: System::host_name().unwrap_or_else(|| "Unknown".into()),
            os_name: System::name().unwrap_or_else(|| "Unknown".into()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".into()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".into()),
            cpu_brand,
            cpu_arch: std::env::consts::ARCH.to_string(),
            boot_time: System::boot_time(),
            start_time: Instant::now(),

            system,
            disks,
            networks,
            cpu_history: vec![VecDeque::from(vec![0.0; HISTORY_LEN]); cpu_count],
            global_cpu_history: VecDeque::from(vec![0.0; HISTORY_LEN]),
            mem_history: VecDeque::from(vec![0.0; HISTORY_LEN]),
            net_rx_history: VecDeque::from(vec![0.0; HISTORY_LEN]),
            net_tx_history: VecDeque::from(vec![0.0; HISTORY_LEN]),
            processes: Vec::new(),
            network_interfaces: Vec::new(),
            total_memory: 0,
            used_memory: 0,
            total_swap: 0,
            used_swap: 0,
            cpu_count,
            global_cpu: 0.0,
            net_rx: 0,
            net_tx: 0,

            active_tab: Tab::Overview,
            sort_by: SortBy::Cpu,
            process_scroll: 0,
            network_scroll: 0,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            filtered_processes: Vec::new(),
            theme: Theme::Default,
            show_help: false,
            kill_confirm: None,
            status_message: None,
            tick_count: 0,
        };
        app.update_stats();
        app
    }

    pub fn tick(&mut self) {
        self.system.refresh_all();
        self.networks.refresh(true);
        self.disks.refresh(true);
        self.update_stats();
        self.tick_count += 1;

        if let Some((_, time)) = &self.status_message {
            if time.elapsed().as_secs() >= 3 {
                self.status_message = None;
            }
        }
    }

    fn update_stats(&mut self) {
        self.global_cpu = self.system.global_cpu_usage();
        self.global_cpu_history.pop_front();
        self.global_cpu_history.push_back(self.global_cpu as f64);

        for (i, cpu) in self.system.cpus().iter().enumerate() {
            if i < self.cpu_history.len() {
                self.cpu_history[i].pop_front();
                self.cpu_history[i].push_back(cpu.cpu_usage() as f64);
            }
        }

        self.total_memory = self.system.total_memory();
        self.used_memory = self.system.used_memory();
        self.total_swap = self.system.total_swap();
        self.used_swap = self.system.used_swap();
        let mem_pct = if self.total_memory > 0 {
            (self.used_memory as f64 / self.total_memory as f64) * 100.0
        } else {
            0.0
        };
        self.mem_history.pop_front();
        self.mem_history.push_back(mem_pct);

        let (mut rx, mut tx) = (0u64, 0u64);
        self.network_interfaces.clear();
        for (name, data) in self.networks.iter() {
            rx += data.received();
            tx += data.transmitted();
            self.network_interfaces.push(NetworkInterface {
                name: name.to_string(),
                received: data.received(),
                transmitted: data.transmitted(),
                packets_in: data.packets_received(),
                packets_out: data.packets_transmitted(),
                errors_in: data.errors_on_received(),
                errors_out: data.errors_on_transmitted(),
                mac_address: data.mac_address().to_string(),
            });
        }
        self.net_rx = rx;
        self.net_tx = tx;
        self.net_rx_history.pop_front();
        self.net_rx_history.push_back(rx as f64 / 1024.0);
        self.net_tx_history.pop_front();
        self.net_tx_history.push_back(tx as f64 / 1024.0);

        self.processes = self
            .system
            .processes()
            .iter()
            .map(|(pid, proc_)| ProcessInfo {
                pid: pid.as_u32(),
                name: proc_.name().to_string_lossy().to_string(),
                cpu: proc_.cpu_usage(),
                memory: proc_.memory(),
                status: format!("{:?}", proc_.status()),
                run_time: proc_.run_time(),
                disk_read: proc_.disk_usage().read_bytes,
                disk_write: proc_.disk_usage().written_bytes,
            })
            .collect();

        self.sort_processes();
        self.update_filtered();
    }

    fn sort_processes(&mut self) {
        match self.sort_by {
            SortBy::Cpu => self.processes.sort_by(|a, b| {
                b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal)
            }),
            SortBy::Memory => self.processes.sort_by(|a, b| b.memory.cmp(&a.memory)),
            SortBy::Name => self.processes.sort_by(|a, b| {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }),
            SortBy::Pid => self.processes.sort_by(|a, b| a.pid.cmp(&b.pid)),
        }
    }

    fn update_filtered(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_processes = (0..self.processes.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_processes = self
                .processes
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.name.to_lowercase().contains(&query)
                        || p.pid.to_string().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    pub fn next_tab(&mut self) {
        let tabs = Tab::all();
        let idx = self.active_tab.index();
        self.active_tab = tabs[(idx + 1) % tabs.len()];
    }

    pub fn prev_tab(&mut self) {
        let tabs = Tab::all();
        let idx = self.active_tab.index();
        self.active_tab = tabs[(idx + tabs.len() - 1) % tabs.len()];
    }

    pub fn scroll_down(&mut self) {
        match self.active_tab {
            Tab::Processes => {
                let max = self.filtered_processes.len().saturating_sub(1);
                if self.process_scroll < max {
                    self.process_scroll += 1;
                }
            }
            Tab::NetworkDetail => {
                let max = self.network_interfaces.len().saturating_sub(1);
                if self.network_scroll < max {
                    self.network_scroll += 1;
                }
            }
            _ => {}
        }
    }

    pub fn scroll_up(&mut self) {
        match self.active_tab {
            Tab::Processes => {
                self.process_scroll = self.process_scroll.saturating_sub(1);
            }
            Tab::NetworkDetail => {
                self.network_scroll = self.network_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn page_down(&mut self) {
        for _ in 0..10 {
            self.scroll_down();
        }
    }

    pub fn page_up(&mut self) {
        for _ in 0..10 {
            self.scroll_up();
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.process_scroll = 0;
        self.network_scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        match self.active_tab {
            Tab::Processes => {
                self.process_scroll = self.filtered_processes.len().saturating_sub(1);
            }
            Tab::NetworkDetail => {
                self.network_scroll = self.network_interfaces.len().saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn toggle_sort(&mut self) {
        self.sort_by = self.sort_by.next();
        self.sort_processes();
        self.update_filtered();
    }

    pub fn toggle_theme(&mut self) {
        self.theme = self.theme.next();
        self.set_status(format!("Theme: {}", self.theme.label()));
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn enter_search(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_query.clear();
    }

    pub fn exit_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_query.clear();
        self.update_filtered();
    }

    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.process_scroll = 0;
        self.update_filtered();
    }

    pub fn search_pop(&mut self) {
        self.search_query.pop();
        self.update_filtered();
    }

    pub fn request_kill(&mut self) {
        if self.active_tab != Tab::Processes {
            return;
        }
        if let Some(&idx) = self.filtered_processes.get(self.process_scroll) {
            if let Some(proc) = self.processes.get(idx) {
                self.kill_confirm = Some(proc.pid);
            }
        }
    }

    pub fn confirm_kill(&mut self) {
        if let Some(pid) = self.kill_confirm.take() {
            let sysinfo_pid = Pid::from_u32(pid);
            if let Some(process) = self.system.process(sysinfo_pid) {
                if process.kill_with(Signal::Term).unwrap_or(false) {
                    self.set_status(format!("Sent SIGTERM to PID {pid}"));
                } else if process.kill() {
                    self.set_status(format!("Killed PID {pid}"));
                } else {
                    self.set_status(format!("Failed to kill PID {pid}"));
                }
            } else {
                self.set_status(format!("Process {pid} not found"));
            }
        }
    }

    pub fn cancel_kill(&mut self) {
        self.kill_confirm = None;
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn uptime_str(&self) -> String {
        let sys_uptime = System::uptime();
        format_duration(sys_uptime)
    }

    pub fn monitor_uptime_str(&self) -> String {
        format_duration(self.start_time.elapsed().as_secs())
    }

    pub fn boot_time_str(&self) -> String {
        let secs_since_epoch = self.boot_time;
        let uptime = System::uptime();
        format!("{}s ago (uptime: {})", secs_since_epoch, format_duration(uptime))
    }

    pub fn selected_process(&self) -> Option<&ProcessInfo> {
        self.filtered_processes
            .get(self.process_scroll)
            .and_then(|&idx| self.processes.get(idx))
    }
}

pub fn format_duration(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
