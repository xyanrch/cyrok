// A client sends this message to the server over the control channel
// to request a new tunnel be opened on the client's behalf.
// ReqId is a random number set by the client that it can pull
// from future NewTunnel's to correlate then to the requesting ReqTunnel.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ReqTunnel {
	ReqId :   String,
	Protocol: String,

	// http only
	Hostname:  String,
	Subdomain: String,
	HttpAuth:  String,

	// tcp only
	RemotePort: u16
}