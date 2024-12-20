# Chess Engine

A powerful Chinese Chess (Xiangqi) engine written in Rust, featuring:
- Alpha-beta pruning with negamax search
- Move ordering with MVV-LVA
- Transposition tables
- Iterative deepening
- Killer moves and history heuristics

## Installation from Source

1. Install Rust and Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone the repository:
```bash
git clone https://github.com/hienduc/chess_engine.git
cd xiangqi-engine
```

3. Build the project:
```bash
cargo build --release
```

The compiled binary will be in `target/release/chess_engine`

## Download

You can download pre-built binaries from the [Releases](https://github.com/hien-duc/chess_engine/releases/tag/0.1.0) page.

## Usage

The engine uses standard I/O for communication and is compatible with most Chinese Chess GUIs that support the UCI protocol.


