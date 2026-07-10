use iroh::{
    Endpoint, EndpointAddr,
    endpoint::{BindError, ConnectError, ConnectingError, Connection, presets},
};
use iroh_tickets::{ParseError, endpoint::EndpointTicket};

pub const ALPN: &[u8] = b"/tic-tac-foe/1";

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

pub fn encode_ticket(addr: EndpointAddr) -> String {
    EndpointTicket::new(addr).to_string()
}

pub fn decode_ticket(ticket: &str) -> Result<EndpointAddr, ParseError> {
    let ticket = ticket.trim().parse::<EndpointTicket>()?;
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

    endpoint.online().await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_round_trip() {
        let key = iroh::SecretKey::generate();
        let addr = EndpointAddr::new(key.public());

        let ticket = encode_ticket(addr.clone());
        let decoded = decode_ticket(&ticket).unwrap();

        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_decode_ticket_trims_input() {
        let key = iroh::SecretKey::generate();
        let addr = EndpointAddr::new(key.public());
        let ticket = format!("  {}\n", encode_ticket(addr.clone()));

        let decoded = decode_ticket(&ticket).unwrap();

        assert_eq!(decoded, addr);
    }
}
