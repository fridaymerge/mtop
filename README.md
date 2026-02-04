# mtop

A terminal resource cost monitor that puts a dollar value on CPU, RAM, and power usage.

## Features
- Real-time CPU/RAM/Network cost breakdown
- Per-core cost visualization
- Process list ranked by cost
- Built-in 1‑minute stress test (`t`)

## Requirements
- Rust toolchain (stable) with `cargo`

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Build
```bash
cargo build --release
```

Binary location:
```
target/release/mtop
```

## Run
```bash
cargo run --release
```

Or run the built binary:
```bash
./target/release/mtop
```

## Install (local)
```bash
cargo install --path .
```

Make sure Cargo’s bin directory is on your `PATH`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Controls
- `q` / `Esc` – Quit
- `j` / `k` – Scroll process list
- `r` – Refresh now
- `t` – Start/stop 1‑minute stress test

## Notes
- Hardware prices are refreshed at most once per day and cached locally.
