//! Tic-Tac-Foe game logic, terminal UI state, AI opponents, and online networking.

#![warn(missing_docs)]

extern crate core;

/// Artificial-intelligence players and shared game-tree abstractions.
pub mod ai;
/// Top-level application state and input handling.
pub mod app;
/// Classic and Ultimate tic-tac-toe board models.
pub mod game;
/// Peer-to-peer networking and wire protocol support.
pub mod network;
/// Menu and gameplay scene state.
pub mod scenes;
/// Terminal user-interface rendering.
pub mod ui;
/// Shared board-selection utilities.
pub mod utils;
