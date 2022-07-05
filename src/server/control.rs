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
    pub proxys: Vec<(AtomicBool, TlsStream<TcpStream>)>,
}
pub async fn register_tunnel(c: &Arc<Mutex<Control>>, req_tunnel: ReqTunnel) {
    let t = Tunnel::new(&c, req_tunnel);
    {
        let conn = Control::get_conn(c).await;
        conn.lock()
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
impl Control {
    pub async fn get_conn(c: &Arc<Mutex<Control>>) -> Arc<Mutex<Conn>> {
        let lock = c.lock().await;
        let conn = lock.conn.clone();
        conn
    }
    pub async fn wait_message(c: Arc<Mutex<Control>>) -> Result<(), Box<dyn Error>> {
        let conn = Control::get_conn(&c).await;
        // let mut lock = conn.lock().await;
        let socket = Conn::get_socket(&conn).await;
        loop {
            match message::Message::from_conn(&socket).await? {
                Message::ReqTunnel(req_tunnel) => {
                    log::info!("MESSAG:{:?}", req_tunnel);
                    register_tunnel(&c, req_tunnel).await;
                    // lock.register_tunnel(req_tunnel);
                }
                Message::RegProxy(reg_proxy) => {
                    log::info!("Regproxy:{:?}", reg_proxy);
                }
                Message::Ping(_) => {
                    conn.lock()
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

    pub async fn send_message(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        let mut lock = self.conn.lock().await;
        lock.send_message(message).await
    }
}
pub async fn handle_ctrl_conn(
    connection: Conn,
    msg: message::auth::AuthReq,
) -> Result<(), Box<dyn Error>> {
    let control = Arc::new(Mutex::new(Control {
        conn: Arc::new(Mutex::new(connection)),
        id: match &msg.ClientId[..] {
            // it's a new session, assign an ID
            "" => "1234567890".to_owned(),
            _ => msg.ClientId,
        },
        proxys: Vec::new(),
    }));
    let mut lock_guard = control.lock().await;
    registery::add_control_cache(lock_guard.id.clone(), control.clone()).await;

    let resp = Message::AuthResp(AuthResp {
        Version: VERSON_MARJOR.to_string(),
        MmVersion: VERSON_MINI.to_string(),
        ClientId: lock_guard.id.clone(),
        Error: "".to_owned(),
    });

    lock_guard.send_message(resp).await?;
    lock_guard
        .send_message(Message::ReqProxy(ReqProxy {}))
        .await?;
    let ctrol_clone = control.clone();
    tokio::spawn(async move {
        Control::wait_message(ctrol_clone)
            .await
            .expect("connection closed");
    });

    // curr.send_message(Message::ReqProxy(ReqProxy {})).await?;

    /// let arc_control = registery::get_control_cache(id)
    Ok(())
}

pub async fn handle_proxy_conn(
    connection: TlsStream<TcpStream>,
    reg_proxy: RegProxy,
) -> Result<(), Box<dyn Error>> {
    if let Some(control) = registery::get_control_cache(&reg_proxy.ClientId) {
        let mut lock_guard = control.lock().await;
        
        lock_guard.proxys.push((AtomicBool::new(false), connection));
        log::info!("put proxy into pool");
    } else {
        log::error!("Can't find a control connection with this proxy");
    }

    Ok(())
}
