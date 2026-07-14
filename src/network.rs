pub mod protocol;

use self::protocol::{HandshakeMessage, PROTOCOL_VERSION};
use crate::game::Mark;
use iroh::{
    Endpoint, EndpointAddr,
    endpoint::{BindError, ConnectError, ConnectingError, Connection, presets},
};
use iroh_tickets::{ParseError, endpoint::EndpointTicket};
use std::{io, sync::mpsc, thread};
use tokio::{runtime::Builder, sync::mpsc as tokio_mpsc, task::JoinHandle};

pub const ALPN: &[u8] = b"/tic-tac-foe/1";

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkCommand {
    Host,
    Join(String),
    Disconnect,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkEvent {
    Hosting { ticket: String, relay_ready: bool },
    Connecting,
    Connected { mark: Mark },
    Disconnected,
    Failed(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum NetworkStatus {
    #[default]
    Idle,
    Hosting {
        ticket: String,
        relay_ready: bool,
    },
    Connecting,
    Connected {
        mark: Mark,
    },
    Failed(String),
}

impl From<NetworkEvent> for NetworkStatus {
    fn from(event: NetworkEvent) -> Self {
        match event {
            NetworkEvent::Hosting {
                ticket,
                relay_ready,
            } => Self::Hosting {
                ticket,
                relay_ready,
            },
            NetworkEvent::Connecting => Self::Connecting,
            NetworkEvent::Connected { mark } => Self::Connected { mark },
            NetworkEvent::Disconnected => Self::Idle,
            NetworkEvent::Failed(error) => Self::Failed(error),
        }
    }
}

pub fn encode_ticket(addr: EndpointAddr) -> String {
    EndpointTicket::new(addr).to_string()
}

pub fn decode_ticket(ticket: &str) -> Result<EndpointAddr, ParseError> {
    let ticket = ticket
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .parse::<EndpointTicket>()?;
    Ok(ticket.into())
}

pub async fn create_endpoint() -> Result<Endpoint, BindError> {
    Endpoint::builder(presets::N0)
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
}

pub async fn create_host() -> Result<(Endpoint, String), BindError> {
    let endpoint = create_endpoint().await?;

    let ticket = encode_ticket(endpoint.addr());

    Ok((endpoint, ticket))
}

pub async fn connect_to_host(
    endpoint: &Endpoint,
    addr: EndpointAddr,
) -> Result<Connection, ConnectError> {
    endpoint.connect(addr, ALPN).await
}

pub async fn accept_connection(endpoint: &Endpoint) -> Option<Result<Connection, ConnectingError>> {
    match endpoint.accept().await {
        Some(incoming) => Some(incoming.await),
        None => None,
    }
}

pub struct NetworkClient {
    command_tx: Option<tokio_mpsc::UnboundedSender<NetworkCommand>>,
    event_rx: mpsc::Receiver<NetworkEvent>,
    worker: Option<thread::JoinHandle<()>>,
}

impl NetworkClient {
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

    pub fn send(
        &self,
        command: NetworkCommand,
    ) -> Result<(), tokio_mpsc::error::SendError<NetworkCommand>> {
        self.command_tx
            .as_ref()
            .expect("network client is shutting down")
            .send(command)
    }

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
        tokio::select! {
            command = command_rx.recv() => {
                let Some(command) = command else { break };
                operation_id += 1;
                if let Some(task) = pending.take() {
                    task.abort();
                }
                if let Some(session) = session.take() {
                    close_session(session).await;
                }

                match command {
                    NetworkCommand::Host => {
                        let result_tx = result_tx.clone();
                        let event_tx = event_tx.clone();
                        let id = operation_id;
                        pending = Some(tokio::spawn(async move {
                            let result = host_session(event_tx).await;
                            let _ = result_tx.send(OperationResult { id, result });
                        }));
                    }
                    NetworkCommand::Join(ticket) => {
                        let result_tx = result_tx.clone();
                        let event_tx = event_tx.clone();
                        let id = operation_id;
                        pending = Some(tokio::spawn(async move {
                            let result = join_session(ticket, event_tx).await;
                            let _ = result_tx.send(OperationResult { id, result });
                        }));
                    }
                    NetworkCommand::Disconnect => {
                        let _ = event_tx.send(NetworkEvent::Disconnected);
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
                        session = Some(new_session);
                        let _ = event_tx.send(NetworkEvent::Connected { mark });
                    }
                    Err(error) => {
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

async fn close_session(session: NetworkSession) {
    session.connection.close(0u32.into(), b"disconnected");
    session.endpoint.close().await;
}

async fn host_session(event_tx: mpsc::Sender<NetworkEvent>) -> Result<NetworkSession, String> {
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

    let mark = perform_host_handshake(&connection).await?;

    Ok(NetworkSession {
        endpoint,
        connection,
        mark,
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
    event_tx: mpsc::Sender<NetworkEvent>,
) -> Result<NetworkSession, String> {
    let _ = event_tx.send(NetworkEvent::Connecting);
    let addr = decode_ticket(&ticket).map_err(|error| error.to_string())?;
    let endpoint = create_endpoint().await.map_err(|error| error.to_string())?;
    let connection = connect_to_host(&endpoint, addr)
        .await
        .map_err(|error| error.to_string())?;
    let mark = perform_join_handshake(&connection).await?;

    Ok(NetworkSession {
        endpoint,
        connection,
        mark,
    })
}

const MAX_HANDSHAKE_SIZE: usize = 1024;

async fn perform_host_handshake(connection: &Connection) -> Result<Mark, String> {
    let (mut send, mut receive) = connection
        .accept_bi()
        .await
        .map_err(|error| error.to_string())?;
    let bytes = receive
        .read_to_end(MAX_HANDSHAKE_SIZE)
        .await
        .map_err(|error| error.to_string())?;

    match protocol::decode(&bytes).map_err(|error| error.to_string())? {
        HandshakeMessage::Hello { protocol_version } if protocol_version == PROTOCOL_VERSION => {}
        HandshakeMessage::Hello { protocol_version } => {
            return Err(format!("unsupported protocol version: {protocol_version}"));
        }
        HandshakeMessage::Welcome { .. } => {
            return Err("expected hello handshake message".to_string());
        }
    }

    let response =
        protocol::encode(&HandshakeMessage::welcome(Mark::O)).map_err(|error| error.to_string())?;
    send.write_all(&response)
        .await
        .map_err(|error| error.to_string())?;
    send.finish().map_err(|error| error.to_string())?;

    Ok(Mark::X)
}

async fn perform_join_handshake(connection: &Connection) -> Result<Mark, String> {
    let (mut send, mut receive) = connection
        .open_bi()
        .await
        .map_err(|error| error.to_string())?;
    let hello = protocol::encode(&HandshakeMessage::hello()).map_err(|error| error.to_string())?;
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
        } if protocol_version == PROTOCOL_VERSION => Ok(mark),
        HandshakeMessage::Welcome {
            protocol_version, ..
        } => Err(format!("unsupported protocol version: {protocol_version}")),
        HandshakeMessage::Hello { .. } => Err("expected welcome handshake message".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        client.send(NetworkCommand::Host).unwrap();

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
            .send(NetworkCommand::Join("invalid ticket".to_string()))
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

        host.send(NetworkCommand::Host).unwrap();
        let NetworkEvent::Hosting { ticket, .. } = host
            .event_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap()
        else {
            panic!("expected hosting event");
        };

        joiner.send(NetworkCommand::Join(ticket)).unwrap();
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

    fn wait_for_connected_mark(client: &NetworkClient) -> Mark {
        loop {
            match client
                .event_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap()
            {
                NetworkEvent::Connected { mark } => return mark,
                NetworkEvent::Hosting { .. } => {}
                event => panic!("expected connected event, got {event:?}"),
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
