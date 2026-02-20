<div align="center">

# RustMonitor

A lightweight, cross-platform terminal system monitor built with Rust.

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()


</div>

---

## Overview

RustMonitor is a terminal-based system monitor that provides real-time visibility into CPU, memory, disk, network, and process activity. It is built entirely in Rust using [Ratatui](https://ratatui.rs) for the terminal UI and [sysinfo](https://github.com/GuillaumeGomez/sysinfo) for system data collection.

It is designed for environments where a graphical monitor is unavailable or impractical:

| Scenario | Description |
|----------|-------------|
| SSH sessions | Monitor remote servers directly from the terminal |
| Docker containers | Single binary, no GUI dependencies required |
| Headless servers | Full system visibility without a display server |
| Terminal multiplexers | Run alongside your workflow in a tmux or screen pane |
| Minimal environments | No `htop` or `btop` available — just needs the Rust toolchain |
| Stealth monitoring | Unlike `taskmgr.exe` or `procexp.exe`, RustMonitor is not recognized by process-hiding malware — hidden miners and suspicious processes remain visible |

---

## Features

- **4 dashboard tabs** — Overview, Processes, System Info, Network Detail
- **CPU monitoring** — Per-core usage gauges with color coding and 60-second sparkline history
- **Memory & swap** — Real-time gauges with historical trend visualization
- **GPU monitoring** — NVIDIA (via NVML), AMD (via sysfs on Linux), Apple Silicon/Intel (via system_profiler on macOS) — utilization, VRAM, temperature, fan speed, power draw with sparkline history
- **Process management** — Sortable columns (CPU / Memory / Name / PID), live search filtering, process kill with confirmation
- **Process detail popup** — Press Enter to inspect PID, parent PID, executable path, command line, threads, virtual memory, disk I/O, environment variable count
- **Network monitoring** — Per-interface statistics (RX/TX, packets, errors, MAC address) with live traffic graphs
- **Disk usage** — Per-disk utilization bars with filesystem type display
- **System information** — Hostname, OS, kernel version, CPU model, architecture, uptime, GPU details
- **4 color themes** — Default, Ocean, Forest, Sunset — cycle with a single keypress
- **Keyboard-driven** — Full navigation without a mouse, vim-style keybindings supported
- **Help overlay** — In-app keybinding reference

---

## Prerequisites

Rust must be installed on your system. If it isn't, install it via [rustup](https://rustup.rs):

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Windows users: download and run [rustup-init.exe](https://win.rustup.rs).

---

## Installation

```bash
git clone https://github.com/KamilErdogmus/Rust-Monitor.git
cd Rust-Monitor
cargo build --release
```

Run the application:

```bash
./target/release/rustmonitor        # macOS / Linux
.\target\release\rustmonitor.exe    # Windows
```

Or install it directly:

```bash
cargo install --path .
```

---

## Keybindings

### General

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `1` `2` `3` `4` | Jump to tab directly |
| `t` | Cycle color theme |
| `?` | Toggle help overlay |

### Navigation

| Key | Action |
|-----|--------|
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `PgUp` / `PgDn` | Page up / down |
| `Home` / `End` | Jump to top / bottom |

### Processes

| Key | Action |
|-----|--------|
| `/` | Search / filter processes |
| `s` | Cycle sort column (CPU → Memory → Name → PID) |
| `x` | Kill selected process |
| `Enter` | View process details |

---

## Themes

Cycle through themes with `t`:

| Theme | Description |
|-------|-------------|
| Default | Classic terminal palette |
| Ocean | Cool blue tones |
| Forest | Natural green hues |
| Sunset | Warm orange and red |

---

## Why Rust?

A system monitor reads hardware metrics and renders them to the screen hundreds of times per minute. The choice of language has a direct impact on how much overhead the tool itself introduces.

| Concern | How Rust addresses it |
|---------|----------------------|
| **Runtime overhead** | No garbage collector. No GC pauses. The UI stays responsive under load. |
| **Memory footprint** | The compiled binary uses ~2–3 MB of RAM. Equivalent tools in Python or Node.js typically consume 50–100 MB. |
| **System-level access** | Process enumeration, signal handling, and network interface access are performed with compile-time safety guarantees — no segfaults, no memory leaks. |
| **Portability** | A single codebase compiles to native binaries on Windows, macOS, and Linux. |
| **Distribution** | Release builds produce a single ~3–4 MB executable with zero runtime dependencies. |

---

## Stress Testing

To verify that RustMonitor accurately reflects system activity, run it in one terminal and execute the following commands in a second terminal.

> **Warning:** Memory stress tests can freeze your system if you allocate more than the available free RAM. Check available memory before running.

### CPU (10 seconds, all cores)

**Windows (PowerShell):**
```powershell
1..[Environment]::ProcessorCount | ForEach-Object {
  Start-Job { $end = (Get-Date).AddSeconds(10); while((Get-Date) -lt $end){} }
}
# Cleanup
Get-Job | Stop-Job; Get-Job | Remove-Job
```

**macOS / Linux:**
```bash
for i in $(seq $(nproc 2>/dev/null || sysctl -n hw.ncpu)); do
  (timeout 10 yes > /dev/null &)
done
```

### Memory (10 seconds)

**Windows (PowerShell) — 4 GB:**
```powershell
Start-Job {
  $blocks = @(); for($i=0; $i -lt 4; $i++){ $blocks += New-Object byte[] 1GB }
  Start-Sleep 10
}
Get-Job | Stop-Job; Get-Job | Remove-Job
```

**macOS / Linux — 4 GB:**
```bash
stress-ng --vm 1 --vm-bytes 4G --timeout 10s
```

> Check free memory first:
> - **Windows:** `[math]::Round((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory/1MB, 1)`
> - **macOS:** `memory_pressure | head -1`
> - **Linux:** `free -h`

### Network (10 seconds)

**Windows (PowerShell):**
```powershell
Start-Job {
  $end = (Get-Date).AddSeconds(10)
  while((Get-Date) -lt $end){
    Invoke-WebRequest -Uri "https://speed.hetzner.de/100MB.bin" -OutFile NUL -ErrorAction SilentlyContinue
  }
}
Get-Job | Stop-Job; Get-Job | Remove-Job
```

**macOS / Linux:**
```bash
timeout 10 curl -o /dev/null https://speed.hetzner.de/100MB.bin
```

---

## GPU Support

| Platform | Backend | Data Available |
|----------|---------|----------------|
| NVIDIA (all OS) | NVML via `nvml-wrapper` | Utilization, VRAM, temperature, fan speed, power draw |
| Apple Silicon (macOS) | IOReport private API | Utilization, temperature, power draw, frequency |
| AMD (Linux) | sysfs (`/sys/class/drm`) | Utilization, VRAM, temperature |
| No GPU detected | — | Graceful fallback, panel hidden |

> **Note:** Apple Silicon monitoring uses undocumented macOS APIs (same approach as [macmon](https://github.com/vladkens/macmon)). No sudo required. VRAM is not shown because Apple Silicon uses unified memory shared with the CPU.

---

## Project Structure

```
rustmonitor/
├── Cargo.toml
├── build.rs             # Platform-specific link flags (IOKit on macOS)
├── src/
│   ├── main.rs          # Entry point, event loop, key handling
│   ├── app.rs           # Application state, system data collection, GPU detection
│   ├── macos_gpu.rs     # Apple Silicon GPU via IOReport (macOS only)
│   ├── theme.rs         # 4 color theme definitions
│   └── ui/
│       ├── mod.rs       # Main draw dispatcher, tabs, footer
│       ├── overview.rs  # Overview tab (CPU, memory, disks, network, GPU)
│       ├── processes.rs # Processes tab (table, search bar)
│       ├── system.rs    # System info tab (details + resource gauges)
│       ├── network.rs   # Network detail tab (sparklines + interface table)
│       ├── popups.rs    # Help, kill confirm, process detail popups
│       └── helpers.rs   # Shared utilities (centered_rect, info_line, etc.)
```

---

## Tech Stack

| Crate | Version | Purpose |
|-------|---------|---------|
| [Ratatui](https://ratatui.rs) | 0.30 | Terminal UI framework |
| [Crossterm](https://github.com/crossterm-rs/crossterm) | 0.29 | Cross-platform terminal manipulation |
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.38.2 | System information gathering |
| [nvml-wrapper](https://github.com/Cldfire/nvml-wrapper) | 0.12 | NVIDIA GPU monitoring (with cross-platform fallbacks) |

## License

[MIT](LICENSE)
