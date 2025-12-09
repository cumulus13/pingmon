// File: src/pingmon.rs
// Real-time Ping Monitor with ASCII Chart
// Author: Hadi Cahyadi <cumulus13@gmail.com>

use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use std::thread;
use std::io::{self, Write};
use clap::Parser;
use colored::*;
use rasciichart::{plot_with_config, Config};
use surge_ping::{Client, Config as PingConfig, PingIdentifier, PingSequence, IcmpPacket};

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

fn render_static_line(stats: &Stats, lat: f64, ttl: u8, ok: bool) {
    // Simple one-line output for static mode
    let status = if ok { 
        format!("{:.2}ms", lat).green() 
    } else { 
        "TIMEOUT".red() 
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

fn render_chart_only(args: &Args, history: &VecDeque<f64>, lat: f64, ttl: u8, ok: bool, chart_width: usize) {
    move_cursor_home();

    // Single status line
    print!("{} ", "Latency:".bright_cyan().bold());
    if ok {
        print!("{}", format!(" {:.2} ms ", lat).white().on_blue());
    } else {
        print!("{}", " TIMEOUT ".white().on_red());
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

fn render_dynamic_screen(args: &Args, stats: &Stats, history: &VecDeque<f64>, lat: f64, ttl: u8, ok: bool, chart_width: usize) {
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
        print!("{}", " TIMEOUT ".white().on_red());
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

        // Render output based on mode
        if args.static_mode {
            render_static_line(&stats, lat, ttl, ok);
        } else if args.chart_only {
            render_chart_only(&args, &history, lat, ttl, ok, chart_width);
        } else {
            render_dynamic_screen(&args, &stats, &history, lat, ttl, ok, chart_width);
        }

        if !running.load(Ordering::SeqCst) { 
            break; 
        }
        
        thread::sleep(Duration::from_secs_f64(args.interval));
    }

    // Print final statistics
    print_final_stats(&stats);
}