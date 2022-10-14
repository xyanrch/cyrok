#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AuthReq {
    pub Version: String,   // protocol version
    pub MmVersion: String, // major/minor software version (informational only)
    pub User: String,
    pub Password: String,
    pub OS: String,
    pub Arch: String,
    pub ClientId: String, // empty for new sessions
}

// A server responds to an Auth message with an
// AuthResp message over the control channel.
//
// If Error is not the empty String
// the server has indicated it will not accept
// the new session and will close the connection.
//
// The server response includes a unique ClientId
// that is used to associate and authenticate future
// proxy connections via the same field in RegProxy messages.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AuthResp {
    pub Version: String,
    pub MmVersion: String,
    pub ClientId: String,
    pub Error: String,
}
