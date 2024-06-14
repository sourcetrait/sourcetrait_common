
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_tungstenite::{self, MaybeTlsStream, WebSocketStream};
use crate::*;

pub type ConnectionResult<W> = Result<Connection<W>, NetworkError>;

pub struct Connection<P: Protocol> {
    who: Who<P::WhoWhat>,
    stream: Stream,
    msg_num: u8,
}

impl<P> ConnectionTrait<P> for Connection<P>
where
    P: Protocol,
{
    type StreamType = Stream;

    fn new(who: Who<P::WhoWhat>, stream: Self::StreamType) -> Self {
        Self {
            who,
            stream,
            msg_num: 0,
        }
    }

    fn who(&self) -> &Who<P::WhoWhat> {
        &self.who
    }

    fn stream(&mut self) -> &mut Self::StreamType {
        &mut self.stream
    }
}

impl<P> Connection<P>
where
    P: Protocol
{
    pub fn msg_num(&self) -> u8 {
        self.msg_num
    }

    pub fn next_msg_num(&mut self) -> u8 {
        self.msg_num = self.msg_num.wrapping_add(1);
        self.msg_num
    }
}

pub fn connection_send_error<P:Protocol>(who: &Who<P::WhoWhat>, error: tokio_tungstenite::tungstenite::error::Error) -> Result<(),()> {
    log_error!("Connection with {who} failed :> Error while sending data :> {}", error.to_string());
    Err(())
}

pub async fn connection_close<P:Protocol>(
    who: &Who<P::WhoWhat>,
    mut websocket_stream: WebSocketStream<TlsStream<TcpStream>>
) -> Result<(),()> {
    log!("Closed connection with {who}.");
    let _ = websocket_stream.close(None).await;
    Ok(())
}


pub async fn host_connection_close<P:Protocol>(
    who: &Who<P::WhoWhat>,
    mut websocket_stream: WebSocketStream<MaybeTlsStream<TcpStream>>
) -> Result<(),()> {
    log!("Closed connection with {who}.");
    let _ = websocket_stream.close(None).await;
    Ok(())
}
