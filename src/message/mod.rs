mod auth;
use auth::AuthReq;
use auth::AuthResp;
use std::io::Error;
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Envelope {
    pub Type: String,
    pub Payload: serde_json::Value,
}
#[derive(Debug)]
pub struct Unknown {
    pub err: String,
}
#[derive(Debug)]
pub enum Message {
    AuthReq(AuthReq),
    AuthResp(AuthResp),
    //ReqTunnel(ReqTunnel),
    // NewTunnel(NewTunnel),
    // RegProxy(RegProxy),
    // StartProxy(StartProxy),
    Unknown(Unknown),
}
impl Message {
    pub fn frome_envelop(val: Envelope) -> std::result::Result<Message, Error> {
        log::info!("test :{:?}",val.Payload);
        let mut message = match &val.Type[..] {
            "Auth" => Message::AuthReq(serde_json::from_value(val.Payload).unwrap()),
            _ => Message::Unknown(Unknown {
                err: String::from("unkown"),
            }),
        };
        Ok(message)
    }
}
