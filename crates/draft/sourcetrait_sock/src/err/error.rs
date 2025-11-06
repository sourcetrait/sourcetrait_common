use crate::*;

pub type SockResult<T> = Result<T, SockError>;

#[derive(Debug, snafu::Snafu)]
pub enum SockError {
    Config { cause: String },
    Tls { e: TlsSockError },
}

#[derive(Debug, snafu::Snafu)]
pub enum TlsSockError {
    Base { source: r::tls::Error },
    Pem { source: r::tls::pem::Error },
    Io { source: io::Error },
}

impl From<r::tls::Error> for SockError {
    fn from(source: r::tls::Error) -> Self {
        SockError::Tls { e: TlsSockError::Base { source } }
    }
}

impl From<r::tls::pem::Error> for SockError {
    fn from(source: r::tls::pem::Error) -> Self {
        SockError::Tls{ e: TlsSockError::Pem { source } }
    }
}
