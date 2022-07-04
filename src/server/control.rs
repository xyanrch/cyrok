use crate::connection::Conn;
use crate::registery;
use bytes::BytesMut;
use cyrok::{
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
use log::info;
use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc, RwLock},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::error::Elapsed,
};
use tokio::{sync::Mutex, time::sleep};
#[derive(Debug)]
pub struct Control {
    pub conn: Conn,
    pub id: String, //tunnels:Vec<Tunnel>,
    pub proxys: Vec<(AtomicBool, Conn)>,
}
impl Control {
    pub async fn wait_message(c: Arc<Mutex<Control>>) -> Result<(), Box<dyn Error>> {
        loop {
            let mut lock = c.lock().await;
            match message::Message::from_conn(&mut lock.conn.tls_socket).await? {
                Message::ReqTunnel(req_tunnel) => {
                    log::info!("MESSAG:{:?}", req_tunnel);
                    lock.register_tunnel(req_tunnel);
                }
                Message::RegProxy(reg_proxy) => {
                    log::info!("Regproxy:{:?}", reg_proxy);
                }
                Message::Ping(_) => {
                    lock.send_message(Message::Pong(Pong {})).await?;
                }
                Message::Unknown(_) => {}
                _ => {
                    //  break;
                }
            }
            drop(lock);
            sleep(Duration::from_millis(100)).await;
        }

        // Ok(())
    }
    pub async fn send_message(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
        let raw = serde_json::to_string(&message.to_envelop())?;
        self.conn
            .tls_socket
            .write_i64_le(raw.len().try_into().unwrap())
            .await?;
        self.conn.tls_socket.write_all(raw.as_bytes()).await?;

        Ok(())
    }
    pub fn register_tunnel(&mut self, req_tunnel: ReqTunnel) {}
}
pub async fn handle_ctrl_conn(
    connection: Conn,
    msg: message::auth::AuthReq,
) -> Result<(), Box<dyn Error>> {
    let control = Arc::new(Mutex::new(Control {
        conn: connection,
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
  
        Control::wait_message(ctrol_clone).await.unwrap();
     });

    // curr.send_message(Message::ReqProxy(ReqProxy {})).await?;

    /// let arc_control = registery::get_control_cache(id)
    Ok(())
}

pub async fn handle_proxy_conn(
    connection: Conn,
    reg_proxy: RegProxy,
) -> Result<(), Box<dyn Error>> {
    if let Some(control) = registery::get_control_cache(&reg_proxy.ClientId) {
        let mut lock_guard = control.lock().await;
        log::info!("Find a control connection");
        lock_guard.proxys.push((AtomicBool::new(false), connection));
    } else {
        log::error!("Can't find a control connection with this proxy");
    }

    Ok(())
}
