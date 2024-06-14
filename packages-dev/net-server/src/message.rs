use std::fmt::Display;
use crate::*;

/// A const named MSGTYPENAME must be defined to use this.
macro_rules! msgname {
    ($name:literal) => {
        ::const_format::concatcp!(MSGTYPENAME, "::", $name)
    };
}

pub type MessageID = u16;
pub type ErrorCode = u8;

pub trait Messaging: Sized + serde::Serialize + serde::de::DeserializeOwned {
    const MESSAGE_TYPE_NAME: &'static str;

    fn message_type_name() -> &'static str {
        Self::MESSAGE_TYPE_NAME
    }

    fn message_name(&self) -> &'static str;
}

pub enum ErrorCodes {
    IllegalWebsocketFrame = 0x01
}

pub trait Protocol: Sized + serde::Serialize + serde::de::DeserializeOwned + PartialEq + Eq + Clone + Copy + Display {
    const NAME: &'static str;
    const VERSION: u16;
    type WhoWhat: WhoWhat;
    type HostMessaging: Messaging;
    type ClientMessaging: Messaging;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum ProtocolIdentity {
    Host,
    Client
}

#[derive(serde::Serialize, serde::Deserialize, Debug, strum::AsRefStr)]
pub enum HostSessionMessage {
    Connected,
    ConnectRejected,
    Disconnect
}

#[derive(serde::Serialize, serde::Deserialize, Debug, strum::AsRefStr)]
pub enum ClientSessionMessage {
    Connect,
    Disconnect
}

impl Messaging for ClientSessionMessage {
    const MESSAGE_TYPE_NAME: &'static str = "ClientSessionMessage";

    fn message_name(&self) -> &'static str {
        const MSGTYPENAME: &'static str = ClientSessionMessage::MESSAGE_TYPE_NAME;
        match self {
            ClientSessionMessage::Connect => msgname!("Connect"),
            ClientSessionMessage::Disconnect => msgname!("Disconnect"),
        }
    }
}

impl Messaging for HostSessionMessage {
    const MESSAGE_TYPE_NAME: &'static str = "HostSessionMessage";

    fn message_name(&self) -> &'static str {
        const MSGTYPENAME: &'static str = HostSessionMessage::MESSAGE_TYPE_NAME;
        match self {
            HostSessionMessage::Connected => msgname!("Connect"),
            HostSessionMessage::ConnectRejected => msgname!("ConnectRejected"),
            HostSessionMessage::Disconnect => msgname!("Disconnect"),
        }
    }
}


#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProtocolHeader {
    pub name: String,
    pub version: u16,
    pub identity: ProtocolIdentity,
}

impl ProtocolHeader {
    pub fn current<P: Protocol>(identity: ProtocolIdentity) -> Self {
        Self {
            name: P::NAME.to_string(),
            version: P::VERSION,
            identity,
        }
    }

    /// Checks this library's version and the expected protocol
    pub fn compatible(&self, expected: &ProtocolHeader) -> bool {
        self.version == expected.version && self.identity != expected.identity && self.name == expected.name
    }
}

impl Messaging for ProtocolHeader {
    const MESSAGE_TYPE_NAME: &'static str = "ProtocolHeader";

    fn message_name(&self) -> &'static str {
        Self::MESSAGE_TYPE_NAME
    }
}

impl Display for ProtocolHeader
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{}", self.name, self.version)
    }
}


/*
this is created by the end user
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, Debug, strum::Display)]
pub enum Protocol {
    Unsupported,
    AuthToUniverse,
    ClientToZone,
    ZoneToWorld,
    ZoneToUniverse,
    WorldToUniverse,
    UniverseToWorld,
    UniverseToZone,
    UniverseToAuth,
    WorldToZone,
    ZoneToClient,
}*/
