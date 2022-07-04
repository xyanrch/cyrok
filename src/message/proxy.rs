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