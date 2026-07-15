use std::fmt;

use serde::{Deserialize, Serialize};

use crate::game::{GameVariant, Mark};

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HandshakeMessage {
    Hello {
        protocol_version: u16,
        game: GameVariant,
    },
    Welcome {
        protocol_version: u16,
        mark: Mark,
        game: GameVariant,
    },
    Rejected {
        reason: String,
    },
}

impl HandshakeMessage {
    pub fn hello(game: GameVariant) -> Self {
        Self::Hello {
            protocol_version: PROTOCOL_VERSION,
            game,
        }
    }

    pub fn welcome(mark: Mark, game: GameVariant) -> Self {
        Self::Welcome {
            protocol_version: PROTOCOL_VERSION,
            mark,
            game,
        }
    }

    pub fn rejected(reason: impl Into<String>) -> Self {
        Self::Rejected {
            reason: reason.into(),
        }
    }
}

pub fn encode(message: &HandshakeMessage) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(message)
}

pub fn decode(bytes: &[u8]) -> Result<HandshakeMessage, serde_json::Error> {
    serde_json::from_slice(bytes)
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "MoveCoordinates")]
pub struct MoveMessage {
    row: u8,
    col: u8,
}

impl MoveMessage {
    pub fn new(row: u8, col: u8) -> Result<Self, InvalidMoveCoordinates> {
        if row < 3 && col < 3 {
            Ok(Self { row, col })
        } else {
            Err(InvalidMoveCoordinates { row, col })
        }
    }

    pub fn row(self) -> usize {
        self.row.into()
    }

    pub fn col(self) -> usize {
        self.col.into()
    }
}

#[derive(Deserialize)]
struct MoveCoordinates {
    row: u8,
    col: u8,
}

impl TryFrom<MoveCoordinates> for MoveMessage {
    type Error = InvalidMoveCoordinates;

    fn try_from(coordinates: MoveCoordinates) -> Result<Self, Self::Error> {
        Self::new(coordinates.row, coordinates.col)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvalidMoveCoordinates {
    row: u8,
    col: u8,
}

impl fmt::Display for InvalidMoveCoordinates {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "move coordinates ({}, {}) are outside the board",
            self.row, self.col
        )
    }
}

impl std::error::Error for InvalidMoveCoordinates {}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(try_from = "UltimateMoveCoordinates")]
pub struct UltimateMoveMessage {
    board_row: u8,
    board_col: u8,
    cell_row: u8,
    cell_col: u8,
}

impl UltimateMoveMessage {
    pub fn new(
        board_row: u8,
        board_col: u8,
        cell_row: u8,
        cell_col: u8,
    ) -> Result<Self, InvalidUltimateMoveCoordinates> {
        if board_row < 3 && board_col < 3 && cell_row < 3 && cell_col < 3 {
            Ok(Self {
                board_row,
                board_col,
                cell_row,
                cell_col,
            })
        } else {
            Err(InvalidUltimateMoveCoordinates {
                board_row,
                board_col,
                cell_row,
                cell_col,
            })
        }
    }

    pub fn board_row(self) -> usize {
        self.board_row.into()
    }
    pub fn board_col(self) -> usize {
        self.board_col.into()
    }
    pub fn cell_row(self) -> usize {
        self.cell_row.into()
    }
    pub fn cell_col(self) -> usize {
        self.cell_col.into()
    }
}

#[derive(Deserialize)]
struct UltimateMoveCoordinates {
    board_row: u8,
    board_col: u8,
    cell_row: u8,
    cell_col: u8,
}

impl TryFrom<UltimateMoveCoordinates> for UltimateMoveMessage {
    type Error = InvalidUltimateMoveCoordinates;

    fn try_from(coordinates: UltimateMoveCoordinates) -> Result<Self, Self::Error> {
        Self::new(
            coordinates.board_row,
            coordinates.board_col,
            coordinates.cell_row,
            coordinates.cell_col,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvalidUltimateMoveCoordinates {
    board_row: u8,
    board_col: u8,
    cell_row: u8,
    cell_col: u8,
}

impl fmt::Display for InvalidUltimateMoveCoordinates {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "ultimate move coordinates ({}, {}, {}, {}) are outside the board",
            self.board_row, self.board_col, self.cell_row, self.cell_col
        )
    }
}

impl std::error::Error for InvalidUltimateMoveCoordinates {}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameMessage {
    Move { position: MoveMessage },
    UltimateMove { position: UltimateMoveMessage },
    RematchReady,
    YieldFirstMove,
    Concede,
}

pub fn encode_game_message(message: &GameMessage) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(message)
}

pub fn decode_game_message(bytes: &[u8]) -> Result<GameMessage, serde_json::Error> {
    serde_json::from_slice(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejection_round_trips_with_reason() {
        let message = HandshakeMessage::rejected("game variant mismatch");

        let encoded = encode(&message).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn hello_round_trips() {
        let message = HandshakeMessage::hello(GameVariant::Classic);

        let encoded = encode(&message).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn welcome_round_trips_with_assigned_mark() {
        let message = HandshakeMessage::welcome(Mark::O, GameVariant::Ultimate);

        let encoded = encode(&message).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn move_round_trips_with_valid_coordinates() {
        let message = MoveMessage::new(2, 1).unwrap();

        let game_message = GameMessage::Move { position: message };
        let encoded = encode_game_message(&game_message).unwrap();
        let decoded = decode_game_message(&encoded).unwrap();

        assert_eq!(decoded, game_message);
    }

    #[test]
    fn move_outside_board_is_rejected_while_decoding() {
        let result = decode_game_message(br#"{"type":"move","position":{"row":3,"col":0}}"#);

        assert!(result.is_err());
    }

    #[test]
    fn ultimate_move_round_trips_with_valid_coordinates() {
        let position = UltimateMoveMessage::new(2, 1, 0, 2).unwrap();
        let message = GameMessage::UltimateMove { position };

        let encoded = encode_game_message(&message).unwrap();
        let decoded = decode_game_message(&encoded).unwrap();

        assert_eq!(decoded, message);
        assert_eq!(position.board_row(), 2);
        assert_eq!(position.board_col(), 1);
        assert_eq!(position.cell_row(), 0);
        assert_eq!(position.cell_col(), 2);
    }

    #[test]
    fn ultimate_move_outside_board_is_rejected_while_decoding() {
        let result = decode_game_message(
            br#"{"type":"ultimate_move","position":{"board_row":1,"board_col":0,"cell_row":3,"cell_col":2}}"#,
        );

        assert!(result.is_err());
    }

    #[test]
    fn game_actions_round_trip() {
        for message in [
            GameMessage::RematchReady,
            GameMessage::YieldFirstMove,
            GameMessage::Concede,
        ] {
            let encoded = encode_game_message(&message).unwrap();
            let decoded = decode_game_message(&encoded).unwrap();

            assert_eq!(decoded, message);
        }
    }

    #[test]
    fn malformed_message_is_rejected() {
        let result = decode(br#"{"type":"welcome","protocol_version":1}"#);

        assert!(result.is_err());
    }
}
