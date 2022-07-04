use crate::{control::Control, cli::Options};
use cyrok::message::tunnel::ReqTunnel;
use std::{sync::Arc, sync::Weak, fmt::format};
use tokio::sync::Mutex;

/**
 * Tunnel: A control connection, metadata and proxy connections which
 *         route public traffic to a firewalled endpoint.
 */
pub struct Tunnel {
    // request that opened the tunnel
    //req *msg.ReqTunnel

    // time when the tunnel was opened
    //start time.Time

    // public url
    pub url: String,

    // tcp listener
    //listener: TcpListener,

    // control connection
    pub ctl: Weak<Mutex<Control>>,
}
impl Tunnel {
    pub fn new(c: &Arc<Mutex<Control>>, req_tunnel:&ReqTunnel) -> Tunnel {
        let url =match req_tunnel.Protocol.as_str()
        {
            "tcp" =>
            {
                ""
            }
            "http" =>
            {
              ""
            }
            &_ =>{
                "unkown"
            }

        };
        let tunnel = Tunnel {
            url: req_tunnel.Protocol.clone(),
            ctl: Arc::downgrade(&c),
        };
        tunnel
    }
}

// Common functionality for registering virtually hosted protocols
pub fn register_vhost(t :&Tunnel, protocol: String, serving_port:u16) {


    let vhost = format!("{}:{}",Options.domain,server_port);
}

// closing
