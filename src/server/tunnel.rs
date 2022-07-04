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
	url: String,

	// tcp listener
	listener: TcpListener,

	// control connection
	ctl: Arc<Control>

}
impl Tunnel{
    pub fn new()->Tunnel
    {

    }
}

	// closing
