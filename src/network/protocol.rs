use std::fmt;

use serde::{Deserialize, Serialize};

use crate::game::Mark;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HandshakeMessage {
    Hello { protocol_version: u16 },
    Welcome { protocol_version: u16, mark: Mark },
}

impl HandshakeMessage {
    pub fn hello() -> Self {
        Self::Hello {
            protocol_version: PROTOCOL_VERSION,
        }
    }

    pub fn welcome(mark: Mark) -> Self {
        Self::Welcome {
            protocol_version: PROTOCOL_VERSION,
            mark,
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameMessage {
    Move { position: MoveMessage },
    RematchReady,
    YieldFirstMove,
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
    fn hello_round_trips() {
        let message = HandshakeMessage::hello();

        let encoded = encode(&message).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(decoded, message);
    }

    #[test]
    fn welcome_round_trips_with_assigned_mark() {
        let message = HandshakeMessage::welcome(Mark::O);

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
    fn rematch_and_yield_messages_round_trip() {
        for message in [GameMessage::RematchReady, GameMessage::YieldFirstMove] {
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
