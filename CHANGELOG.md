# Changelog

## [1.1.0] - 2026/07/15

### Added
- Peer-to-peer online matches for classic and Ultimate tic-tac-toe using iroh [#1](https://github.com/Cr3sp1/TicTacFoe/pull/1).
- LAN-first connectivity with public relay fallback and shareable endpoint tickets [#1](https://github.com/Cr3sp1/TicTacFoe/pull/1).
- Online match hosting, joining, game-variant negotiation, and handshake error reporting [#1](https://github.com/Cr3sp1/TicTacFoe/pull/1).
- Synchronized moves, first-move yielding, concessions, two-player rematches, and opponent disconnect handling [#1](https://github.com/Cr3sp1/TicTacFoe/pull/1).

### Changed
- Network work runs asynchronously on a lazily started Tokio worker while the TUI remains synchronous [#1](https://github.com/Cr3sp1/TicTacFoe/pull/1).

## [1.0.0] - 2026/02/28

### Added
- Strong AI that implements the Monte Carlo Tree Search algorithm [#3](https://github.com/Cr3sp1/TicTacFoe/pull/3).
- AI vs AI mode [#3](https://github.com/Cr3sp1/TicTacFoe/pull/3).
- Different AI options (simple, random) [#3](https://github.com/Cr3sp1/TicTacFoe/pull/3).
- AI for Ultimate tic-tac-toe [#3](https://github.com/Cr3sp1/TicTacFoe/pull/3).

## [0.2.0] - 2026/01/06

### Added
- Improved ui.
- Ultimate tic-tac-toe [#2](https://github.com/Cr3sp1/TicTacFoe/pull/2).

### Bug Fixes
- Fix link to binaries in readme.


## [0.1.1] - 2026/01/06

### Bug Fixes
- On Windows kwy presses were counted twice


## [0.1.0] - 2026/01/06

### Added
- Initial release
- Basic tic-tac-toe gameplay
- TUI interface with ratatui