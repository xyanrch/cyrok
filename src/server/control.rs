use crate::registery;
use crate::tunnel::Tunnel;
use bytes::BytesMut;
use cyrok::connection::Conn;
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
use tokio::net::{tcp, TcpListener, TcpStream};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader},
    sync::mpsc,
};
use tokio::{net::TcpSocket, time::error::Elapsed};
use tokio::{sync::Mutex, time::sleep};
use tokio_rustls::server::TlsStream;
#[derive(Debug)]
pub struct Control {
    pub conn: Arc<Conn>,
    pub id: String, //tunnels:Vec<Tunnel>,
    pub proxys: Arc<Mutex<Vec<TlsStream<TcpStream>>>>,
    pub proxy_rx: Mutex<mpsc::Receiver<TlsStream<TcpStream>>>,
    pub proxy_tx: mpsc::Sender<TlsStream<TcpStream>>,
}

impl Control {
    async fn register_tunnel(self: &Arc<Self>, req_tunnel: ReqTunnel) {
        let t = Tunnel::new(&self, req_tunnel);
        {
            self.conn
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
        loop {
            match message::Message::from_conn(&self.conn).await? {
                Message::ReqTunnel(req_tunnel) => {
                    log::info!("MESSAG:{:?}", req_tunnel);
                    self.register_tunnel(req_tunnel).await;
                    // lock.register_tunnel(req_tunnel);
                }
                Message::RegProxy(reg_proxy) => {
                    log::info!("Regproxy:{:?}", reg_proxy);
                }
                Message::Ping(_) => {
                    self.conn.send_message(Message::Pong(Pong {})).await?;
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
        self.conn.send_message(message).await
    }
    pub async fn get_proxy_conn(&self) -> TlsStream<TcpStream> {
        //let mut rx:mpsc::Receiver<TlsStream<TcpStream>> = self.proxy_rx.clone();
        let mut rx = self.proxy_rx.lock().await;
        if let Ok(proxy) = rx.try_recv() {
            return proxy;
        }
        log::info!("send reqproxy to client");

        self.conn
            .send_message(Message::ReqProxy(ReqProxy {}))
            .await
            .unwrap();

        rx.recv().await.unwrap()

        // else {
        // proxy_guard.pop().unwrap()

        //  }
    }
}
pub async fn handle_ctrl_conn(
    connection: Conn,
    msg: message::auth::AuthReq,
) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<TlsStream<TcpStream>>(10);
    let control = Arc::new(Control {
        conn: Arc::new(connection),
        id: match &msg.ClientId[..] {
            // it's a new session, assign an ID
            "" => "1234567890".to_owned(),
            _ => msg.ClientId,
        },
        proxys: Arc::new(Mutex::new(Vec::new())),
        proxy_rx: Mutex::new(rx),
        proxy_tx: tx,
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
        // let mut proxy_lock_guard = control.proxys.lock().await;
        //  proxy_lock_guard.push(tcpstream);
        log::debug!("will sent signal");
        control.proxy_tx.send(tcpstream).await?;
        log::debug!("put proxy into pool");
    } else {
        log::error!("Can't find a control connection with this proxy");
    }

    Ok(())
}
