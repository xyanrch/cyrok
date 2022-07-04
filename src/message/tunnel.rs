// A client sends this message to the server over the control channel
// to request a new tunnel be opened on the client's behalf.
// ReqId is a random number set by the client that it can pull
// from future NewTunnel's to correlate then to the requesting ReqTunnel.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ReqTunnel {
    pub ReqId: String,
    pub Protocol: String,

    // http only
    pub Hostname: String,
    pub Subdomain: String,
    pub HttpAuth: String,

    // tcp only
    pub RemotePort: u16,
}

// When the server opens a new tunnel on behalf of
// a client, it sends a NewTunnel message to notify the client.
// ReqId is the ReqId from the corresponding ReqTunnel message.
//
// A client may receive *multiple* NewTunnel messages from a single
// ReqTunnel. (ex. A client opens an https tunnel and the server
// chooses to open an http tunnel of the same name as well)
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct NewTunnel {
    pub ReqId: String,
    pub Url: String,
    pub Protocol: String,
    pub Error: String,
}
