pub mod connection;
pub use connection::Conn;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

pub type TlsConnnection = Conn<TlsStream<TcpStream>>;
pub type TcpConnnection = Conn<TcpStream>;
