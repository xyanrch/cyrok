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

use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;
use cyrok::message;
//pub mod cmd;
#[derive(Debug, Clone)]
enum Type {
    Control,
    Data,
}
struct ListenerWrapper {
    tls_acceptor:TlsAcceptor,
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
                        if let Err(err) = handle_tunnel_conn(tcp_socket,tlsaccetor).await {
                            log::error!("connection error:{}", err);
                        }
                    }
                    Type::Data => {}
                }
            });
        }
    }
}
/*
struct Handler {
    socket: TcpStream,
    handle_type: Type,
}

impl Handler {
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {

        match self.handle_type {
            Type::Control => {
                log::info!("handle control/proxy connection");
                let mut lines =Framed::new(self.socket, LinesCodec::new());

                // Send a prompt to the client to enter their username.

                // Read the first line from the `LineCodec` stream to get the username.
                   match lines.next().await {
                    Some(Ok(line)) =>
                    {
                        log::info!("receive message:{}",line);
                    }
                    // We didn't get a line so we return early here.
                    _ => {
                        log::error!(
                            "Failed to get message from ctrol connection"
                        );
                        return Ok(());
                    }
                };
            }
            Type::Data => {
                log::info!("handle data connection");
            }
        }
        Ok(())
    }
}*/

async fn handle_tunnel_conn(mut socket: TcpStream, tlsacceptor:TlsAcceptor) -> Result<(), Box<dyn Error>> {
    log::info!("handle control/proxy connection");
    //let(r,w) = socket.split();
    let mut tls_socket = tlsacceptor.accept(socket).await?;
    //let con = connection::Conn::new(tlsacceptor.accept(socket).await.unwrap());

    let len = tls_socket.read_u64_le().await?;
    log::info!("receive message len:{:?}", len);
    let mut buf =BytesMut::with_capacity(len.try_into().unwrap());
    tls_socket.read_buf(&mut buf).await?;
    log::info!("receive message {:?}", buf);
    let raw:message::Envelope = serde_json::from_slice(&mut buf)?;
    log::info!("receive parsed message {:?}", raw);
    let auth = message::Message::frome_envelop(raw)?;
    log::info!("receive auth message {:?}", auth);

    // let mut reader = BufReader::new(&mut socket);
    //let ress1 = reader.read_buf(&mut buf).await?;
    // let len = reader.read_u64_le().await?;
    // log::info!("receive message len:{}", len);
    // let mut buf = vec![0; 8];
    //let mut bufstr=BytesMut::with_capacity(len.try_into().unwrap());
    // let re = reader.read_buf(&mut buf).await?;
    // log::info!("receive buffer:{:?}", buf);

    // obtaining string
    //   match String::from_utf8(buf) {
    // Ok(val) => {
    //    println!("[+] So the parsed value is {}",val);
    //let temp = val.as_str();
    //  let parsed:serde_json::Value = serde_json::from_str(&val).unwrap();

    //  println!("{:?}",parsed);
    // socket.write_all(b"So yeah thanks for sending this\r\n").await.unwrap();
    //continue;
    //  }
    // Err(err) => {
    //  println!("ERROR Could not convert to string {:?}",err);
    //   //  continue;
    //  }
    // }
    //let mut de :Msg = serde_json::from_slice(&bufstr).unwrap();
    //

    //log::info!("receive message:{:?}", de);

    //
    //buf
    // bufstr.to_ascii_lowercase();
    //let s:String = String::from_
    // let mut de :Msg = serde_json::from_slice(&bufstr).unwrap();

    //let re = reader.read_buf(&mut bufstr).await?;
    // reader.read_buf(&mut buf).await?;
    //  let result =socket.read_to_string(&mut buf).await;
    //let s:String =  String::from_utf8(bufstr.to_vec()).unwrap();//bufstr.into();

    // log::info!("receive message:{:?}", String::from_utf8_lossy(&bufstr));
    //

    /*let mut lines =Framed::new(socket, LinesCodec::new());

                    // Send a prompt to the client to enter their username.

                    // Read the first line from the `LineCodec` stream to get the username.
                       match lines.next().await {
                        Some(Ok(line)) =>
                        {
                            log::info!("receive message:{}",line);
                        }
                        // We didn't get a line so we return early here.
                        _ => {
                            log::error!(
                                "Failed to get message from ctrol connection"
                            );
                            return Ok(());
                        }
                    };
    */
    Ok(())
}
pub async fn run(tls_acceptor:TlsAcceptor,listener: TcpListener, public_listener: TcpListener, shutdown: impl Future) {
    //  let (notify_shutdown, _) = broadcast::channel(1);
    //  let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let mut ctrl_service = ListenerWrapper {
        tls_acceptor:tls_acceptor.clone(),
        listener,
        listener_type: Type::Control,
    };
    let mut data_service = ListenerWrapper {
        tls_acceptor:tls_acceptor.clone(),
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
