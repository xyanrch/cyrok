use crate::control::Control;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, RwLock};
extern crate lazy_static;

lazy_static::lazy_static! {
    static  ref  CONTROL_REGISTERY : RwLock<HashMap<String, Arc<Control>>> = RwLock::new(HashMap::new());

}
pub fn get_control_cache(id: &str) -> Option<Arc<Control>> {
    let mut control: Option<Arc<Control>> = None;
    if let Ok(lock) = CONTROL_REGISTERY.read() {
        control = Some(lock.get(&id.to_owned()).unwrap().clone());
    }

    control
}
pub  async fn add_control_cache(ctrl: Control) -> Option<Arc<Control>> {
    let mut old: Option<Arc<Control>> = None;
    if let Ok(mut wlock) = CONTROL_REGISTERY.write() {
        if wlock.contains_key(&ctrl.id) {
            old = wlock.remove(&ctrl.id);
        }
        wlock.insert(ctrl.id.clone(), Arc::new(ctrl));
    }
    old
}
pub fn dump_control_registery() {
    if let Ok(lock) = CONTROL_REGISTERY.read() {
        print!("dump {:?}", *lock);
    }
}
