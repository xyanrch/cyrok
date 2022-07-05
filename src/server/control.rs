use crate::connection::Conn;
use crate::registery;
use crate::tunnel::Tunnel;
use bytes::BytesMut;
use cyrok::{
    message::{
        self,
        auth::AuthResp,
        heatbeat::{Ping, Pong},
        proxy::{RegProxy, ReqProxy},
        tunnel::NewTunnel,
        tunnel::ReqTunnel,
        Message,
    },
    VERSON_MARJOR, VERSON_MINI,
};
use log::info;
use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc, RwLock},
    time::Duration,
};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{tcp, TcpListener, TcpStream};
use tokio::{net::TcpSocket, time::error::Elapsed};
use tokio::{sync::Mutex, time::sleep};
use tokio_rustls::server::TlsStream;
#[derive(Debug)]
pub struct Control {
    pub conn: Arc<Mutex<Conn>>,
    pub id: String, //tunnels:Vec<Tunnel>,
    pub proxys: Arc<Mutex<Vec<TlsStream<TcpStream>>>>,
}

impl Control {
    async fn register_tunnel(self: &Arc<Self>, req_tunnel: ReqTunnel) {
        let t = Tunnel::new(&self, req_tunnel);
        {
            self.conn
                .lock()
                .await
                .send_message(Message::NewTunnel(NewTunnel {
                    Url: t.url.clone(),
                    Protocol: t.req.Protocol.clone(),
                    ReqId: t.req.ReqId.clone(),
                    Error: String::new(),
                }))
                .await
                .unwrap();
            //TODO add tunnel to control
        }
        registery::add_tunnel_cache(t.url.clone(), Arc::new(Mutex::new(t))).await;
    }
    pub async fn wait_message(self: &Arc<Self>) -> Result<(), Box<dyn Error>> {
        // let mut lock = conn.lock().await;
        let socket = Conn::get_socket(&self.conn).await;
        loop {
            match message::Message::from_conn(&socket).await? {
                Message::ReqTunnel(req_tunnel) => {
                    log::info!("MESSAG:{:?}", req_tunnel);
                    self.register_tunnel(req_tunnel).await;
                    // lock.register_tunnel(req_tunnel);
                }
                Message::RegProxy(reg_proxy) => {
                    log::info!("Regproxy:{:?}", reg_proxy);
                }
                Message::Ping(_) => {
                    self.conn
                        .lock()
                        .await
                        .send_message(Message::Pong(Pong {}))
                        .await?;
                    log::info!("send pong");
                }
                Message::Unknown(_) => {}
                _ => {
                    //  break;
                }
            }
            //drop(lock);
            //sleep(Duration::from_millis(100)).await;
        }

        // Ok(())
    }

    pub async fn send_message(&self, message: Message) -> Result<(), Box<dyn Error>> {
        let mut lock = self.conn.lock().await;
        lock.send_message(message).await
    }
}
pub async fn handle_ctrl_conn(
    connection: Conn,
    msg: message::auth::AuthReq,
) -> Result<(), Box<dyn Error>> {
    let control = Arc::new(Control {
        conn: Arc::new(Mutex::new(connection)),
        id: match &msg.ClientId[..] {
            // it's a new session, assign an ID
            "" => "1234567890".to_owned(),
            _ => msg.ClientId,
        },
        proxys: Arc::new(Mutex::new(Vec::new())),
    });
    registery::add_control_cache(control.id.clone(), control.clone()).await;

    control
        .send_message(Message::AuthResp(AuthResp {
            Version: VERSON_MARJOR.to_string(),
            MmVersion: VERSON_MINI.to_string(),
            ClientId: control.id.clone(),
            Error: "".to_owned(),
        }))
        .await?;
    control.send_message(Message::ReqProxy(ReqProxy {})).await?;
    let ctrol_clone = control.clone();
    tokio::spawn(async move {
        control.wait_message().await.expect("connection closed");
    });

    Ok(())
}

pub async fn handle_proxy_conn(
    tcpstream: TlsStream<TcpStream>,
    reg_proxy: RegProxy,
) -> Result<(), Box<dyn Error>> {
    if let Some(control) = registery::get_control_cache(&reg_proxy.ClientId) {
        let mut proxy_lock_guard = control.proxys.lock().await;
        proxy_lock_guard.push(tcpstream);
        log::debug!("put proxy into pool");
    } else {
        log::error!("Can't find a control connection with this proxy");
    }

    Ok(())
}
