#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AuthReq {
    Version: String,   // protocol version
    MmVersion: String, // major/minor software version (informational only)
    User: String,
    Password: String,
    OS: String,
    Arch: String,
    ClientId: String, // empty for new sessions
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
    version: String,
    mmversion: String,
    clien_id: String,
    error: String,
}
