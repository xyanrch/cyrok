pub mod auth;
pub mod tunnel;
use auth::AuthReq;
use auth::AuthResp;
use tunnel::ReqTunnel;

use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use std::io::Error;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Envelope {
    pub Type: String,
    pub Payload: serde_json::Value,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Unknown {
    pub err: String,
}
#[derive(Debug)]
pub enum Message {
    AuthReq(AuthReq),
    AuthResp(AuthResp),
    ReqTunnel(ReqTunnel),
// NewTunnel(NewTunnel),
    // RegProxy(RegProxy),
    // StartProxy(StartProxy),
    Unknown(Unknown),
}
impl Message {
    pub async fn from_conn( tls_socket:&mut TlsStream<TcpStream>)->std::result::Result<Message, Error>
    {
        let len = tls_socket.read_u64_le().await?;
        log::info!("receive message len:{:?}", len);
        let mut buf = BytesMut::with_capacity(len.try_into().unwrap());
        tls_socket.read_buf(&mut buf).await?;
        log::info!("receive message {:?}", buf);
        let raw: Envelope = serde_json::from_slice(&mut buf)?;
        log::info!("receive parsed message {:?}", raw);
        Message::frome_envelop(raw)
    }
    pub fn frome_envelop(val: Envelope) -> std::result::Result<Message, Error> {
        log::info!("test :{:?}", val.Payload);
        let  message = match &val.Type[..] {
            "Auth" => Message::AuthReq(serde_json::from_value(val.Payload).unwrap()),
            "AuthRes" => Message::AuthResp(serde_json::from_value(val.Payload).unwrap()),
            "ReqTunnel" =>Message::ReqTunnel(serde_json::from_value(val.Payload).unwrap()),
            _ => Message::Unknown(Unknown {
                err: String::from("unkown"),
            }),
        };
        Ok(message)
    }
    pub fn to_envelop(&self) -> Envelope {
        Envelope {
            Type: match self {
                Message::AuthReq(_) => "Auth".to_owned(),
                Message::AuthResp(_) => "AuthResp".to_owned(),
                Message::ReqTunnel(_)=>"ReqTunnel".to_owned(),
                Message::Unknown(_) => "unkown".to_owned(),
            },

            Payload: match self {
                Message::AuthReq(auth) => serde_json::to_value(auth).unwrap(),
                Message::AuthResp(auth_resp) => serde_json::to_value(auth_resp).unwrap(),
                Message::ReqTunnel(req_tunnel)=>serde_json::to_value(req_tunnel).unwrap(),
                Message::Unknown(unknown) => serde_json::to_value(unknown).unwrap(),
            },
        }
    }
}
