use crate::control;
use crate::registery::get_tunnel_cache;
use bytes::{Bytes, BytesMut};
use cyrok::connection;
use cyrok::message::proxy::RegProxy;
use cyrok::message::proxy::ReqProxy;
use cyrok::message::proxy::StartProxy;
use cyrok::message::{self, proxy, Message};
use futures::future::ok;
use futures::SinkExt;
use hyper::http::{Request, Response, StatusCode};
use hyper::{server::conn::Http, service::service_fn, Body};
use serde::Deserializer;
use serde_json::Map;
use std::convert::Infallible;
use std::error::Error;
use std::future::Future;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{tcp, TcpListener, TcpStream};
use tokio::time::error::Elapsed;
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
            let ( tcp_socket, _) = self.listener.accept().await?;
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
                        handle_http_conn(tcp_socket)
                            .await
                            .expect("Transport endpoint is not connected");
                    }
                }
            });
        }
    }
}

async fn handle_http_conn(mut tcp_socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    //
    //log::info!("receive http connection")
    let mut buf = [0;1024];
    //tcp.read_buf(&mut buf).await?;
    let n = tcp_socket.peek(&mut buf).await;
    log::info!("receive public http: {:?},{:?}", buf, n);

    if buf.is_empty() {
        tcp_socket.shutdown().await?;
        return Ok(());
    }
    req.parse(&buf).unwrap();
    log::info!("parsed http req:{:?}", req);
    let t = req
        .headers
        .iter()
        .find(|&x| x.name == "Host")
        .unwrap()
        .value;
    let url = format!("http://{}", std::str::from_utf8(t).unwrap());
    //let host_name = std::str::from_utf8(t).unwrap();
    log::info!("host:{}", url);
    let tunnel = get_tunnel_cache(&url).unwrap();
    let l = tunnel.lock().await;
    let c = l.ctl.upgrade().unwrap();
    drop(l);
    let mut proxy = c.get_proxy_conn().await;

    {
        let message = Message::StartProxy(StartProxy {
            Url: "http://test.ngrok.me:7777".to_owned(),
            ClientAddr: tcp_socket.peer_addr().unwrap().clone().to_string(),
        });
        let raw = serde_json::to_string(&message.to_envelop())?;

        proxy.write_i64_le(raw.len().try_into().unwrap()).await?;
        proxy.write_all(raw.as_bytes()).await?;
    }

    // proxy.write_all(&buf).await?;

    // Proxying data
    match tokio::io::copy_bidirectional(&mut proxy, &mut tcp_socket).await {
        Ok((from_client, from_server)) => {
            log::info!(
                "Copy data from_clinet：{}， from_server:{}",
                from_client,
                from_server
            );
            // proxy.shutdown().await?;
            // tcp.shutdown().await?;
        }
        Err(err) => {
            log::info!("the err is {}", err);
            // proxy.shutdown().await?;
            // tcp.shutdown().await?;
        }
    }

    // let (mut ri, mut wi) = tcp.split();

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
