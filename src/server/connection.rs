

use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
#[derive(Debug)]
pub struct Conn
{
    pub tls_socket:TlsStream<TcpStream>,
    pub conn_type:Option<String>

}
impl Conn
{
    pub fn new(socket:TlsStream<TcpStream>,conn_type:Option<String>)->Conn
    {
        Conn{
            tls_socket:socket,
            conn_type
        }
    }
}