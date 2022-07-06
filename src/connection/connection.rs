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
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::error::Elapsed,
};
use tokio_rustls::server::TlsStream;
#[derive(Debug)]
pub struct Conn {
    pub tls_socket: Arc<Mutex<TlsStream<TcpStream>>>,
    pub conn_type: Option<String>,
}
impl Conn {
    pub fn new(socket: TlsStream<TcpStream>, conn_type: Option<String>) -> Conn {
        Conn {
            tls_socket: Arc::new(Mutex::new(socket)),
            conn_type,
        }
    }

    pub async fn send_message(&self, message: Message) -> Result<(), Box<dyn Error>> {
        let raw = serde_json::to_string(&message.to_envelop())?;
        let mut socket_lock_guard = self.tls_socket.lock().await;
        socket_lock_guard
            .write_i64_le(raw.len().try_into().unwrap())
            .await?;
        socket_lock_guard.write_all(raw.as_bytes()).await?;

        Ok(())
    }
}
