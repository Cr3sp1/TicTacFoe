#[derive(Clone, Debug, PartialEq)]
pub enum NetworkCommand {
    Host,
    Join(String),
    Disconnect,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkEvent {
    Hosting { ticket: String },
    Connecting,
    Connected,
    Disconnected,
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_command_stores_ticket() {
        let command = NetworkCommand::Join("ticket".to_string());

        assert_eq!(command, NetworkCommand::Join("ticket".to_string()));
    }

    #[test]
    fn test_hosting_event_stores_ticket() {
        let event = NetworkEvent::Hosting {
            ticket: "ticket".to_string(),
        };

        assert_eq!(
            event,
            NetworkEvent::Hosting {
                ticket: "ticket".to_string()
            }
        );
    }
}
