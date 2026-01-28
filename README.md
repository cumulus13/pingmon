# Pingmon - Real-time Ping Monitor with Timeout Notifications

[![Crates.io](https://img.shields.io/crates/v/pingmon.svg)](https://crates.io/crates/pingmon)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Real-time network monitoring tool with beautiful ASCII/bar charts, TTL display, statistics, and intelligent timeout notifications.

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
- `pingmon2` - Vertical bar chart with filled blocks

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

🔔 Smart timeout notifications via Growl

## Installation

### From crates.io

```bash
cargo install pingmon
```

### From source

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/cumulus13/pingmon.git
cd pingmon

# Build both binaries
cargo build --release

# Binaries will be in target/release/
# - pingmon (ASCII line chart)
# - pingmon-bar (bar chart)
```

The binaries will be in `target/release/`:
- `pingmon` (line chart)
- `pingmon2` (bar chart)

## Usage

### Basic Usage

```bash
# Using ASCII line chart (default)
pingmon 8.8.8.8

# Using bar chart
pingmon-bar 8.8.8.8

# With custom interval
pingmon google.com -i 0.5

# Chart-only mode (minimal display)
pingmon 1.1.1.1 --chart-only

# Static mode (line-by-line output)
pingmon 8.8.8.8 --static-mode
```

### Timeout Configuration

#### 1. Via Command Line

```bash
# Set 2-minute threshold before notification
pingmon 8.8.8.8 --timeout-threshold 120

# Short form
pingmon 8.8.8.8 -t 120
```

#### 2. Via Environment Variable

```bash
# Linux/macOS
export PINGMON_TIMEOUT=120
pingmon 8.8.8.8

# Windows
set PINGMON_TIMEOUT=120
pingmon.exe 8.8.8.8
```

#### 3. Via Configuration File

Create a configuration file in one of the supported locations (see Configuration File Locations below).

### Configuration File Locations

**Windows:**
- `%USERPROFILE%\.pingmon\.env`
- `%USERPROFILE%\.pingmon\pingmon.toml`
- `%USERPROFILE%\.pingmon\pingmon.json`
- `%USERPROFILE%\.pingmon\pingmon.ini`
- `%USERPROFILE%\.pingmon\pingmon.yml`
- `%APPDATA%\.pingmon\` (all formats above)

**Linux/macOS:**
- `~/.pingmon/.env`
- `~/.pingmon/pingmon.toml`
- `~/.pingmon/pingmon.json`
- `~/.pingmon/pingmon.ini`
- `~/.pingmon/pingmon.yml`
- `~/.config/.pingmon/` (all formats above)
- `~/.config/` (all formats above)

### Configuration File Formats

**Format .env:**
```bash
PINGMON_TIMEOUT=120
```

**Format .toml:**
```toml
pingmon_timeout = 120
```

**Format .json:**
```json
{
  "pingmon_timeout": 120
}
```

**Format .ini:**
```ini
pingmon_timeout=120
```

**Format .yml/.yaml:**
```yaml
pingmon_timeout: 120
```

## Configuration Priority

The system uses timeout values with the following priority (highest to lowest):

1. **Command line argument** (`--timeout-threshold` or `-t`)
2. **Environment variable** (`PINGMON_TIMEOUT`)
3. **Config file** (first found in search order)
4. **Default value** (60 seconds)

## Command Line Options

### pingmon (ASCII Line Chart)

```
USAGE:
    pingmon [OPTIONS] [HOST]

ARGS:
    <HOST>    Target host to ping [default: 8.8.8.8]

OPTIONS:
    -c, --chart-only                     Chart-only mode: minimal display with chart
    -h, --help                           Print help information
    -H, --height <HEIGHT>                Chart height [default: 15]
    -i, --interval <INTERVAL>            Interval between pings (seconds) [default: 1.0]
    -s, --static-mode                    Static mode: line-by-line output without chart
    -t, --timeout-threshold <SECONDS>    Timeout threshold before sending notification
    -V, --version                        Print version information
    -W, --width <WIDTH>                  Chart width (0 = auto) [default: 0]
```

### pingmon-bar (Bar Chart)

```
USAGE:
    pingmon-bar [OPTIONS] [HOST]

ARGS:
    <HOST>    Target host to ping [default: 8.8.8.8]

OPTIONS:
    -c, --chart-only                     Chart-only mode: minimal display with chart
    -h, --help                           Print help information
    -H, --height <HEIGHT>                Chart height [default: 12]
    -i, --interval <INTERVAL>            Interval between pings (seconds) [default: 1.0]
    -s, --static-mode                    Static mode: line-by-line output without chart
    -t, --timeout-threshold <SECONDS>    Timeout threshold before sending notification
    -V, --version                        Print version information
    -W, --width <WIDTH>                  Chart width (0 = auto) [default: 0]
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

## How Timeout Notification Works

### Timeout Detection Logic

1. **First timeout detected** → Timer starts
2. **Timeout persists** → Duration counter increments
3. **Threshold reached** (e.g., 60s) → **Notification sent**
4. **Connection recovers** → Timer resets, counter clears
5. **Notification sent once** per continuous timeout period

### Example Scenarios

#### Scenario 1: Brief Timeout (< threshold)
```
0s  - Ping OK
1s  - TIMEOUT (timer starts)
2s  - TIMEOUT (2s elapsed)
3s  - Ping OK (timer resets, NO notification sent)
```
✅ No notification - timeout was brief

#### Scenario 2: Extended Timeout (≥ threshold)
```
0s  - Ping OK
1s  - TIMEOUT (timer starts)
2s  - TIMEOUT (2s)
...
60s - TIMEOUT (60s) → **📢 NOTIFICATION SENT**
61s - TIMEOUT (61s)
62s - Ping OK (timer resets)
```
✅ Notification sent - threshold exceeded

#### Scenario 3: Repeated Timeouts
```
0s  - TIMEOUT (timer starts)
...
60s - TIMEOUT (60s) → **📢 NOTIFICATION SENT**
70s - Ping OK (timer resets)
71s - TIMEOUT (new timer starts)
...
131s - TIMEOUT (60s) → **📢 NOTIFICATION SENT AGAIN**
```
✅ Multiple notifications - each for separate timeout period

## Display Modes

### 1. Dynamic Mode (Default)
- Complete statistics
- Real-time chart
- Status indicators
- All metrics visible

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

### 2. Static Mode (`-s`, `--static-mode`)
- Line-by-line output
- No terminal manipulation
- Ideal for logging

Simple line-by-line output, perfect for logging.

```
Pinging 8.8.8.8 ...
seq=1 20.07ms ttl=112 (loss=0.0% avg=20.07ms)
seq=2 19.85ms ttl=112 (loss=0.0% avg=19.96ms)
seq=3 21.34ms ttl=112 (loss=0.0% avg=20.42ms)
```

### 3. Chart-Only Mode (`-c`, `--chart-only`)
- Minimal status line
- Larger chart display
- Perfect for monitoring

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

### Bar Chart (`pingmon2 / bar`)
```
Latency History (ms):
  39.6 │    █    █
  26.4 │█████████████
  13.2 │█████████████
   0.0  └──────────────
```

## Requirements

- **Rust**: 1.70.0 or later
- **Operating System:** Linux, macOS, or Windows
- **Privileges**: May require administrator/root for ICMP ping
  - Linux/macOS: Requires `sudo` or `CAP_NET_RAW` capability
  - Windows: Run as Administrator

### Growl Notifications
- **Windows/Linux**: Growl for Windows or compatible
- **macOS**: Growl or compatible notification system
- Notifications fail gracefully if Growl is unavailable

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

### Ping Fails
- **Linux**: Run with `sudo` for raw socket access
- **Windows**: Run as Administrator
- Check firewall settings for ICMP

### Notifications Not Working
- Verify Growl is installed and running
- Check notification settings in Growl
- Notifications fail silently if Growl unavailable

### Config File Not Found
- Check file is in correct location
- Verify file permissions
- Run with `--timeout-threshold` to bypass config file

### Wrong Timeout Value Used
Check priority:
1. CLI argument overrides everything
2. Environment variable overrides config file
3. Config file overrides default
4. Default is 60 seconds

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

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Build specific binary
cargo build --release --bin pingmon
cargo build --release --bin pingmon2
```

## Dependencies

### New Dependencies (v0.1.6)
- `serde` - Configuration deserialization
- `serde_json` - JSON config support
- `toml` - TOML config support
- `serde_yaml` - YAML config support
- `dirs` - Cross-platform directory paths

### Existing Dependencies
- `surge-ping` - ICMP ping implementation
- `colored` - Terminal colors
- `term_size` - Terminal size detection
- `ctrlc` - Ctrl+C handler
- `rand` - Random number generation
- `rasciichart` - ASCII chart rendering
- `gntp` - Growl notification protocol
- `tokio` - Async runtime
- `clap` - Command-line argument parsing

## Testing

### Test with Different Configurations

```bash
# Test with environment variable
PINGMON_TIMEOUT=30 cargo run --bin pingmon -- 8.8.8.8

# Test with config file
mkdir -p ~/.pingmon
echo "pingmon_timeout = 30" > ~/.pingmon/pingmon.toml
cargo run --bin pingmon -- 8.8.8.8

# Test with CLI argument (highest priority)
cargo run --bin pingmon -- 8.8.8.8 --timeout-threshold 30

# Test bar chart version
cargo run --bin pingmon-bar -- 8.8.8.8 -t 45
```

### Test Timeout Notification

```bash
# Monitor a host, then disconnect network after 30s
pingmon 8.8.8.8 -t 30

# Monitor local address, stop service to trigger timeout
pingmon 192.168.1.1 -t 45
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) file for details

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)

## Links

- **Repository**: https://github.com/cumulus13/pingmon
- **Documentation**: https://docs.rs/pingmon
- **Issues**: https://github.com/cumulus13/pingmon/issues

## Changelog

### v0.1.6 (2025-01-28)
- ✨ Added intelligent timeout notification system
- ✨ Added flexible configuration system (env vars + config files)
- ✨ Added timeout duration display in UI
- ✨ Support for multiple config file formats (.env, .toml, .json, .ini, .yml)
- 📝 Updated documentation with timeout notification examples
- 🔧 Improved notification reliability

### v0.1.5
- Previous stable release

### v0.1.0 (Initial Release)
- Real-time ping monitoring with ASCII charts
- Two chart types: line and bar
- Comprehensive statistics
- Multiple display modes
- Auto terminal width detection
- TTL display
- Color-coded output

## Tips

1. **For Continuous Monitoring**: Use chart-only mode for cleaner display
   ```bash
   pingmon 8.8.8.8 --chart-only
   ```

2. **For Logging**: Use static mode and redirect output
   ```bash
   pingmon 8.8.8.8 --static-mode >> ping.log
   ```

3. **Custom Thresholds**: Adjust based on your network stability
   ```bash
   # Stable network: longer threshold
   pingmon 8.8.8.8 -t 300  # 5 minutes
   
   # Unstable network: shorter threshold
   pingmon 8.8.8.8 -t 30   # 30 seconds
   ```

4. **Multiple Monitors**: Run different instances for different hosts
   ```bash
   # Terminal 1
   pingmon google.com -t 60
   
   # Terminal 2
   pingmon-bar 1.1.1.1 -t 60
   ```

## Support

For issues, questions, or suggestions:
- Open an issue on GitHub
- Email: cumulus13@gmail.com
