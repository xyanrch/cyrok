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

use cyrok::message;
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
    mut socket: TcpStream,
    tlsacceptor: TlsAcceptor,
) -> Result<(), Box<dyn Error>> {
    log::info!("handle control/proxy connection");
    //let(r,w) = socket.split();
    let mut tls_socket = tlsacceptor.accept(socket).await?;
    //let con = connection::Conn::new(tlsacceptor.accept(socket).await.unwrap());

    let len = tls_socket.read_u64_le().await?;
    log::info!("receive message len:{:?}", len);
    let mut buf = BytesMut::with_capacity(len.try_into().unwrap());
    tls_socket.read_buf(&mut buf).await?;
    log::info!("receive message {:?}", buf);
    let raw: message::Envelope = serde_json::from_slice(&mut buf)?;
    log::info!("receive parsed message {:?}", raw);
    let auth = message::Message::frome_envelop(raw)?;
    log::info!("receive auth message {:?}", auth);

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
