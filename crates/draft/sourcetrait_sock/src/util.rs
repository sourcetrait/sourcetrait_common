use crate::*;

pub async fn resolve_address(hostname: &str, port: u16) -> SockResult<SocketAddr> {
    let host = format!("{}:{}", hostname, port);
    tokio::net::lookup_host(host).await
        .map_err(|source| SockError::Tls{ e: TlsSockError::Io { source }})?
        .next()
        .ok_or_else(|| SockError::Tls{ e: TlsSockError::Io {
            source: io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "Unable to resolve hostname",
            )
        }})
}