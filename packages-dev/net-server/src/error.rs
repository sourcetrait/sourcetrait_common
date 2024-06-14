use thiserror;
use crate::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Server(#[from] ServerError),
    #[error("{0}")]
    Network(#[from] crate::NetworkError),
}

/// Error has been logged via `log_error!()` prior to returning. Success may, optionally, be logged as well.
pub type LoggedResult<T> = Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Failed to send message '{msg_name}' to {who} :> {reason}")]
    Send{who: String, msg_name: &'static str, reason: String},

    #[error("Failed to receive message '{msg_name}' from {who} :> {reason}")]
    Receive{who: String, msg_name: &'static str, reason: String},

    #[error("Unexpected response received from {who} when expecting a {expected}.")]
    UnexpectedResponse{who: String, expected: String},

    #[error("Protocol mismatch with {who}. Expected: {expected}. Received: {received}.")]
    ProtocolMismatch{who: String, expected: String, received: String},

    #[error("Connection rejected from {who}.")]
    Rejected{who: String},

    #[error("Abrupt disconnection from {who}")]
    Disconnected{who: String},

    #[error("Stream IO error: {0}")]
    StreamIO(String),

    #[error("Stream disconnected")]
    StreamDisconnected
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File I/O error for file: {filepath} :> {cause}")]
    FileIO{ filepath: String, cause: String},

    #[error("Config File parsing error: {filepath} :> {cause}")]
    ConfigFileError{ filepath: String, cause: String},

    #[error("TLS error: {0}")]
    TLS(#[from] tokio_native_tls::native_tls::Error),
}

impl ServerError {
    pub fn fileio(error: std::io::Error, filepath: &std::path::Path) -> Self {
        Self::FileIO {
            filepath: filepath.to_str().unwrap().to_string(),
            cause: error.to_string(),
        }
    }

    pub fn config_file(error: impl serde::de::Error, filepath: &std::path::Path) -> Self {
        Self::FileIO {
            filepath: filepath.to_str().unwrap().to_string(),
            cause: error.to_string(),
        }
    }
}
