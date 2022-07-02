use crate::connection::Conn;
use bytes::BytesMut;
use cyrok::{message::{Message,self, auth::AuthResp}, VERSON_MARJOR, VERSON_MINI};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::{error::Error, sync::Arc};
use crate::registery;
#[derive(Debug)]
pub struct Control {
    pub conn: Conn,
    pub id: String, //tunnels:Vec<Tunnel>,
}
impl Control {
    pub async fn wait_message (&mut self) ->Result<(), Box<dyn Error >> {
        match message::Message::from_conn(&mut self.conn.tls_socket).await?
        {
            Message::ReqTunnel(req_tunnel) => {

                log::info!("MESSAG:{:?}",req_tunnel);
    
            }
            Message::Unknown(_) => {}
            _ => {}
        }
        
        Ok(())
        
    }
    
}
pub async fn handle_ctrl_conn(
    connection: Conn,
    msg: message::auth::AuthReq,
) -> Result<(), Box<dyn Error>> {
    let mut control = Control {
        conn: connection,
        id: msg.ClientId,
    };
    if control.id == "" {
        //TODO
		// it's a new session, assign an ID
        control.id = "1234567890".to_owned();
	}
    let resp= Message::AuthResp(AuthResp{
        Version:VERSON_MARJOR.to_string(),
        MmVersion:VERSON_MINI.to_string(),
        ClientId:control.id.clone(),
        Error:"".to_owned()

    });
    control.conn.conn_type = Some("ctrl".to_owned());
    let st = serde_json::to_string(&resp.to_envelop())?; 
    control.conn.tls_socket.write_i64_le(st.len().try_into().unwrap()).await?; 
    control.conn.tls_socket.write_all(st.as_bytes()).await?;
    control.wait_message().await?;
    registery::add_control_cache(control).await;

   /// let arc_control = registery::get_control_cache(id)




    
    Ok(())
}
