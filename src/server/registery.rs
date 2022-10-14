use crate::control::Control;
use crate::tunnel::Tunnel;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
//use tokio::sync::RwLock;
extern crate lazy_static;
type SharedControl = Arc<Control>;
type SharedTunnel = Arc<Tunnel>;
lazy_static::lazy_static! {
    static  ref  CONTROL_REGISTERY : RwLock<HashMap<String, Arc<Control>>> = RwLock::new(HashMap::new());
    static  ref  TUNNEL_REGISTERY : RwLock<HashMap<String, Arc<Tunnel>>> = RwLock::new(HashMap::new());
}
pub fn get_control_cache(id: &str) -> Option<SharedControl> {
    let mut control: Option<SharedControl> = None;
    if let Ok(lock) = CONTROL_REGISTERY.read() {
        control = Some(lock.get(id).unwrap().clone());
    }

    control
}
pub async fn add_control_cache(id: String, ctrl: SharedControl) -> Option<SharedControl> {
    let mut old: Option<SharedControl> = None;
    if let Ok(mut lock) = CONTROL_REGISTERY.write() {
        //let key = c.id.clone();
        // drop(c);
        old = lock.insert(id, ctrl);
    }
    old
}
pub async fn dump_control_registery() {
    if let Ok(lock) = CONTROL_REGISTERY.read() {
        for (k, v) in &*lock {
            print!("dump [{}:{:?}]", k, v);
        }
        //  print!("dump {:?}", *lock);
    }
}
pub fn get_tunnel_cache(id: &str) -> Option<SharedTunnel> {
    let mut t: Option<SharedTunnel> = None;
    if let Ok(lock) = TUNNEL_REGISTERY.read() {
        log::debug!("ID:{}", id);
        if let Some(tunnel) = lock.get(id) {
            t = Some(tunnel.clone());
        }
    }

    t
}

pub async fn add_tunnel_cache(id: String, t: SharedTunnel) -> Option<SharedTunnel> {
    let mut old: Option<SharedTunnel> = None;
    if let Ok(mut lock) = TUNNEL_REGISTERY.write() {
        log::debug!("add tunnel {}:{:?}", id, t);
        old = lock.insert(id, t);
    }
    old
}
pub async fn dump_tunnel_registery() {
    if let Ok(lock) = TUNNEL_REGISTERY.read() {
        print!("dump tunnel；{:?}", lock);
    }
}
