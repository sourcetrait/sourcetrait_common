
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_tungstenite::{self, tungstenite::{protocol::{frame::coding::CloseCode, CloseFrame}, Message, Utf8Bytes}, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use std::{borrow::Cow, io::Bytes};
use bytes;
use crate::*;


pub enum Stream {
    Outgoing(WebSocketStream<MaybeTlsStream<TcpStream>>),
    Incoming(WebSocketStream<TlsStream<TcpStream>>)
}

impl StreamTrait for Stream
{
    async fn send(&mut self, data: Vec<u8>) -> SendResult {
        let msg = Message::Binary(bytes::Bytes::from(data));
        let result = match self {
            Self::Outgoing(s) => s.send(msg).await,
            Self::Incoming(s) => s.send(msg).await,
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                self.halt().await;
                Err(NetworkError::StreamIO(e.to_string()))
            }
        }
    }

    async fn receive(&mut self) -> ReceiveResult<Vec<u8>> {
        let result = match self {
            Stream::Outgoing(s) => s.next().await,
            Stream::Incoming(s) => s.next().await,
        };

        match result {
            Some(Ok(websocket_msg)) => {
                match websocket_msg {
                    Message::Binary(bytes) => Ok(bytes.into()),
                    Message::Close(_close_frame) => {
                        self.halt().await;
                        Err(NetworkError::StreamDisconnected)
                    },
                    _ => {
                        self.close_invalid("Binary").await;
                        Err(NetworkError::StreamIO("Unexpected websocket message type".to_string()))
                    },
                }
            },
            Some(Err(e)) => {
                self.halt().await;
                Err(NetworkError::StreamIO(e.to_string()))
            }
            None => {
                self.halt().await;
                Err(NetworkError::StreamDisconnected)
            }
        }
    }

    async fn close_invalid(&mut self, reason: &str) {
        match self {
            Stream::Outgoing(s) => {
                let _ = s.close(Some(CloseFrame {
                    code: CloseCode::Invalid,
                    reason: Utf8Bytes::from(reason)
                })).await;
            },
            Stream::Incoming(s) => {
                let _ = s.close(Some(CloseFrame {
                    code: CloseCode::Invalid,
                    reason: Utf8Bytes::from(reason)
                })).await;
            },
        }
    }

    async fn halt(&mut self) {
        let _error = match self {
            Stream::Outgoing(s) => s.close(None).await,
            Stream::Incoming(s) => s.close(None).await,
        };
    }
}
