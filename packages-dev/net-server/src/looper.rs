
// todo: Refactor out into a separate module
// Begin Configuration

#[derive(Clone, Copy, Debug)]
pub enum ConnectionDirection {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectionProtocol {
    SecureWebsocket,
}

pub enum ConnectionProtocolDetails {
    IncomingSecureWebsocket (IncomingSecureWebsocket),
    OutgoingSecureWebsocket (OutgoingSecureWebsocket),
}

pub struct IncomingSecureWebsocket {
}

pub struct OutgoingSecureWebsocket {

}

pub struct ConnectionGroup {
    pub name: String,
    pub protocol_details: ConnectionProtocolDetails,
}

impl ConnectionGroup {
    pub fn direction(&self) -> ConnectionDirection {
        match self.protocol_details {
            ConnectionProtocolDetails::IncomingSecureWebsocket(_) => ConnectionDirection::Incoming,
            ConnectionProtocolDetails::OutgoingSecureWebsocket(_) => ConnectionDirection::Outgoing,
        }
    }

    pub fn protocol(&self) -> ConnectionProtocol {
        match self.protocol_details {
            ConnectionProtocolDetails::IncomingSecureWebsocket(_) => ConnectionProtocol::SecureWebsocket,
            ConnectionProtocolDetails::OutgoingSecureWebsocket(_) => ConnectionProtocol::SecureWebsocket,
        }
    }
}

// End Configuration

pub trait ServerLoop {
    fn run() -> impl std::future::Future<Output = std::process::ExitCode>;
}