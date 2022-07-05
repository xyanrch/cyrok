use crate::connection;
use crate::control;
use crate::registery::get_tunnel_cache;
use bytes::{Bytes, BytesMut};
use cyrok::message::proxy::RegProxy;
use cyrok::message::proxy::ReqProxy;
use cyrok::message::proxy::StartProxy;
use cyrok::message::{self, proxy, Message};
use futures::future::ok;
use futures::SinkExt;
use serde::Deserializer;
use serde_json::Map;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{tcp, TcpListener, TcpStream};
use tokio::time::sleep;
//use tokio::prelude::*;
//use tokio::io::AsyncReadExt;
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, Decoder, Encoder, Framed, LinesCodec};
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
                    Type::Data => {
                        handle_date_conn(tcp_socket)
                            .await
                            .expect("Transport endpoint is not connected");
                    }
                }
            });
        }
    }
}
async fn handle_date_conn(tcp_socket: TcpStream) -> Result<(), Box<dyn Error>> {
    //log::info!("receive http connection")
    let mut buf = BytesMut::with_capacity(1024);
    let mut tcp = tcp_socket;
    tcp.read_buf(&mut buf).await?;
    log::info!("receive public http: {:?}", buf);
    let tunnel = get_tunnel_cache("http://test.ngrok.me:7777").unwrap();
    let l = tunnel.lock().await;
    let c = l.ctl.upgrade().unwrap();
    drop(l);
    let mut guard_c = c.lock().await;
    if guard_c.proxys.is_empty() {
        let message = Message::ReqProxy(ReqProxy {});
        // let raw = serde_json::to_string(&message.to_envelop())?;
        let mut conn = guard_c.conn.lock().await;

        conn.send_message(message).await?;

        //  return Ok(());
    }

    drop(guard_c);
    sleep(Duration::from_millis(10)).await;
    let mut guard_c = c.lock().await;
    let mut proxy = guard_c.proxys.pop().unwrap();

    {
        let message = Message::StartProxy(StartProxy {
            Url: "http://test.ngrok.me:7777".to_owned(),
            ClientAddr: tcp.peer_addr().unwrap().clone().to_string(),
        });
        let raw = serde_json::to_string(&message.to_envelop())?;

        proxy.1.write_i64_le(raw.len().try_into().unwrap()).await?;
        proxy.1.write_all(raw.as_bytes()).await?;
    }

    proxy.1.write_all(&buf).await?;
    let mut s = proxy.1;
    let (mut ri, mut wi) = tcp.split();
    let (mut ro, mut wo) = io::split(s); //s.split();
                                         //s.
                                         //let (mut pp, kk) = s.into_inner();
                                         // kk.
                                         //  let (mut ro, mut wo) = pp.split();

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        // Ok(())
        wi.shutdown().await
    };
    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        // Ok(())
        wo.shutdown().await
    };
    tokio::try_join!(client_to_server, server_to_client)?;

    /*tokio::spawn(async move {
            let mut s = proxy.1;
            //let (mut ri, mut wi) = tcp.split();
           // let (mut ro, mut wo)= s.split();
             //let (mut pp, _) = s.into_inner();
            // let (mut ro, mut wo) = pp.split();
            let mut bufk = BytesMut::with_capacity(10240);
            loop {
                if let Ok(n) = s.read_buf(&mut bufk).await {
                    if n > 0 {
                        log::info!("read internal contet {:?}", bufk);
                        tcp.write_all(&mut bufk).await.unwrap();
                    }
                }
               else
               {
                    break;
                }

                log::info!("MMMMMMMMM");

                if let Ok(n) = tcp.read_buf(&mut bufk).await {
                    if n > 0 {
                        log::info!("read public contet {:?}", bufk);
                        s.write_all(&mut bufk).await.unwrap();
                    }
                    else
                    {
                       // log::info!("eptm");
                    }
                }
                else
                {
                    break;
                }
            }
            /*   let server_to_client = async {
                io::copy(&mut ro, &mut wi).await?;
                wi.shutdown().await
            };
            let client_to_server = async {
                io::copy(&mut ri, &mut wo).await?;
                wo.shutdown().await
            };*/

            log::info!("END???????????");
        });
    */
    Ok(())
}
async fn handle_tunnel_conn(
    socket: TcpStream,
    tlsacceptor: TlsAcceptor,
) -> Result<(), Box<dyn Error>> {
    log::info!("handle control/proxy connection");
    //let(r,w) = socket.split();
    let mut tls_socket = tlsacceptor.accept(socket).await?;

    match message::Message::from_conn_2(&mut tls_socket).await? {
        Message::AuthReq(authreq) => {
            let mut conn = connection::Conn::new(tls_socket, None);
            conn.conn_type = Some("ctrl".to_owned());
            control::handle_ctrl_conn(conn, authreq).await?;
        }
        Message::RegProxy(reg_proxy) => {
            // conn.conn_type = Some("proxy".to_owned());
            log::info!("receive new  proxy connection  {:?}", reg_proxy);
            control::handle_proxy_conn(tls_socket, reg_proxy).await?;
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
