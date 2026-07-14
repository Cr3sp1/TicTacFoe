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
    fn malformed_message_is_rejected() {
        let result = decode(br#"{"type":"welcome","protocol_version":1}"#);

        assert!(result.is_err());
    }
}
