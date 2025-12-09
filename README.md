# pingmon

[![Crates.io](https://img.shields.io/crates/v/pingmon.svg)](https://crates.io/crates/pingmon)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Real-time ping monitor with beautiful ASCII charts, TTL display, and comprehensive statistics.

![pingmon screenshot](https://raw.githubusercontent.com/cumulus13/pingmon/main/screenshots/pingmon.png)

## Screenshot

<p align="center">
  <img src="https://raw.githubusercontent.com/cumulus13/pingmon/master/screenshots/pingmon_1_2.png" alt="pingmon line 1">
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/cumulus13/pingmon/master/screenshots/pingmon_1_1.png" alt="pingmon line 2">
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/cumulus13/pingmon/master/screenshots/pingmon_2_1.png" alt="pingmon bar 1">
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/cumulus13/pingmon/master/screenshots/pingmon_2_2.png" alt="pingmon bar 2">
</p>


## Features

✨ **Two Chart Types:**
- `pingmon` - Line chart using ASCII characters
- `pingmon-bar` - Vertical bar chart with filled blocks

📊 **Real-time Visualization:**
- Live latency chart that updates every ping
- Automatic terminal width detection
- Customizable chart height and width

📈 **Comprehensive Statistics:**
- Min/Max/Average latency
- Standard deviation
- Packet loss percentage
- Sent/Received/Lost packet counts
- TTL (Time To Live) display

🎨 **Beautiful Output:**
- Color-coded status indicators
- Clean, modern interface
- Multiple display modes (dynamic, static, chart-only)

## Installation

### From crates.io

```bash
cargo install pingmon
```

### From source

```bash
git clone https://github.com/cumulus13/pingmon
cd pingmon
cargo build --release
```

The binaries will be in `target/release/`:
- `pingmon` (line chart)
- `pingmon-bar` (bar chart)

## Usage

### Basic Usage

```bash
# Ping Google DNS with line chart
pingmon

# Ping specific host with bar chart
pingmon-bar 1.1.1.1

# Ping with custom interval
pingmon google.com -i 0.5
```

### Command Line Options

```
USAGE:
    pingmon [OPTIONS] [HOST]
    pingmon-bar [OPTIONS] [HOST]

ARGS:
    <HOST>    Target host to ping [default: 8.8.8.8]

OPTIONS:
    -H, --height <HEIGHT>      Chart height [default: 15 for pingmon, 12 for bar]
    -W, --width <WIDTH>        Chart width (0 = auto) [default: 0]
    -i, --interval <INTERVAL>  Interval between pings (seconds) [default: 1.0]
    -s, --static-mode          Simple line-by-line output without chart
    -c, --chart-only           Only show chart and current status
    -h, --help                 Print help information
    -V, --version              Print version information
```

### Examples

```bash
# Ping with custom chart size
pingmon 8.8.8.8 -H 20 -W 100

# Fast pinging (every 0.5 seconds)
pingmon google.com -i 0.5

# Static mode (no chart, just lines)
pingmon cloudflare.com --static-mode

# Chart-only mode (minimal display)
pingmon-bar 1.1.1.1 --chart-only

# Tall bar chart
pingmon-bar -H 25
```

## Display Modes

### 1. Dynamic Mode (Default)
Full-featured display with header, statistics, and chart.

```
=== Real-time Ping Monitor: 8.8.8.8 ===

Latency:  20.07 ms  | TTL:  112  | Status:  CONNECTED 

Statistics:
  Sent: 45 | Received: 45 | Lost: 0 (0.0%)
  Min: 18.23ms | Avg: 20.15ms | Max: 25.67ms | StdDev: 1.45ms

Latency History (ms):
[ASCII chart here]
```

### 2. Static Mode (`-s`)
Simple line-by-line output, perfect for logging.

```
Pinging 8.8.8.8 ...
seq=1 20.07ms ttl=112 (loss=0.0% avg=20.07ms)
seq=2 19.85ms ttl=112 (loss=0.0% avg=19.96ms)
seq=3 21.34ms ttl=112 (loss=0.0% avg=20.42ms)
```

### 3. Chart-Only Mode (`-c`)
Minimal display with just status and chart.

```
Latency:  20.07 ms  | TTL:  112  | Status:  CONNECTED  | Host: 8.8.8.8

Latency History (ms):
[ASCII chart here]
```

## Chart Types

### Line Chart (`pingmon`)
```
Latency History (ms):
   25.0 ┤     ╭╮    
   20.0 ┤ ╭───╯╰─╮  
   15.0 ┼─╯      ╰─
   10.0 ┤          
```

### Bar Chart (`pingmon-bar`)
```
Latency History (ms):
  39.6 │    █    █
  26.4 │█████████████
  13.2 │█████████████
   0.0  └──────────────
```

## Requirements

- **Operating System:** Linux, macOS, or Windows
- **Privileges:** 
  - Linux/macOS: Requires `sudo` or `CAP_NET_RAW` capability
  - Windows: Run as Administrator

### Linux Capability Setup (Recommended)

Instead of using `sudo` every time, you can set the capability:

```bash
# After installation
sudo setcap cap_net_raw+ep $(which pingmon)
sudo setcap cap_net_raw+ep $(which pingmon-bar)
```

## How It Works

`pingmon` uses:
- **surge-ping** for efficient ICMP ping implementation
- **rasciichart** for beautiful ASCII line charts
- **colored** for terminal color output
- **tokio** for async runtime
- Custom bar chart rendering for filled graphs

The tool automatically:
- Detects terminal width and adjusts chart size
- Handles both IPv4 and IPv6 addresses
- Resolves hostnames to IP addresses
- Displays TTL (Time To Live) values
- Calculates comprehensive statistics

## Stopping the Monitor

Press `Ctrl+C` to stop. Final statistics will be displayed:

```
✓ Stopped

Final Statistics:
  Packets: Sent = 45, Received = 45, Lost = 0 (0.0%)
  Latency: Min = 18.23ms, Avg = 20.15ms, Max = 25.67ms, StdDev = 1.45ms
```

## Troubleshooting

### Permission Denied

**Linux/macOS:**
```bash
# Option 1: Use sudo
sudo pingmon 8.8.8.8

# Option 2: Set capability (recommended)
sudo setcap cap_net_raw+ep ~/.cargo/bin/pingmon
```

**Windows:**
- Run Command Prompt or PowerShell as Administrator

### Host Not Found

Make sure the hostname is correct:
```bash
pingmon google.com  # Correct
pingmon gogle.com   # Will fail - typo
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Author

**Hadi Cahyadi** - [cumulus13@gmail.com](mailto:cumulus13@gmail.com)

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)

[Support me on Patreon](https://www.patreon.com/cumulus13)

## Changelog

### v0.1.0 (Initial Release)
- Real-time ping monitoring with ASCII charts
- Two chart types: line and bar
- Comprehensive statistics
- Multiple display modes
- Auto terminal width detection
- TTL display
- Color-coded output