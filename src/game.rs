pub mod base;
pub mod ultimate;

use std::fmt;

/// Represents a player's mark (X or O) on the tic-tac-toe board.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mark {
    X,
    O,
}

impl fmt::Display for Mark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mark::X => write!(f, "X"),
            Mark::O => write!(f, "O"),
        }
    }
}
