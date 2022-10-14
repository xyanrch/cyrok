use crate::registery;
use crate::{cli::Options, control::Control};
use cyrok::message::tunnel::ReqTunnel;
use std::{fmt::format, sync::Arc, sync::Weak};
use tokio::sync::Mutex;

/**
 * Tunnel: A control connection, metadata and proxy connections which
 *         route public traffic to a firewalled endpoint.
 */
#[derive(Debug)]
pub struct Tunnel {
    // request that opened the tunnel
    pub req: ReqTunnel,

    // time when the tunnel was opened
    //start time.Time

    // public url
    pub url: String,

    // tcp listener
    //listener: TcpListener,

    // control connection
    pub ctl: Weak<Control>,
}
impl Tunnel {
    pub fn new(c: &Arc<Control>, req_tunnel: ReqTunnel) -> Tunnel {
        /* let url =match req_tunnel.Protocol.as_str()
        {
            "tcp" =>
            {
                ""
            }
            "http" =>
            {
               // register_vhost(tunnel,req_tunnel.Protocol.clone(),Options::instance().http_addr.port())
            }
            &_ =>{
                "unkown"
            }

        };*/
        let proto = req_tunnel.Protocol.clone();
        let mut tunnel = Tunnel {
            req: req_tunnel,
            url: "url".to_owned(),
            ctl: Arc::downgrade(&c),
        };
        register_vhost(&mut tunnel, proto, Options::instance().http_addr.port());
        tunnel
    }
}

// Common functionality for registering virtually hosted protocols
pub fn register_vhost(t: &mut Tunnel, protocol: String, serving_port: u16) {
    let vhost = format!("{}:{}", Options::instance().domain, serving_port);
    // Register for specific subdomain
    //subdomain := strings.ToLower(strings.TrimSpace(t.req.Subdomain))
    if t.req.Subdomain != "" {
        t.url = format!("{}://{}.{}", protocol, t.req.Subdomain.clone(), vhost);
    }
    //if subdomain != "" {
    //	t.url = fmt.Sprintf("%s://%s.%s", protocol, subdomain, vhost)
    //	return tunnelRegistry.Register(t.url, t)
    //}
}

// closing
