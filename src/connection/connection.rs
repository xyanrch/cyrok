use crate::{
    message::{
        self,
        auth::AuthResp,
        heatbeat::{Ping, Pong},
        proxy::{RegProxy, ReqProxy},
        tunnel::ReqTunnel,
        Message,
    },
    VERSON_MARJOR, VERSON_MINI,
};
use std::error::Error;
use std::{sync::Arc, sync::Weak};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::error::Elapsed,
};
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
};
use tokio_rustls::server::TlsStream;
#[derive(Debug)]
pub struct Conn<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    // pub tls_socket: Arc<Mutex<T>>,
    pub read_stream: Arc<Mutex<ReadHalf<T>>>,
    pub write_stream: Arc<Mutex<WriteHalf<T>>>,
    pub conn_type: Option<String>,
}
impl<T> Conn<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(socket: T, conn_type: Option<String>) -> Conn<T> {
        let (r, w) = tokio::io::split(socket);
        Conn {
            // tls_socket: Arc::new(Mutex::new(socket)),
            read_stream: Arc::new(Mutex::new(r)),
            write_stream: Arc::new(Mutex::new(w)),
            conn_type,
        }
    }

    pub async fn send_message(&self, message: Message) -> Result<(), Box<dyn Error>> {
        let raw = serde_json::to_string(&message.to_envelop())?;
        let mut socket_lock_guard = self.write_stream.lock().await;
        socket_lock_guard
            .write_i64_le(raw.len().try_into().unwrap())
            .await?;
        socket_lock_guard.write_all(raw.as_bytes()).await?;

        Ok(())
    }
}
