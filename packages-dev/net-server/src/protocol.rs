use bincode;
use crate::*;

// Default backend ports
pub const ELSE_UNIVERSE_WORLD_PORT: u16 = 3151;
pub const ELSE_UNIVERSE_ZONE_PORT: u16  = 3152;
pub const ELSE_WORLD_ZONE_PORT: u16     = 3153;
pub const ELSE_AUTH_UNIVERSE_PORT: u16  = 3150;

// Default frontend ports
pub const ELSE_ZONE_CLIENT_PORT: u16    = 8443;

pub const ELSE_LOCALHOST_ZONE_ADDR: &'static str = "127.0.0.1:8443";
pub const ELSE_LOCALHOST_ZONE_URL: &'static str = "wss://127.0.0.1:8443";

pub const WEBSOCKET_PAYLOAD_ERROR: u16 = 1007;

pub const MAX_RECONNECT_WAIT: u64 = 120;

pub type SendResult = Result<(), NetworkError>;
pub type ReceiveResult<M> = Result<M, NetworkError>;
pub type StreamResult<P: Protocol> = Result<Who<P::WhoWhat>, NetworkError>;
/// Void Ok and Err
pub type TaskResult = Result<(),()>;

pub type ConnectionID = u32;

pub trait WhoWhat: Copy + Clone + PartialEq + Eq + std::fmt::Display + std::fmt::Debug + Send + Sync + 'static {}

#[derive(Debug, Clone, PartialEq)]
pub struct Who<W>{
    what: W,
    connection_id: ConnectionID,
    name: String,
}

impl<W> Who<W>
where
    W: WhoWhat
{
    pub fn connection_id(&self) -> ConnectionID {
        self.connection_id
    }

    pub fn what(&self) -> W {
        self.what
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<W> std::fmt::Display for Who<W>
where
    W: WhoWhat
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{who}, #{id} ({addr})", who = self.what(), id = self.connection_id(), addr = self.name())
    }
}

#[allow(async_fn_in_trait)]
pub trait StreamTrait
{
    /// Will throw only NetworkError variants: StreamIO
    async fn send(&mut self, bytes: Vec<u8>) -> SendResult;
    /// Will throw only NetworkError variants: StreamIO and StreamDisconnected
    async fn receive(&mut self) -> ReceiveResult<Vec<u8>>;
    async fn close_invalid(&mut self, reason: &str);
    async fn halt(&mut self);
}

#[allow(async_fn_in_trait)]
pub trait ConnectionTrait<P>
where
    P: Protocol,
{
    type StreamType: StreamTrait;

    fn new(who: Who<P::WhoWhat>, stream: Self::StreamType) -> Self;

    fn who(&self) -> &Who<P::WhoWhat>;
    fn stream(&mut self) -> &mut Self::StreamType;

    async fn send<M: Messaging>(&mut self, serializable: M) -> SendResult {
        let config = bincode::config::standard();
        let bytes = bincode::serde::encode_to_vec(&serializable, config).unwrap();
        let result = self.stream().send(bytes).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                self.halt().await;
                Err(NetworkError::Send{who: self.who().to_string(), msg_name: serializable.message_name(), reason: e.to_string()})
            }
        }
    }

    async fn receive<M: crate::Messaging>(&mut self) -> ReceiveResult<M> {
        match self.stream().receive().await {
            Ok(bytes) => {
                let msg: M = match bincode::serde::decode_from_slice(&bytes, bincode::config::standard()) {
                    Ok(m) => m.0,
                    Err(_) => return Err(self.error_payload(M::message_type_name()).await)
                };

                Ok(msg)
            },
            Err(NetworkError::StreamIO(e)) => {
                Err(NetworkError::Receive{who: self.who().to_string(), msg_name: M::message_type_name(), reason: e.to_string()})
            },
            Err(NetworkError::StreamDisconnected) => {
                Err(NetworkError::Disconnected{who: self.who().to_string()})
            },
            Err(_) => unreachable!("Unexpected NetworkError variant from StreamTrait::send()")
        }
    }

    async fn halt(&mut self) {
        self.stream().halt().await;
    }

    async fn error_protocol(&mut self, expected: impl Protocol, received: ProtocolHeader) -> NetworkError {
        let error = NetworkError::ProtocolMismatch {
            who: self.who().to_string(),
            expected: format!("{} v{}", expected, P::VERSION),
            received: format!("{} v{}", received.name, received.version) };
        self.stream().close_invalid(&error.to_string()).await;
        error
    }

    async fn error_payload(&mut self, expected: &str) -> NetworkError {
        let error = NetworkError::UnexpectedResponse {who: self.who().to_string(), expected: expected.to_string() };
        self.stream().close_invalid(&error.to_string()).await;
        error
    }
}

/// Both ends send their protocol headers, the connecting end first.
/// If the actual and expected protocols are incompatible, returns an error pending disconnection.
/// TODO: BROKEN. Are we even using this?
pub async fn negotiate_protocol<P: Protocol>(
    conn: &mut impl ConnectionTrait<P>,
    our_identity: ProtocolIdentity,
    our_protocol: P,
    their_expected_protocol: P
) -> Result<(), NetworkError>
{
    let our_protocol_header = ProtocolHeader::current::<P>(our_identity);
    let their_protocol_header: ProtocolHeader;

    if our_identity == ProtocolIdentity::Host {
        their_protocol_header = conn.receive().await?;
        conn.send(our_protocol_header.clone()).await?;
    } else {
        conn.send(our_protocol_header.clone()).await?;
        their_protocol_header = conn.receive().await?;
    }

    if our_protocol_header.compatible(&their_protocol_header) {
        Ok(())
    } else {
        Err(conn.error_protocol(our_protocol, their_protocol_header).await)
    }
}
