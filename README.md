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

### Via Cargo

```bash
cargo install tic-tac-foe
```

### From Source

```bash
git clone https://github.com/yourusername/tic-tac-foe.git
cd tic-tac-foe
cargo build --release
```

The compiled binary will be in `target/release/tic-tac-foe`.

## Usage

Run the game:

```bash
cargo run
```

Or if installed via Cargo:

```bash
tic-tac-foe
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