use sysinfo::{Disks, Networks, Pid, Signal, System};
use std::collections::VecDeque;
use std::time::Instant;
use nvml_wrapper::Nvml;

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

pub struct GpuInfo {
    pub name: String,
    pub temperature: u32,
    pub utilization: u32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub fan_speed: Option<u32>,
    pub power_usage: Option<u32>,
    pub power_limit: Option<u32>,
}

pub struct ProcessDetail {
    pub base: ProcessInfo,
    pub parent_pid: Option<u32>,
    pub cmd: String,
    pub exe: String,
    pub root: String,
    pub environ_count: usize,
    pub threads: Option<u64>,
    pub virtual_memory: u64,
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
    pub show_process_detail: bool,
    pub process_detail: Option<ProcessDetail>,
    pub nvml: Option<Nvml>,
    pub gpus: Vec<GpuInfo>,
    pub gpu_util_history: Vec<VecDeque<f64>>,
    #[cfg(target_os = "macos")]
    pub apple_gpu_sampler: Option<crate::macos_gpu::AppleGpuSampler>,
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
            show_process_detail: false,
            process_detail: None,
            nvml: Nvml::init().ok(),
            gpus: Vec::new(),
            gpu_util_history: Vec::new(),
            #[cfg(target_os = "macos")]
            apple_gpu_sampler: crate::macos_gpu::AppleGpuSampler::new(),
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

        if let Some((_, time)) = &self.status_message
            && time.elapsed().as_secs() >= 3
        {
            self.status_message = None;
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
        self.update_gpu();
    }

    fn update_gpu(&mut self) {
        // Try NVML first (NVIDIA GPUs on all platforms)
        if let Some(nvml) = &self.nvml
            && let Ok(count) = nvml.device_count()
        {
                self.gpus.clear();
                for i in 0..count {
                    let device = match nvml.device_by_index(i) {
                        Ok(d) => d,
                        Err(_) => continue,
                    };

                    let name = device.name().unwrap_or_else(|_| "Unknown GPU".into());
                    let temperature = device
                        .temperature(
                            nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu,
                        )
                        .unwrap_or(0);
                    let utilization =
                        device.utilization_rates().map(|u| u.gpu).unwrap_or(0);
                    let memory = device.memory_info().ok();
                    let memory_used = memory.as_ref().map(|m| m.used).unwrap_or(0);
                    let memory_total = memory.as_ref().map(|m| m.total).unwrap_or(0);
                    let fan_speed = device.fan_speed(0).ok();
                    let power_usage = device.power_usage().ok();
                    let power_limit = device.enforced_power_limit().ok();

                    self.gpus.push(GpuInfo {
                        name,
                        temperature,
                        utilization,
                        memory_used,
                        memory_total,
                        fan_speed,
                        power_usage,
                        power_limit,
                    });

                    while self.gpu_util_history.len() <= i as usize {
                        self.gpu_util_history
                            .push(VecDeque::from(vec![0.0; HISTORY_LEN]));
                    }
                    self.gpu_util_history[i as usize].pop_front();
                    self.gpu_util_history[i as usize].push_back(utilization as f64);
                }
                if !self.gpus.is_empty() {
                    return;
                }
        }

        // Fallback: platform-specific GPU detection
        self.detect_platform_gpu();
    }

    fn detect_platform_gpu(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.detect_macos_gpu();
        }
        #[cfg(target_os = "linux")]
        {
            self.detect_linux_gpu();
        }
        // Windows without NVML: no fallback (AMD/Intel don't expose easy APIs)
    }

    #[cfg(target_os = "macos")]
    fn detect_macos_gpu(&mut self) {
        // Use IOReport sampler for real-time metrics
        if let Some(sampler) = &mut self.apple_gpu_sampler {
            if let Some(metrics) = sampler.sample() {
                // Get a nice GPU name from system_profiler
                let gpu_name = if metrics.gpu_name == "Apple GPU" {
                    crate::macos_gpu::get_apple_gpu_name()
                } else {
                    metrics.gpu_name
                };

                // Convert power from milliwatts to the same unit as NVML (milliwatts)
                let power_usage = metrics.power_mw;

                self.gpus.clear();
                self.gpus.push(GpuInfo {
                    name: gpu_name,
                    temperature: metrics.temperature,
                    utilization: metrics.utilization,
                    memory_used: 0,  // Apple Silicon uses unified memory
                    memory_total: 0, // No separate VRAM
                    fan_speed: None,
                    power_usage,
                    power_limit: None,
                });

                if self.gpu_util_history.is_empty() {
                    self.gpu_util_history
                        .push(VecDeque::from(vec![0.0; HISTORY_LEN]));
                }
                self.gpu_util_history[0].pop_front();
                self.gpu_util_history[0].push_back(metrics.utilization as f64);
                return;
            }
        }

        // Fallback: just get GPU name from system_profiler
        let name = crate::macos_gpu::get_apple_gpu_name();
        if !self.gpus.iter().any(|g| g.name == name) {
            self.gpus.push(GpuInfo {
                name,
                temperature: 0,
                utilization: 0,
                memory_used: 0,
                memory_total: 0,
                fan_speed: None,
                power_usage: None,
                power_limit: None,
            });
        }
    }

    #[cfg(target_os = "linux")]
    fn detect_linux_gpu(&mut self) {
        use std::fs;
        use std::path::Path;
        use std::process::Command;

        let drm_path = Path::new("/sys/class/drm");
        if !drm_path.exists() {
            return;
        }

        // Build a PCI slot → human-readable name map from lspci
        let gpu_names = Command::new("lspci")
            .output()
            .ok()
            .map(|out| {
                let text = String::from_utf8_lossy(&out.stdout);
                text.lines()
                    .filter(|l| {
                        l.contains("VGA") || l.contains("3D") || l.contains("Display")
                    })
                    .filter_map(|l| {
                        let slot = l.split_whitespace().next()?;
                        // Line format: "01:00.0 VGA compatible controller: AMD ... [Radeon ...]"
                        let name = l.splitn(2, ": ").nth(1)?;
                        // Take the part after the second ": " (vendor: product)
                        let product = name.splitn(2, ": ").nth(1).unwrap_or(name);
                        Some((slot.to_string(), product.to_string()))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let entries = match fs::read_dir(drm_path) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name_str = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Only look at card* directories (not renderD* or card0-HDMI-A-1 etc.)
            if !name_str.starts_with("card") || name_str.contains('-') {
                continue;
            }

            let device_path = path.join("device");
            if !device_path.exists() {
                continue;
            }

            // Get PCI slot from uevent, then match to lspci name
            let pci_slot = fs::read_to_string(device_path.join("uevent"))
                .ok()
                .and_then(|content| {
                    content
                        .lines()
                        .find(|l| l.starts_with("PCI_SLOT_NAME="))
                        .map(|l| {
                            l.trim_start_matches("PCI_SLOT_NAME=")
                                .trim_start_matches("0000:")
                                .to_string()
                        })
                });

            let gpu_name = pci_slot
                .as_ref()
                .and_then(|slot| {
                    gpu_names
                        .iter()
                        .find(|(s, _)| s == slot)
                        .map(|(_, name)| name.clone())
                })
                .unwrap_or_else(|| format!("GPU ({name_str})"));

            // Utilization (AMD: gpu_busy_percent, Intel i915: similar)
            let utilization = fs::read_to_string(device_path.join("gpu_busy_percent"))
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
                .unwrap_or(0);

            // VRAM (AMD only)
            let mem_used = fs::read_to_string(device_path.join("mem_info_vram_used"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0);
            let mem_total = fs::read_to_string(device_path.join("mem_info_vram_total"))
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .unwrap_or(0);

            // Temperature: scan hwmon subdirectories for temp1_input
            let hwmon_dir = device_path.join("hwmon");
            let temperature = if hwmon_dir.is_dir() {
                fs::read_dir(&hwmon_dir)
                    .ok()
                    .and_then(|entries| {
                        for e in entries.flatten() {
                            let temp_path = e.path().join("temp1_input");
                            if let Ok(val) = fs::read_to_string(&temp_path) {
                                if let Ok(t) = val.trim().parse::<u32>() {
                                    return Some(t / 1000); // millidegrees → degrees
                                }
                            }
                        }
                        None
                    })
                    .unwrap_or(0)
            } else {
                0
            };

            // Power usage (AMD: power1_average in hwmon, microwatts)
            let power_usage = if hwmon_dir.is_dir() {
                fs::read_dir(&hwmon_dir)
                    .ok()
                    .and_then(|entries| {
                        for e in entries.flatten() {
                            let power_path = e.path().join("power1_average");
                            if let Ok(val) = fs::read_to_string(&power_path) {
                                if let Ok(uw) = val.trim().parse::<u64>() {
                                    return Some((uw / 1000) as u32); // microwatts → milliwatts
                                }
                            }
                        }
                        None
                    })
            } else {
                None
            };

            if !self.gpus.iter().any(|g| g.name == gpu_name) {
                self.gpus.push(GpuInfo {
                    name: gpu_name,
                    temperature,
                    utilization,
                    memory_used: mem_used,
                    memory_total: mem_total,
                    fan_speed: None,
                    power_usage,
                    power_limit: None,
                });

                let idx = self.gpus.len() - 1;
                while self.gpu_util_history.len() <= idx {
                    self.gpu_util_history
                        .push(VecDeque::from(vec![0.0; HISTORY_LEN]));
                }
                self.gpu_util_history[idx].pop_front();
                self.gpu_util_history[idx].push_back(utilization as f64);
            }
        }
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
        if let Some(&idx) = self.filtered_processes.get(self.process_scroll)
            && let Some(proc) = self.processes.get(idx)
        {
            self.kill_confirm = Some(proc.pid);
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

    pub fn show_detail(&mut self) {
        if self.active_tab != Tab::Processes {
            return;
        }
        if let Some(&idx) = self.filtered_processes.get(self.process_scroll)
            && let Some(p) = self.processes.get(idx)
        {
            let pid = Pid::from_u32(p.pid);
            let base = ProcessInfo {
                pid: p.pid,
                name: p.name.clone(),
                cpu: p.cpu,
                memory: p.memory,
                status: p.status.clone(),
                run_time: p.run_time,
                disk_read: p.disk_read,
                disk_write: p.disk_write,
            };
            let detail = if let Some(proc_) = self.system.process(pid) {
                ProcessDetail {
                    base,
                    parent_pid: proc_.parent().map(|pp| pp.as_u32()),
                    cmd: proc_.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect::<Vec<_>>().join(" "),
                    exe: proc_.exe().map(|e| e.to_string_lossy().to_string()).unwrap_or_default(),
                    root: proc_.root().map(|r| r.to_string_lossy().to_string()).unwrap_or_default(),
                    environ_count: proc_.environ().len(),
                    threads: proc_.tasks().map(|t| t.len() as u64),
                    virtual_memory: proc_.virtual_memory(),
                }
            } else {
                ProcessDetail {
                    base,
                    parent_pid: None,
                    cmd: String::new(),
                    exe: String::new(),
                    root: String::new(),
                    environ_count: 0,
                    threads: None,
                    virtual_memory: 0,
                }
            };
            self.process_detail = Some(detail);
            self.show_process_detail = true;
        }
    }

    pub fn close_detail(&mut self) {
        self.show_process_detail = false;
        self.process_detail = None;
    }

    pub fn has_gpu(&self) -> bool {
        !self.gpus.is_empty()
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
