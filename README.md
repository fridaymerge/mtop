# mtop - Money Top 💰

A TUI-based resource monitor that shows your computer's resource usage in terms of **money**!

Instead of seeing "CPU: 75%", you see "CPU: $0.75 / $1.00" - because why not translate your hardware usage into cold, hard cash?

## Two Versions Available

### 🦀 Rust Version (Recommended)
- **10-100x more efficient** (~1-2MB RAM vs 50-100MB)
- **Native performance** with minimal CPU overhead
- **Single binary** - no dependencies needed
- See [README_RUST.md](README_RUST.md) for build instructions

### 🐍 Python Version
- **Quick to modify** for experimentation
- **No compilation** needed
- Uses more resources but easier to hack on

## Features

- 💻 **Hardware Detection**: Automatically detects your CPU, RAM, and disk specs
- 💰 **Cost Estimation**: Estimates hardware costs based on market prices
- ⚡ **Power Consumption**: Calculates electricity costs in real-time
- 📊 **Real-time Monitoring**: Live updates of resource usage in dollars
- 🔥 **Per-Core CPU Usage**: See which cores are burning money with btop-style dot bars
- 📈 **Process Cost Ranking**: Hardware depreciation + power cost per process
- 🎨 **btop-Style UI**: Respects your terminal theme, dot-matrix visualizations
- 🚀 **Minimal Overhead**: Especially the Rust version!

## Quick Start

### Rust (Recommended)
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and run
cargo build --release
./target/release/mtop
```

### Python
```bash
pip install -r requirements.txt
python mtop.py
```

## Keybindings

- `q` - Quit
- `r` - Force refresh
- `↑`/`k` - Scroll up (process list)
- `↓`/`j` - Scroll down (process list)

## How It Works

1. **Hardware Detection**: Uses `psutil` and system calls to detect your hardware specs
2. **Cost Estimation**: Estimates costs based on typical market prices:
   - CPUs: $150-$3000 depending on model
   - RAM: ~$7/GB
   - Disk: ~$0.08/GB (SSD pricing)
3. **Usage Calculation**: Converts resource % into dollar value
   - Example: If your CPU is worth $500 and running at 50%, you're "using" $250 worth of CPU

## Customization

Want to set custom hardware costs? Edit the `hardware.py` file and modify the estimation functions, or add a config file (future feature).

## Screenshots

See your expensive processes burning through your hardware investment! 🔥💸

## Why?

Because regular resource monitors are boring. This one makes you feel financially invested in your system's performance.
