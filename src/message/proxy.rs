// When the server wants to initiate a new tunneled connection, it sends
// this message over the control channel to the client. When a client receives
// this message, it must initiate a new proxy connection to the server.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ReqProxy  {
}
// After a client receives a ReqProxy message, it opens a new
// connection to the server and sends a RegProxy message.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct  RegProxy  {
	pub ClientId: String
}


// This message is sent by the server to the client over a *proxy* connection before it
// begins to send the bytes of the proxied request.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct StartProxy  {
	pub Url  :      String, // URL of the tunnel this connection connection is being proxied for
	pub ClientAddr: String // Network address of the client initiating the connection to the tunnel
}