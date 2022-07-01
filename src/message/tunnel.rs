// A client sends this message to the server over the control channel
// to request a new tunnel be opened on the client's behalf.
// ReqId is a random number set by the client that it can pull
// from future NewTunnel's to correlate then to the requesting ReqTunnel.
pub struct ReqTunnel {
	reqid :   String,
	protocol: String,

	// http only
	hostname:  String,
	subdomain: String,
	httpauth:  String,

	// tcp only
	remoteport: u16
}