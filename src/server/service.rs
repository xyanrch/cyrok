use bytes::{Bytes, BytesMut};
use futures::SinkExt;
use serde::Deserializer;
use serde_json::Map;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, Decoder, Encoder, Framed, LinesCodec};

use crate::connection;
use crate::control;
use cyrok::message::{self, Message};
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;
//pub mod cmd;
#[derive(Debug, Clone)]
enum Type {
    Control,
    Data,
}
struct ListenerWrapper {
    tls_acceptor: TlsAcceptor,
    listener: TcpListener,
    listener_type: Type,
}
impl ListenerWrapper {
    async fn run(&mut self) -> Result<(), Box<dyn Error + '_>> {
        loop {
            let (tcp_socket, _) = self.listener.accept().await?;
            let ltype = self.listener_type.clone();
            let tlsaccetor = self.tls_acceptor.clone();
            tokio::spawn(async move {
                match ltype {
                    Type::Control => {
                        if let Err(err) = handle_tunnel_conn(tcp_socket, tlsaccetor).await {
                            log::error!("connection error:{}", err);
                        }
                    }
                    Type::Data => {}
                }
            });
        }
    }
}

async fn handle_tunnel_conn(
    socket: TcpStream,
    tlsacceptor: TlsAcceptor,
) -> Result<(), Box<dyn Error>> {
    log::info!("handle control/proxy connection");
    //let(r,w) = socket.split();
    let tls_socket = tlsacceptor.accept(socket).await?;
    let mut conn = connection::Conn::new(tls_socket, None);
    match message::Message::from_conn(&mut conn.tls_socket).await? {
        Message::AuthReq(authreq) => {
            conn.conn_type = Some("ctrl".to_owned());
            control::handle_ctrl_conn(conn, authreq).await?;
        }
        Message::RegProxy(reg_proxy) => {
            conn.conn_type = Some("proxy".to_owned());
            log::info!("receive new  proxy connection  {:?}", reg_proxy);
            control::handle_proxy_conn(conn, reg_proxy).await?;

        }
        Message::Unknown(_) => {}
        _ => {}
    }

    Ok(())
}
pub async fn run(
    tls_acceptor: TlsAcceptor,
    listener: TcpListener,
    public_listener: TcpListener,
    shutdown: impl Future,
) {
    //  let (notify_shutdown, _) = broadcast::channel(1);
    //  let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let mut ctrl_service = ListenerWrapper {
        tls_acceptor: tls_acceptor.clone(),
        listener,
        listener_type: Type::Control,
    };
    let mut data_service = ListenerWrapper {
        tls_acceptor: tls_acceptor.clone(),
        listener: public_listener,
        listener_type: Type::Data,
    };
    // let f = ctrl_service.run();
    tokio::select! {
        res1=ctrl_service.run()=>
        {
            if let Err(err) = res1 {
                log::error!("failed to accept control:{}",err);
            }
        }
        res2 = data_service.run()=>
        {
            if let Err(err) = res2 {
                log::error!("failed to accept data connection:{}",err);
            }

       }
        _ = shutdown => {
            // The shutdown signal has been received.
            log::info!("shutting down");
        }
    }
}
