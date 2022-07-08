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
use http::response;
//use hyper::http::{Request, Response, StatusCode};
//use hyper::{server::conn::Http, service::service_fn, Body};
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

use http::{Response, StatusCode};
const NOT_FOUND: &'static str = r#"HTTP/1.0 404 Not Found
Content-Length: {}

Tunnel {} not found"#;

const BadRequest: &'static str = r#"HTTP/1.0 400 Bad Request
Content-Length: 12

Bad Request"#;

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
    let mut req_buf = [0; 1024];
    tcp_socket.read(&mut req_buf).await?;
    // tcp_socket.peek(&mut buf).await.unwrap();
    if let Err(err) = req.parse(&req_buf) {
        log::info!("Parse http request got wrong:{}", err);
        tcp_socket.write_all(BadRequest.as_bytes()).await?;
        // tcp_socket.shutdown().await?;
        return Ok(());
    }
    log::debug!("parsed http req:{:?}", req);
    let url = format!(
        "http://{}",
        std::str::from_utf8(
            req.headers
                .iter()
                .find(|&x| x.name == "Host")
                .unwrap()
                .value
        )
        .unwrap()
    );
    log::debug!("host:{}", url);
    if let Some(tunnel) = get_tunnel_cache(&url) {
        let c = tunnel.ctl.upgrade().unwrap();
        let proxy = c.get_proxy_conn().await;

        {
            Message::StartProxy(StartProxy {
                Url: tunnel.url.clone(),
                ClientAddr: tcp_socket.peer_addr().unwrap().clone().to_string(),
            })
            .send_message(&proxy)
            .await?;
        }
        {
            proxy.write_stream.lock().await.write_all(&req_buf).await?;
        }
        let (mut r, mut w) = tcp_socket.split();
        //let wo = *proxy.read_stream.lock().await;
        let client_to_server = async {
            io::copy(&mut r, &mut *proxy.write_stream.lock().await).await
            // wo.shutdown().await
        };

        let server_to_client = async {
            io::copy(&mut *proxy.read_stream.lock().await, &mut w).await
            // wi.shutdown().await
        };

        tokio::try_join!(client_to_server, server_to_client)?;
    } else {
        tcp_socket
            .write_all(format!("Tunnel:{} not found,please check again!", url).as_bytes())
            .await?;
    }

    Ok(())
}
async fn handle_tunnel_conn(
    socket: TcpStream,
    tlsacceptor: TlsAcceptor,
) -> Result<(), Box<dyn Error>> {
    log::info!("handle control/proxy connection");
    //let(r,w) = socket.split();
    let mut tls_socket = tlsacceptor.accept(socket).await?;
    //let mut conn = connection::Conn::new(tls_socket, None);
    match message::Message::from_stream(&mut tls_socket).await? {
        Message::AuthReq(authreq) => {
            // conn.conn_type = Some("ctrl".to_owned());
            control::handle_ctrl_conn(
                Arc::new(connection::Conn::new(tls_socket, Some("ctrl".to_owned()))),
                authreq,
            )
            .await?;
        }
        Message::RegProxy(reg_proxy) => {
            // conn.conn_type = Some("proxy".to_owned());
            log::info!("receive new  proxy connection  {:?}", reg_proxy);
            control::handle_proxy_conn(
                connection::Conn::new(tls_socket, Some("proxy".to_owned())),
                reg_proxy,
            )
            .await?;
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
