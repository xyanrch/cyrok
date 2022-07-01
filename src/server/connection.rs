

use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;
pub struct Conn
{
    pub tls_socket:TlsStream<TcpStream>,

}
impl Conn
{
    pub fn new(socket:TlsStream<TcpStream>)->Conn
    {
        Conn{
            tls_socket:socket
        }
    }
}