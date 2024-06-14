// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

use thiserror;

/// Dataset errors
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("Already connected")]
    Connected,
    #[error("Disconnected")]
    Disconnected,
    #[error("Expected row missing after write: {0}")]
    MissingRow(crate::ID),
    #[error("Websocket response: {0}")]
    WebsocketResponse(#[from] crate::WebsocketErrorResponse),
    #[error("Database: {0}")]
    Database(String),

}

/// Dataset results
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "sql")]
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::Database(e.to_string())
    }
}
