// File: src/pingmon.rs
// Real-time Ping Monitor with ASCII Chart
// Author: Hadi Cahyadi <cumulus13@gmail.com>

use std::collections::VecDeque;
use std::net::IpAddr;
use std::env;
use std::path::PathBuf;
use std::fs;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};
use clap::Parser;
use colored::*;
use rasciichart::{plot_with_config, Config};
use surge_ping::{Client, Config as PingConfig, PingIdentifier, PingSequence, IcmpPacket};
use gntp::{GntpClient, NotificationType, NotifyOptions, Resource};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Ping monitor with real-time chart")]
struct Args {
    /// Target host to ping
    #[clap(default_value = "8.8.8.8")]
    host: String,
    
    /// Chart height
    #[clap(short = 'H', long, default_value = "15")]
    height: usize,
    
    /// Chart width (0 = auto)
    #[clap(short = 'W', long, default_value = "0")]
    width: usize,
    
    /// Interval between pings (seconds)
    #[clap(short, long, default_value = "1.0")]
    interval: f64,
    
    /// Static mode: simple line-by-line output without chart
    #[clap(short, long)]
    static_mode: bool,
    
    /// Chart-only mode: only show chart and current status
    #[clap(short, long)]
    chart_only: bool,
    
    /// Timeout threshold in seconds before sending notification
    #[clap(short, long)]
    timeout_threshold: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct AppConfig {
    #[serde(default = "default_timeout")]
    pingmon_timeout: u64,
}

fn default_timeout() -> u64 {
    60 // 1 minute default
}

impl AppConfig {
    fn load() -> Self {
        // Try to get from environment variable first
        if let Ok(timeout_str) = env::var("PINGMON_TIMEOUT") {
            if let Ok(timeout) = timeout_str.parse::<u64>() {
                return AppConfig {
                    pingmon_timeout: timeout,
                };
            }
        }

        // Try to load from config file
        if let Some(config_path) = get_config_file() {
            if let Ok(contents) = fs::read_to_string(&config_path) {
                // Try different formats based on file extension
                let ext = config_path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                
                match ext {
                    "toml" => {
                        if let Ok(config) = toml::from_str::<AppConfig>(&contents) {
                            return config;
                        }
                    }
                    "json" => {
                        if let Ok(config) = serde_json::from_str::<AppConfig>(&contents) {
                            return config;
                        }
                    }
                    "yml" | "yaml" => {
                        if let Ok(config) = serde_yaml::from_str::<AppConfig>(&contents) {
                            return config;
                        }
                    }
                    "ini" => {
                        // Simple INI parser for pingmon_timeout
                        for line in contents.lines() {
                            let line = line.trim();
                            if line.starts_with("pingmon_timeout") {
                                if let Some(value) = line.split('=').nth(1) {
                                    if let Ok(timeout) = value.trim().parse::<u64>() {
                                        return AppConfig {
                                            pingmon_timeout: timeout,
                                        };
                                    }
                                }
                            }
                        }
                    }
                    "env" => {
                        // Parse .env file
                        for line in contents.lines() {
                            let line = line.trim();
                            if line.starts_with("PINGMON_TIMEOUT") {
                                if let Some(value) = line.split('=').nth(1) {
                                    if let Ok(timeout) = value.trim().parse::<u64>() {
                                        return AppConfig {
                                            pingmon_timeout: timeout,
                                        };
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Return default if nothing found
        AppConfig::default()
    }
}

fn get_config_file() -> Option<PathBuf> {
    let config_files = if cfg!(windows) {
        vec![
            dirs::home_dir()?.join(".pingmon").join(".env"),
            dirs::config_dir()?.join(".pingmon").join(".env"),
            
            dirs::home_dir()?.join(".pingmon").join("pingmon.ini"),
            dirs::config_dir()?.join(".pingmon").join("pingmon.ini"),
            
            dirs::home_dir()?.join(".pingmon").join("pingmon.toml"),
            dirs::config_dir()?.join(".pingmon").join("pingmon.toml"),
            
            dirs::home_dir()?.join(".pingmon").join("pingmon.json"),
            dirs::config_dir()?.join(".pingmon").join("pingmon.json"),
            
            dirs::home_dir()?.join(".pingmon").join("pingmon.yml"),
            dirs::config_dir()?.join(".pingmon").join("pingmon.yml"),
        ]
    } else {
        vec![
            dirs::home_dir()?.join(".pingmon").join(".env"),
            dirs::config_dir()?.join(".pingmon").join(".env"),
            dirs::config_dir()?.join(".env"),
            
            dirs::home_dir()?.join(".pingmon").join("pingmon.ini"),
            dirs::config_dir()?.join(".pingmon").join("pingmon.ini"),
            dirs::config_dir()?.join("pingmon.ini"),
            
            dirs::home_dir()?.join(".pingmon").join("pingmon.toml"),
            dirs::config_dir()?.join(".pingmon").join("pingmon.toml"),
            dirs::config_dir()?.join("pingmon.toml"),
            
            dirs::home_dir()?.join(".pingmon").join("pingmon.json"),
            dirs::config_dir()?.join(".pingmon").join("pingmon.json"),
            dirs::config_dir()?.join("pingmon.json"),
            
            dirs::home_dir()?.join(".pingmon").join("pingmon.yml"),
            dirs::config_dir()?.join(".pingmon").join("pingmon.yml"),
            dirs::config_dir()?.join("pingmon.yml"),
        ]
    };

    for path in config_files {
        if path.exists() {
            return Some(path);
        }
    }

    None
}

#[derive(Clone)]
struct Stats {
    sent: u64,
    received: u64,
    lost: u64,
    min: f64,
    max: f64,
    latencies: Vec<f64>,
}

impl Stats {
    fn new() -> Self {
        Self {
            sent: 0,
            received: 0,
            lost: 0,
            min: f64::INFINITY,
            max: 0.0,
            latencies: Vec::new(),
        }
    }

    fn avg(&self) -> f64 {
        if self.latencies.is_empty() { 
            0.0 
        } else { 
            self.latencies.iter().sum::<f64>() / self.latencies.len() as f64 
        }
    }

    fn stddev(&self) -> f64 {
        if self.latencies.len() < 2 { 
            return 0.0; 
        }
        let avg = self.avg();
        let variance = self.latencies.iter()
            .map(|&x| (x - avg).powi(2))
            .sum::<f64>() / (self.latencies.len() - 1) as f64;
        variance.sqrt()
    }

    fn loss_pct(&self) -> f64 {
        if self.sent == 0 { 
            0.0 
        } else { 
            (self.lost as f64 / self.sent as f64) * 100.0 
        }
    }
}

struct TimeoutTracker {
    timeout_start: Option<Instant>,
    notification_sent: bool,
    threshold_seconds: u64,
}

impl TimeoutTracker {
    fn new(threshold_seconds: u64) -> Self {
        Self {
            timeout_start: None,
            notification_sent: false,
            threshold_seconds,
        }
    }

    fn update(&mut self, is_timeout: bool) -> bool {
        if is_timeout {
            if self.timeout_start.is_none() {
                // First timeout detected
                self.timeout_start = Some(Instant::now());
                self.notification_sent = false;
            } else if !self.notification_sent {
                // Check if timeout duration exceeds threshold
                if let Some(start) = self.timeout_start {
                    let elapsed = start.elapsed().as_secs();
                    if elapsed >= self.threshold_seconds {
                        self.notification_sent = true;
                        return true; // Signal to send notification
                    }
                }
            }
        } else {
            // Connection restored
            self.timeout_start = None;
            self.notification_sent = false;
        }
        false
    }

    fn get_timeout_duration(&self) -> Option<u64> {
        self.timeout_start.map(|start| start.elapsed().as_secs())
    }
}

fn get_icon_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        // Development mode
        Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("pingmon.png"))
    } else {
        // Production mode
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cannot get executable directory"))?;
        
        let possible_paths = vec![
            exe_dir.join("pingmon.png"),
            exe_dir.join("icons").join("pingmon.png"),
            exe_dir.join("resources").join("pingmon.png"),
            env::current_dir()?.join("pingmon.png"),
            PathBuf::from("pingmon.png"),
        ];
        
        // Find first existing file
        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }
        
        // Return default even if not exists
        Ok(PathBuf::from("pingmon.png"))
    }
}

fn send_notification(title: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let icon_mode = gntp::IconMode::Binary;
    let mut client = GntpClient::new("pingmon").with_icon_mode(icon_mode);
    let icon_path = get_icon_path()?;

    let notif_icon = Resource::from_file(&icon_path).ok();

    let mut notification = if let Some(ref icon) = notif_icon {
        NotificationType::new("alert")
            .with_display_name("Alert")
            .with_enabled(true)
            .with_icon(icon.clone())
    } else {
        NotificationType::new("alert")
            .with_display_name("Alert")
            .with_enabled(true)
    };
    
    if let Some(ref icon) = notif_icon {
        notification = notification.with_icon(icon.clone());
    }

    client.register(vec![notification])?;

    let mut options = NotifyOptions::new();
    if let Some(ref icon) = notif_icon {
        options = options.with_icon(icon.clone());
    }

    client.notify_with_options(
        "alert",
        title,
        message,
        options
    )?;

    Ok(())
}

fn ping_once_sync(host: &str) -> (f64, u8, bool) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return (0.0, 0, false),
    };
    
    rt.block_on(async {
        let addr: IpAddr = match host.parse() {
            Ok(ip) => ip,
            Err(_) => {
                match tokio::net::lookup_host(format!("{}:0", host)).await {
                    Ok(mut addrs) => {
                        if let Some(addr) = addrs.next() { 
                            addr.ip() 
                        } else { 
                            return (0.0, 0, false); 
                        }
                    }
                    Err(_) => return (0.0, 0, false),
                }
            }
        };

        let client = match Client::new(&PingConfig::default()) {
            Ok(c) => c,
            Err(_) => return (0.0, 0, false),
        };

        let mut pinger = client.pinger(addr, PingIdentifier(rand::random())).await;
        match pinger.ping(PingSequence(0), &[0; 56]).await {
            Ok((IcmpPacket::V4(packet), duration)) => {
                let ttl = packet.get_ttl().unwrap_or(0);
                (duration.as_secs_f64() * 1000.0, ttl, true)
            },
            Ok((IcmpPacket::V6(_packet), duration)) => {
                // IPv6 uses hop limit instead of TTL
                (duration.as_secs_f64() * 1000.0, 0, true)
            }
            Err(_) => (0.0, 0, false),
        }
    })
}

fn get_term_size() -> (u16, u16) {
    term_size::dimensions()
        .map(|(w, h)| (w as u16, h as u16))
        .unwrap_or((80, 24))
}

fn move_cursor_home() {
    print!("\x1B[H");
    let _ = io::stdout().flush();
}

fn clear_line_to_end() {
    print!("\x1B[K");
}

fn render_static_line(stats: &Stats, lat: f64, ttl: u8, ok: bool, timeout_duration: Option<u64>) {
    // Simple one-line output for static mode
    let status = if ok { 
        format!("{:.2}ms", lat).green() 
    } else {
        let mut timeout_msg = "TIMEOUT".to_string();
        if let Some(duration) = timeout_duration {
            timeout_msg.push_str(&format!(" ({}s)", duration));
        }
        timeout_msg.red()
    };
    
    let ttl_str = if ok { 
        format!("ttl={}", ttl).yellow() 
    } else { 
        "ttl=-".yellow() 
    };
    
    print!("seq={} {} {} ", 
        format!("{}", stats.sent).cyan(),
        status,
        ttl_str
    );
    
    if stats.received > 0 {
        print!("(loss={:.1}% avg={:.2}ms)", 
            stats.loss_pct(),
            stats.avg()
        );
    }
    
    println!();
}

fn render_chart_only(args: &Args, history: &VecDeque<f64>, lat: f64, ttl: u8, ok: bool, chart_width: usize, timeout_duration: Option<u64>) {
    move_cursor_home();

    // Single status line
    print!("{} ", "Latency:".bright_cyan().bold());
    if ok {
        print!("{}", format!(" {:.2} ms ", lat).white().on_blue());
    } else {
        let mut timeout_msg = " TIMEOUT ".to_string();
        if let Some(duration) = timeout_duration {
            timeout_msg = format!(" TIMEOUT ({}s) ", duration);
        }
        print!("{}", timeout_msg.white().on_red());
    }
    print!(" | ");
    print!("{} ", "TTL:".bright_green().bold());
    print!("{}", format!(" {} ", ttl).black().on_green());
    print!(" | ");
    print!("{} ", "Status:".bold());
    if ok {
        print!("{}", " CONNECTED ".black().on_truecolor(0, 255, 255));
    } else {
        print!("{}", " TIMEOUT ".white().on_red());
    }
    print!(" | ");
    print!("{} ", "Host:".bright_magenta().bold());
    print!("{}", args.host.yellow());
    clear_line_to_end();
    println!();

    // Chart only (NO extra newline before chart)
    if history.len() > 1 {
        print!("{}", "Latency History (ms):".bright_cyan().bold());
        clear_line_to_end();
        println!();
        
        let data: Vec<f64> = history.iter().copied().collect();
        
        let config = Config::new()
            .with_height(args.height)
            .with_width(chart_width);
        
        if let Ok(chart) = plot_with_config(&data, config) {
            let lines: Vec<&str> = chart.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                print!("{}", line.yellow());
                clear_line_to_end();
                // NO newline after last line
                if i < lines.len() - 1 {
                    println!();
                }
            }
        }
    }

    // Clear rest of screen (NO extra newline)
    print!("\x1B[J");
    let _ = io::stdout().flush();
}

fn render_dynamic_screen(args: &Args, stats: &Stats, history: &VecDeque<f64>, lat: f64, ttl: u8, ok: bool, chart_width: usize, timeout_duration: Option<u64>) {
    move_cursor_home();

    // Header
    print!("{}", format!("=== Real-time Ping Monitor: {} ===", args.host).bright_magenta().bold());
    clear_line_to_end();
    println!();

    // Status line (NO extra newline before)
    print!("{} ", "Latency:".bright_cyan().bold());
    if ok {
        print!("{}", format!(" {:.2} ms ", lat).white().on_blue());
    } else {
        let mut timeout_msg = " TIMEOUT ".to_string();
        if let Some(duration) = timeout_duration {
            timeout_msg = format!(" TIMEOUT ({}s) ", duration);
        }
        print!("{}", timeout_msg.white().on_red());
    }
    print!(" | ");
    print!("{} ", "TTL:".bright_green().bold());
    print!("{}", format!(" {} ", ttl).black().on_green());
    print!(" | ");
    print!("{} ", "Status:".bold());
    if ok {
        print!("{}", " CONNECTED ".black().on_truecolor(0, 255, 255));
    } else {
        print!("{}", " TIMEOUT ".white().on_red());
    }
    clear_line_to_end();
    println!();

    // Stats
    print!("{}", "Statistics:".bright_yellow().bold());
    clear_line_to_end();
    println!();
    print!("  Sent: {} | Received: {} | Lost: {} ({})",
        format!("{}", stats.sent).cyan(),
        format!("{}", stats.received).green(),
        format!("{}", stats.lost).red(),
        format!("{:.1}%", stats.loss_pct()).red()
    );
    clear_line_to_end();
    println!();

    if stats.received > 0 {
        print!("  Min: {} | Avg: {} | Max: {} | StdDev: {}",
            format!("{:.2}ms", stats.min).green(),
            format!("{:.2}ms", stats.avg()).yellow(),
            format!("{:.2}ms", stats.max).red(),
            format!("{:.2}ms", stats.stddev()).cyan()
        );
        clear_line_to_end();
        println!();
    }

    // Chart (NO extra newline before chart header)
    if history.len() > 1 {
        print!("{}", "Latency History (ms):".bright_cyan().bold());
        clear_line_to_end();
        println!();
        
        let data: Vec<f64> = history.iter().copied().collect();
        
        let config = Config::new()
            .with_height(args.height)
            .with_width(chart_width);
        
        if let Ok(chart) = plot_with_config(&data, config) {
            let lines: Vec<&str> = chart.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                print!("{}", line.yellow());
                clear_line_to_end();
                // NO newline after last line
                if i < lines.len() - 1 {
                    println!();
                }
            }
        }
    }

    // Clear rest of screen (NO extra newline)
    print!("\x1B[J");
    let _ = io::stdout().flush();
}

fn print_final_stats(stats: &Stats) {
    println!("\n{}", "✓ Stopped".green().bold());
    println!("\n{}", "Final Statistics:".bright_yellow().bold());
    println!("  Packets: Sent = {}, Received = {}, Lost = {} ({})",
        stats.sent, 
        stats.received, 
        stats.lost,
        format!("{:.1}%", stats.loss_pct()).red()
    );

    if stats.received > 0 {
        println!("  Latency: Min = {}, Avg = {}, Max = {}, StdDev = {}",
            format!("{:.2}ms", stats.min).green(),
            format!("{:.2}ms", stats.avg()).yellow(),
            format!("{:.2}ms", stats.max).red(),
            format!("{:.2}ms", stats.stddev()).cyan()
        );
    }
}

fn main() {
    let args = Args::parse();
    let config = AppConfig::load();
    
    // Determine timeout threshold: CLI arg > config/env > default (60)
    let timeout_threshold = args.timeout_threshold.unwrap_or(config.pingmon_timeout);
    
    println!("{}", format!("Timeout notification threshold: {} seconds", timeout_threshold).bright_yellow());
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let (term_w, _) = get_term_size();
    let hist_size = if args.width > 0 { 
        args.width 
    } else { 
        (term_w as usize).saturating_sub(14).max(50) 
    };

    // FIX: chart_width harus mengikuti hist_size (current terminal width)
    let chart_width = hist_size;

    let mut history: VecDeque<f64> = VecDeque::with_capacity(hist_size);
    let mut stats = Stats::new();
    let mut timeout_tracker = TimeoutTracker::new(timeout_threshold);

    // Initial setup
    if !args.static_mode && !args.chart_only {
        // Clear screen once for dynamic mode
        print!("\x1B[2J\x1B[H");
        let _ = io::stdout().flush();
    } else if args.static_mode {
        // Print header for static mode
        println!("{}", format!("Pinging {} ...", args.host).bright_magenta().bold());
    } else if args.chart_only {
        // Clear screen once for chart-only mode
        print!("\x1B[2J\x1B[H");
        let _ = io::stdout().flush();
    }

    // Main loop
    while running.load(Ordering::SeqCst) {
        let (lat, ttl, ok) = ping_once_sync(&args.host);
        stats.sent += 1;

        if ok {
            stats.received += 1;
            stats.min = stats.min.min(lat);
            stats.max = stats.max.max(lat);
            stats.latencies.push(lat);
            history.push_back(lat);
        } else {
            stats.lost += 1;
            history.push_back(0.0);
        }

        if history.len() > hist_size {
            history.pop_front();
        }

        // Check timeout and send notification if threshold exceeded
        let should_notify = timeout_tracker.update(!ok);
        if should_notify {
            let duration = timeout_tracker.get_timeout_duration().unwrap_or(0);
            let message = format!("Connection timeout for {} seconds to {}", duration, args.host);
            let _ = send_notification("Pingmon: Connection Timeout", &message);
        }

        let timeout_duration = timeout_tracker.get_timeout_duration();

        // Render output based on mode
        if args.static_mode {
            render_static_line(&stats, lat, ttl, ok, timeout_duration);
        } else if args.chart_only {
            render_chart_only(&args, &history, lat, ttl, ok, chart_width, timeout_duration);
        } else {
            render_dynamic_screen(&args, &stats, &history, lat, ttl, ok, chart_width, timeout_duration);
        }

        if !running.load(Ordering::SeqCst) { 
            break; 
        }
        
        thread::sleep(Duration::from_secs_f64(args.interval));
    }

    // Print final statistics
    print_final_stats(&stats);
}