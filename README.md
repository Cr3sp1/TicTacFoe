# Tic-Tac-Foe

A terminal-based Tic-Tac-Toe game built with Rust and Ratatui.

## Features

- **Local PvP**: Play against another person on the same computer.
- **Play vs AI**: Challenge a simple AI opponent.
- **Intuitive TUI**: Clean terminal user interface with keyboard navigation.

## Requirements
Linux, Windows or macOS operating system.
Installing via Cargo or from source additionally requires Rust 1.70 or higher and Cargo already installed.

## Installation

### Download Pre-built Binary (Recommended)

Download the appropriate binary for your platform from the [latest release](https://github.com/Cr3sp1/TicTacFoe/releases/latest):

- **Linux**: `tic-tac-foe-linux-x86_64`
- **macOS (Intel)**: `tic-tac-foe-macos-x86_64`
- **macOS (Apple Silicon)**: `tic-tac-foe-macos-aarch64`
- **Windows**: `tic-tac-foe-windows-x86_64.exe`

After downloading, you may need to make the binary executable (Linux/macOS only):

```bash
chmod +x tic-tac-foe-linux-x86_64  # or the macOS variant
```

### Via Cargo

```bash
cargo install tic-tac-foe
```

### From Source

```bash
git clone https://github.com/Cr3sp1/tic-tac-foe.git
cd tic-tac-foe
cargo build --release
```

The compiled binary will be in `target/release/tic-tac-foe`.

## Usage

### If you downloaded the pre-built binary:

```bash
./tic-tac-foe-linux-x86_64  # Linux
./tic-tac-foe-macos-x86_64  # macOS (Intel)
./tic-tac-foe-macos-aarch64 # macOS (Apple Silicon)
tic-tac-foe-windows-x86_64.exe  # Windows
```

### If installed via Cargo:

```bash
tic-tac-foe
```

### If built from source:

```bash
cargo run
```

Or directly run the compiled binary:

```bash
./target/release/tic-tac-foe
```

## To Do

- Implement peer to peer connection to play against other people remotely.
- Add Ultimate tic-tac-toe to game modes.
- Implement Monte Carlo tree-search algorithm to provide a strong enemy AI.

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework.
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation.
- [rand](https://github.com/rust-random/rand) - Random number generation.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Developed by [Cr3sp1](https://github.com/Cr3sp1).