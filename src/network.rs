//! Peer-to-peer transport and synchronous application bridge.
//!
//! [`NetworkClient`](crate::network::NetworkClient) owns a Tokio runtime on a dedicated worker thread. The
//! synchronous TUI sends [`NetworkCommand`](crate::network::NetworkCommand) values and polls [`NetworkEvent`](crate::network::NetworkEvent)
//! values without blocking its render loop.

/// Serializable messages exchanged during handshakes and matches.
pub mod protocol;

use self::protocol::{
    GameMessage, HandshakeMessage, MoveMessage, PROTOCOL_VERSION, UltimateMoveMessage,
};
use crate::game::{GameVariant, Mark};
use iroh::{
    Endpoint, EndpointAddr,
    endpoint::{BindError, ConnectError, ConnectingError, Connection, presets},
};
use iroh_tickets::{ParseError, endpoint::EndpointTicket};
use std::{fmt, io, sync::mpsc, thread};
use tokio::{runtime::Builder, sync::mpsc as tokio_mpsc, task::JoinHandle};

/// Application-layer protocol negotiated by iroh connections.
pub const ALPN: &[u8] = b"/tic-tac-foe/1";

/// Commands sent from the synchronous application to the network worker.
#[derive(Clone, Debug, PartialEq)]
pub enum NetworkCommand {
    /// Hosts a match using the selected game variant.
    Host(GameVariant),
    /// Joins a match using an endpoint ticket and expected game variant.
    Join {
        /// Encoded iroh endpoint ticket supplied by the host.
        ticket: String,
        /// Game variant expected during the handshake.
        game: GameVariant,
    },
    /// Sends a classic tic-tac-toe move.
    SendMove(MoveMessage),
    /// Sends an Ultimate tic-tac-toe move.
    SendUltimateMove(UltimateMoveMessage),
    /// Marks the local player as ready for a rematch.
    SendRematchReady,
    /// Gives the first move to the remote player.
    YieldFirstMove,
    /// Concedes the active match.
    Concede,
    /// Closes the active session.
    Disconnect,
}

/// Events emitted by the network worker for the synchronous application.
#[derive(Clone, Debug, PartialEq)]
pub enum NetworkEvent {
    /// Reports a host ticket and whether public relay connectivity is ready.
    Hosting {
        /// Encoded iroh endpoint ticket to share with the joining player.
        ticket: String,
        /// Whether relay connectivity is available in addition to LAN discovery.
        relay_ready: bool,
    },
    /// Reports that a join attempt is in progress.
    Connecting,
    /// Reports a completed handshake and assigned player mark.
    Connected {
        /// Mark assigned to the local player.
        mark: Mark,
        /// Game variant accepted by both peers.
        game: GameVariant,
    },
    /// Delivers a validated classic move.
    MoveReceived(MoveMessage),
    /// Delivers a validated Ultimate move.
    UltimateMoveReceived(UltimateMoveMessage),
    /// Reports that the remote player requested a rematch.
    RematchReadyReceived,
    /// Reports that the remote player yielded the first move.
    YieldFirstMoveReceived,
    /// Reports that the remote player conceded.
    OpponentConceded,
    /// Reports that the remote player unexpectedly left the match.
    OpponentDisconnected,
    /// Confirms a locally requested disconnection.
    Disconnected,
    /// Reports an operation or protocol failure.
    Failed(String),
}

/// User-visible summary of the current networking state.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum NetworkStatus {
    /// No online operation is active.
    #[default]
    Idle,
    /// A local endpoint is waiting for an opponent.
    Hosting {
        /// Encoded endpoint ticket displayed to the user.
        ticket: String,
        /// Whether public relay connectivity is ready.
        relay_ready: bool,
    },
    /// The client is connecting to a host.
    Connecting,
    /// A peer session is active.
    Connected {
        /// Mark assigned to the local player.
        mark: Mark,
    },
    /// The remote player left an active match.
    OpponentDisconnected,
    /// The most recent network operation failed.
    Failed(String),
}

impl NetworkEvent {
    /// Converts status-bearing events into their visible application state.
    ///
    /// Match actions return `None` because they update gameplay rather than status.
    pub fn into_status(self) -> Option<NetworkStatus> {
        match self {
            NetworkEvent::Hosting {
                ticket,
                relay_ready,
            } => Some(NetworkStatus::Hosting {
                ticket,
                relay_ready,
            }),
            NetworkEvent::Connecting => Some(NetworkStatus::Connecting),
            NetworkEvent::Connected { mark, .. } => Some(NetworkStatus::Connected { mark }),
            NetworkEvent::MoveReceived(_)
            | NetworkEvent::UltimateMoveReceived(_)
            | NetworkEvent::RematchReadyReceived
            | NetworkEvent::YieldFirstMoveReceived
            | NetworkEvent::OpponentConceded => None,
            NetworkEvent::OpponentDisconnected => Some(NetworkStatus::OpponentDisconnected),
            NetworkEvent::Disconnected => Some(NetworkStatus::Idle),
            NetworkEvent::Failed(error) => Some(NetworkStatus::Failed(error)),
        }
    }
}

/// Encodes an iroh endpoint address as a shareable ticket.
pub fn encode_ticket(addr: EndpointAddr) -> String {
    EndpointTicket::new(addr).to_string()
}

/// Decodes a ticket, ignoring whitespace introduced by UI formatting.
pub fn decode_ticket(ticket: &str) -> Result<EndpointAddr, ParseError> {
    let ticket = ticket
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .parse::<EndpointTicket>()?;
    Ok(ticket.into())
}

/// Creates an iroh endpoint configured for the application protocol.
pub async fn create_endpoint() -> Result<Endpoint, BindError> {
    Endpoint::builder(presets::N0)
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
}

/// Creates a host endpoint and returns its current shareable ticket.
pub async fn create_host() -> Result<(Endpoint, String), BindError> {
    let endpoint = create_endpoint().await?;

    let ticket = encode_ticket(endpoint.addr());

    Ok((endpoint, ticket))
}

/// Connects an endpoint to a remote address using the application protocol.
pub async fn connect_to_host(
    endpoint: &Endpoint,
    addr: EndpointAddr,
) -> Result<Connection, ConnectError> {
    endpoint.connect(addr, ALPN).await
}

/// Waits for the next incoming connection, or returns `None` if the endpoint closes.
pub async fn accept_connection(endpoint: &Endpoint) -> Option<Result<Connection, ConnectingError>> {
    match endpoint.accept().await {
        Some(incoming) => Some(incoming.await),
        None => None,
    }
}

/// Synchronous handle to the asynchronous network worker.
///
/// The worker runs a current-thread Tokio runtime on a dedicated OS thread.
pub struct NetworkClient {
    command_tx: Option<tokio_mpsc::UnboundedSender<NetworkCommand>>,
    event_rx: mpsc::Receiver<NetworkEvent>,
    worker: Option<thread::JoinHandle<()>>,
}

impl NetworkClient {
    /// Starts the worker thread and its Tokio runtime.
    pub fn start() -> io::Result<Self> {
        let runtime = Builder::new_current_thread().enable_all().build()?;
        let (command_tx, command_rx) = tokio_mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel();
        let worker =
            thread::spawn(move || runtime.block_on(run_network_worker(command_rx, event_tx)));

        Ok(Self {
            command_tx: Some(command_tx),
            event_rx,
            worker: Some(worker),
        })
    }

    /// Queues a command for the network worker.
    pub fn send(
        &self,
        command: NetworkCommand,
    ) -> Result<(), tokio_mpsc::error::SendError<NetworkCommand>> {
        self.command_tx
            .as_ref()
            .expect("network client is shutting down")
            .send(command)
    }

    /// Returns the next pending worker event without blocking.
    pub fn try_recv(&self) -> Result<NetworkEvent, mpsc::TryRecvError> {
        self.event_rx.try_recv()
    }
}

impl Drop for NetworkClient {
    fn drop(&mut self) {
        self.command_tx.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct NetworkSession {
    endpoint: Endpoint,
    connection: Connection,
    mark: Mark,
    game: GameVariant,
}

struct OperationResult {
    id: u64,
    result: Result<NetworkSession, String>,
}

async fn run_network_worker(
    mut command_rx: tokio_mpsc::UnboundedReceiver<NetworkCommand>,
    event_tx: mpsc::Sender<NetworkEvent>,
) {
    let (result_tx, mut result_rx) = tokio_mpsc::unbounded_channel::<OperationResult>();
    let mut operation_id = 0;
    let mut pending: Option<JoinHandle<()>> = None;
    let mut session: Option<NetworkSession> = None;

    loop {
        let move_connection = session.as_ref().map(|session| session.connection.clone());

        tokio::select! {
            command = command_rx.recv() => {
                let Some(command) = command else { break };
                match command {
                    NetworkCommand::SendUltimateMove(message) => {
                        send_worker_game_message(
                            &mut session,
                            &event_tx,
                            GameMessage::UltimateMove { position: message },
                        )
                        .await;
                    }
                    NetworkCommand::SendMove(message) => {
                        send_worker_game_message(
                            &mut session,
                            &event_tx,
                            GameMessage::Move { position: message },
                        )
                        .await;
                    }
                    NetworkCommand::SendRematchReady => {
                        send_worker_game_message(
                            &mut session,
                            &event_tx,
                            GameMessage::RematchReady,
                        )
                        .await;
                    }
                    NetworkCommand::YieldFirstMove => {
                        send_worker_game_message(
                            &mut session,
                            &event_tx,
                            GameMessage::YieldFirstMove,
                        )
                        .await;
                    }
                    NetworkCommand::Concede => {
                        send_worker_game_message(
                            &mut session,
                            &event_tx,
                            GameMessage::Concede,
                        )
                        .await;
                    }
                    command => {
                        operation_id += 1;
                        if let Some(task) = pending.take() {
                            task.abort();
                        }
                        if let Some(session) = session.take() {
                            close_session(session).await;
                        }

                        match command {
                            NetworkCommand::Host(game) => {
                                let result_tx = result_tx.clone();
                                let event_tx = event_tx.clone();
                                let id = operation_id;
                                pending = Some(tokio::spawn(async move {
                                    let result = host_session(game, event_tx).await;
                                    let _ = result_tx.send(OperationResult { id, result });
                                }));
                            }
                            NetworkCommand::Join { ticket, game } => {
                                let result_tx = result_tx.clone();
                                let event_tx = event_tx.clone();
                                let id = operation_id;
                                pending = Some(tokio::spawn(async move {
                                    let result = join_session(ticket, game, event_tx).await;
                                    let _ = result_tx.send(OperationResult { id, result });
                                }));
                            }
                            NetworkCommand::Disconnect => {
                                let _ = event_tx.send(NetworkEvent::Disconnected);
                            }
                            NetworkCommand::SendMove(_)
                            | NetworkCommand::SendUltimateMove(_)
                            | NetworkCommand::SendRematchReady
                            | NetworkCommand::YieldFirstMove
                            | NetworkCommand::Concede => unreachable!(),
                        }
                    }
                }
            }
            result = result_rx.recv() => {
                let Some(OperationResult { id, result }) = result else { break };
                if id != operation_id {
                    continue;
                }
                pending = None;
                match result {
                    Ok(new_session) => {
                        let mark = new_session.mark;
                        let game = new_session.game;
                        session = Some(new_session);
                        let _ = event_tx.send(NetworkEvent::Connected { mark, game });
                    }
                    Err(error) => {
                        let _ = event_tx.send(NetworkEvent::Failed(error));
                    }
                }
            }
            result = receive_session_game_message(move_connection) => {
                match result {
                    Ok(GameMessage::Move { position }) => {
                        let _ = event_tx.send(NetworkEvent::MoveReceived(position));
                    }
                    Ok(GameMessage::UltimateMove { position }) => {
                        let _ = event_tx.send(NetworkEvent::UltimateMoveReceived(position));
                    }
                    Ok(GameMessage::RematchReady) => {
                        let _ = event_tx.send(NetworkEvent::RematchReadyReceived);
                    }
                    Ok(GameMessage::YieldFirstMove) => {
                        let _ = event_tx.send(NetworkEvent::YieldFirstMoveReceived);
                    }
                    Ok(GameMessage::Concede) => {
                        let _ = event_tx.send(NetworkEvent::OpponentConceded);
                    }
                    Err(ReceiveGameMessageError::Disconnected(_)) => {
                        if let Some(session) = session.take() {
                            close_session(session).await;
                        }
                        let _ = event_tx.send(NetworkEvent::OpponentDisconnected);
                    }
                    Err(ReceiveGameMessageError::InvalidMessage(error)) => {
                        if let Some(session) = session.take() {
                            close_session(session).await;
                        }
                        let _ = event_tx.send(NetworkEvent::Failed(error));
                    }
                }
            }
        }
    }

    if let Some(task) = pending {
        task.abort();
    }
    if let Some(session) = session {
        close_session(session).await;
    }
}

async fn send_worker_game_message(
    session: &mut Option<NetworkSession>,
    event_tx: &mpsc::Sender<NetworkEvent>,
    message: GameMessage,
) {
    let result = match session.as_ref() {
        Some(session) => send_game_message(&session.connection, &message).await,
        None => Err("no active network session".to_string()),
    };
    if let Err(error) = result {
        if let Some(session) = session.take() {
            close_session(session).await;
        }
        let _ = event_tx.send(NetworkEvent::Failed(error));
    }
}

async fn receive_session_game_message(
    connection: Option<Connection>,
) -> Result<GameMessage, ReceiveGameMessageError> {
    match connection {
        Some(connection) => receive_game_message(&connection).await,
        None => std::future::pending().await,
    }
}

async fn close_session(session: NetworkSession) {
    session.connection.close(0u32.into(), b"disconnected");
    session.endpoint.close().await;
}

async fn host_session(
    game: GameVariant,
    event_tx: mpsc::Sender<NetworkEvent>,
) -> Result<NetworkSession, String> {
    let (endpoint, ticket) = create_host().await.map_err(|error| error.to_string())?;
    let _ = event_tx.send(NetworkEvent::Hosting {
        ticket,
        relay_ready: false,
    });

    let connection = tokio::select! {
        connection = wait_for_connection(&endpoint) => connection?,
        () = endpoint.online() => {
            let ticket = encode_ticket(endpoint.addr());
            let _ = event_tx.send(NetworkEvent::Hosting {
                ticket,
                relay_ready: true,
            });
            wait_for_connection(&endpoint).await?
        }
    };

    let mark = perform_host_handshake(&connection, game).await?;

    Ok(NetworkSession {
        endpoint,
        connection,
        mark,
        game,
    })
}

async fn wait_for_connection(endpoint: &Endpoint) -> Result<Connection, String> {
    accept_connection(endpoint)
        .await
        .ok_or_else(|| "endpoint closed while hosting".to_string())?
        .map_err(|error| error.to_string())
}

async fn join_session(
    ticket: String,
    game: GameVariant,
    event_tx: mpsc::Sender<NetworkEvent>,
) -> Result<NetworkSession, String> {
    let _ = event_tx.send(NetworkEvent::Connecting);
    let addr = decode_ticket(&ticket).map_err(|error| error.to_string())?;
    let endpoint = create_endpoint().await.map_err(|error| error.to_string())?;
    let connection = connect_to_host(&endpoint, addr)
        .await
        .map_err(|error| error.to_string())?;
    let mark = match perform_join_handshake(&connection, game).await {
        Ok(mark) => mark,
        Err(error) => {
            connection.close(0u32.into(), b"handshake failed");
            endpoint.close().await;
            return Err(error);
        }
    };

    Ok(NetworkSession {
        endpoint,
        connection,
        mark,
        game,
    })
}

const MAX_HANDSHAKE_SIZE: usize = 1024;
const MAX_GAME_MESSAGE_SIZE: usize = 128;

async fn perform_host_handshake(
    connection: &Connection,
    game: GameVariant,
) -> Result<Mark, String> {
    let (mut send, mut receive) = connection
        .accept_bi()
        .await
        .map_err(|error| error.to_string())?;
    let bytes = receive
        .read_to_end(MAX_HANDSHAKE_SIZE)
        .await
        .map_err(|error| error.to_string())?;

    let validation = match protocol::decode(&bytes) {
        Ok(HandshakeMessage::Hello {
            protocol_version,
            game: requested_game,
        }) if protocol_version == PROTOCOL_VERSION && requested_game == game => Ok(()),
        Ok(HandshakeMessage::Hello {
            protocol_version, ..
        }) if protocol_version != PROTOCOL_VERSION => {
            Err(format!("unsupported protocol version: {protocol_version}"))
        }
        Ok(HandshakeMessage::Hello {
            game: requested_game,
            ..
        }) => Err(format!(
            "game variant mismatch: expected {game:?}, received {requested_game:?}"
        )),
        Ok(HandshakeMessage::Welcome { .. } | HandshakeMessage::Rejected { .. }) => {
            Err("expected hello handshake message".to_string())
        }
        Err(error) => Err(format!("invalid handshake message: {error}")),
    };

    if let Err(error) = validation {
        let response = protocol::encode(&HandshakeMessage::rejected(error.clone()))
            .map_err(|encode_error| encode_error.to_string())?;
        send.write_all(&response)
            .await
            .map_err(|write_error| write_error.to_string())?;
        send.finish()
            .map_err(|finish_error| finish_error.to_string())?;
        connection.closed().await;
        return Err(error);
    }

    let response = protocol::encode(&HandshakeMessage::welcome(Mark::O, game))
        .map_err(|error| error.to_string())?;
    send.write_all(&response)
        .await
        .map_err(|error| error.to_string())?;
    send.finish().map_err(|error| error.to_string())?;

    Ok(Mark::X)
}

async fn perform_join_handshake(
    connection: &Connection,
    game: GameVariant,
) -> Result<Mark, String> {
    let (mut send, mut receive) = connection
        .open_bi()
        .await
        .map_err(|error| error.to_string())?;
    let hello =
        protocol::encode(&HandshakeMessage::hello(game)).map_err(|error| error.to_string())?;
    send.write_all(&hello)
        .await
        .map_err(|error| error.to_string())?;
    send.finish().map_err(|error| error.to_string())?;

    let bytes = receive
        .read_to_end(MAX_HANDSHAKE_SIZE)
        .await
        .map_err(|error| error.to_string())?;
    match protocol::decode(&bytes).map_err(|error| error.to_string())? {
        HandshakeMessage::Welcome {
            protocol_version,
            mark,
            game: accepted_game,
        } if protocol_version == PROTOCOL_VERSION && accepted_game == game => Ok(mark),
        HandshakeMessage::Welcome {
            protocol_version, ..
        } if protocol_version != PROTOCOL_VERSION => {
            Err(format!("unsupported protocol version: {protocol_version}"))
        }
        HandshakeMessage::Welcome {
            game: accepted_game,
            ..
        } => Err(format!(
            "game variant mismatch: expected {game:?}, received {accepted_game:?}"
        )),
        HandshakeMessage::Rejected { reason } => Err(reason),
        HandshakeMessage::Hello { .. } => Err("expected welcome handshake message".to_string()),
    }
}

/// Serializes and sends one game message on a unidirectional stream.
pub async fn send_game_message(
    connection: &Connection,
    message: &GameMessage,
) -> Result<(), String> {
    let bytes = protocol::encode_game_message(message).map_err(|error| error.to_string())?;
    let mut send = connection
        .open_uni()
        .await
        .map_err(|error| error.to_string())?;
    send.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    send.finish().map_err(|error| error.to_string())
}

/// Failure encountered while receiving a game message.
#[derive(Debug)]
pub enum ReceiveGameMessageError {
    /// The connection closed before another stream could be accepted.
    Disconnected(String),
    /// A stream was received but its payload was invalid.
    InvalidMessage(String),
}

impl fmt::Display for ReceiveGameMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected(error) | Self::InvalidMessage(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for ReceiveGameMessageError {}

/// Receives and decodes one game message from a unidirectional stream.
pub async fn receive_game_message(
    connection: &Connection,
) -> Result<GameMessage, ReceiveGameMessageError> {
    let mut receive = connection
        .accept_uni()
        .await
        .map_err(|error| ReceiveGameMessageError::Disconnected(error.to_string()))?;
    let bytes = receive
        .read_to_end(MAX_GAME_MESSAGE_SIZE)
        .await
        .map_err(|error| ReceiveGameMessageError::InvalidMessage(error.to_string()))?;
    protocol::decode_game_message(&bytes)
        .map_err(|error| ReceiveGameMessageError::InvalidMessage(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_game_message_is_sent_over_connection() {
        let host_endpoint = create_endpoint().await.unwrap();
        let joiner_endpoint = create_endpoint().await.unwrap();
        let host_addr = host_endpoint.addr();

        let (host_connection, joiner_connection) = tokio::join!(
            async { wait_for_connection(&host_endpoint).await.unwrap() },
            async { connect_to_host(&joiner_endpoint, host_addr).await.unwrap() }
        );
        let message = GameMessage::RematchReady;

        let (sent, received) = tokio::join!(
            send_game_message(&joiner_connection, &message),
            receive_game_message(&host_connection)
        );

        sent.unwrap();
        assert_eq!(received.unwrap(), message);

        host_connection.close(0u32.into(), b"test complete");
        joiner_connection.close(0u32.into(), b"test complete");
        host_endpoint.close().await;
        joiner_endpoint.close().await;
    }

    #[test]
    fn test_worker_handles_disconnect() {
        let client = NetworkClient::start().unwrap();

        client.send(NetworkCommand::Disconnect).unwrap();

        let event = client
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        assert_eq!(event, NetworkEvent::Disconnected);
    }

    #[test]
    fn test_hosting_emits_lan_ticket_before_relay_is_ready() {
        let client = NetworkClient::start().unwrap();

        client
            .send(NetworkCommand::Host(GameVariant::Classic))
            .unwrap();

        let event = client
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        let NetworkEvent::Hosting {
            ticket,
            relay_ready,
        } = event
        else {
            panic!("expected hosting event");
        };
        assert!(!relay_ready);

        let addr = decode_ticket(&ticket).unwrap();
        assert!(addr.ip_addrs().next().is_some());
    }

    #[test]
    fn test_worker_reports_invalid_ticket() {
        let client = NetworkClient::start().unwrap();

        client
            .send(NetworkCommand::Join {
                ticket: "invalid ticket".to_string(),
                game: GameVariant::Classic,
            })
            .unwrap();

        let connecting = client
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        assert_eq!(connecting, NetworkEvent::Connecting);

        let failed = client
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(matches!(failed, NetworkEvent::Failed(error) if !error.is_empty()));
    }

    #[test]
    fn test_workers_complete_handshake_with_opposite_marks() {
        let host = NetworkClient::start().unwrap();
        let joiner = NetworkClient::start().unwrap();

        host.send(NetworkCommand::Host(GameVariant::Classic))
            .unwrap();
        let NetworkEvent::Hosting { ticket, .. } = host
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
        else {
            panic!("expected hosting event");
        };

        joiner
            .send(NetworkCommand::Join {
                ticket,
                game: GameVariant::Classic,
            })
            .unwrap();
        assert_eq!(
            joiner
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap(),
            NetworkEvent::Connecting
        );

        assert_eq!(wait_for_connected_mark(&host), Mark::X);
        assert_eq!(wait_for_connected_mark(&joiner), Mark::O);
    }

    #[test]
    fn test_workers_exchange_ultimate_move() {
        let host = NetworkClient::start().unwrap();
        let joiner = NetworkClient::start().unwrap();

        host.send(NetworkCommand::Host(GameVariant::Ultimate))
            .unwrap();
        let NetworkEvent::Hosting { ticket, .. } = host
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
        else {
            panic!("expected hosting event");
        };
        joiner
            .send(NetworkCommand::Join {
                ticket,
                game: GameVariant::Ultimate,
            })
            .unwrap();
        assert_eq!(
            joiner
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap(),
            NetworkEvent::Connecting
        );
        wait_for_connected_mark(&host);
        wait_for_connected_mark(&joiner);

        let position = UltimateMoveMessage::new(2, 1, 0, 2).unwrap();
        joiner
            .send(NetworkCommand::SendUltimateMove(position))
            .unwrap();

        assert_eq!(
            host.event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            NetworkEvent::UltimateMoveReceived(position)
        );
    }

    #[test]
    fn test_workers_reject_mismatched_game_variants() {
        let host = NetworkClient::start().unwrap();
        let joiner = NetworkClient::start().unwrap();

        host.send(NetworkCommand::Host(GameVariant::Classic))
            .unwrap();
        let NetworkEvent::Hosting { ticket, .. } = host
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
        else {
            panic!("expected hosting event");
        };

        joiner
            .send(NetworkCommand::Join {
                ticket,
                game: GameVariant::Ultimate,
            })
            .unwrap();
        assert_eq!(
            joiner
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap(),
            NetworkEvent::Connecting
        );

        let host_event = host
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let joiner_event = joiner
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();

        let NetworkEvent::Failed(host_error) = host_event else {
            panic!("expected host handshake failure");
        };
        let NetworkEvent::Failed(joiner_error) = joiner_event else {
            panic!("expected joiner handshake failure");
        };
        assert!(host_error.contains("game variant mismatch"));
        assert_eq!(joiner_error, host_error);
    }

    #[test]
    fn test_workers_exchange_game_events_and_disconnect() {
        let host = NetworkClient::start().unwrap();
        let joiner = NetworkClient::start().unwrap();

        host.send(NetworkCommand::Host(GameVariant::Classic))
            .unwrap();
        let NetworkEvent::Hosting { ticket, .. } = host
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
        else {
            panic!("expected hosting event");
        };
        joiner
            .send(NetworkCommand::Join {
                ticket,
                game: GameVariant::Classic,
            })
            .unwrap();
        assert_eq!(
            joiner
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap(),
            NetworkEvent::Connecting
        );
        wait_for_connected_mark(&host);
        wait_for_connected_mark(&joiner);

        let message = MoveMessage::new(2, 1).unwrap();
        joiner.send(NetworkCommand::SendMove(message)).unwrap();

        assert_eq!(wait_for_received_move(&host), message);

        host.send(NetworkCommand::YieldFirstMove).unwrap();
        assert_eq!(
            joiner
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            NetworkEvent::YieldFirstMoveReceived
        );

        joiner.send(NetworkCommand::SendRematchReady).unwrap();
        assert_eq!(
            host.event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            NetworkEvent::RematchReadyReceived
        );

        host.send(NetworkCommand::Concede).unwrap();
        assert_eq!(
            joiner
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            NetworkEvent::OpponentConceded
        );

        host.send(NetworkCommand::Disconnect).unwrap();
        assert_eq!(
            host.event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            NetworkEvent::Disconnected
        );
        assert_eq!(
            joiner
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap(),
            NetworkEvent::OpponentDisconnected
        );
    }

    fn wait_for_connected_mark(client: &NetworkClient) -> Mark {
        loop {
            match client
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap()
            {
                NetworkEvent::Connected { mark, .. } => return mark,
                NetworkEvent::Hosting { .. } => {}
                event => panic!("expected connected event, got {event:?}"),
            }
        }
    }

    fn wait_for_received_move(client: &NetworkClient) -> MoveMessage {
        loop {
            match client
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap()
            {
                NetworkEvent::MoveReceived(message) => return message,
                NetworkEvent::Hosting { .. } => {}
                event => panic!("expected move event, got {event:?}"),
            }
        }
    }

    #[test]
    fn test_ticket_round_trip() {
        let key = iroh::SecretKey::generate();
        let addr = EndpointAddr::new(key.public());

        let ticket = encode_ticket(addr.clone());
        let decoded = decode_ticket(&ticket).unwrap();

        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_decode_ticket_ignores_whitespace() {
        let key = iroh::SecretKey::generate();
        let addr = EndpointAddr::new(key.public());
        let ticket = encode_ticket(addr.clone());
        let midpoint = ticket.len() / 2;
        let ticket = format!("  {}\n  {}\t", &ticket[..midpoint], &ticket[midpoint..]);

        let decoded = decode_ticket(&ticket).unwrap();

        assert_eq!(decoded, addr);
    }
}
