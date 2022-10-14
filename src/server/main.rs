extern crate lazy_static;
mod cli;
mod service;
use rustls_pemfile::{certs, rsa_private_keys};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use tokio::io::{copy, sink, split, AsyncWriteExt};
//use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::{self, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;

use crate::cli::Options;
mod control;
mod registery;
mod tunnel;
fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    rsa_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = cli::Options::instance();

    flexi_logger::Logger::try_with_str("debug")
        .unwrap()
        .start()
        .expect("the logger should start");
    log::info!("server begin");
    log::info!("opts:{:?}", opts);
    //
    let certs = load_certs(Path::new("assets/server/tls/snakeoil.crt"))?;
    let mut keys = load_keys(Path::new("assets/server/tls/snakeoil.key"))?;

    let tlsconfig = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    //
    let tls_acceptor = TlsAcceptor::from(Arc::new(tlsconfig));

    let data_listener = TcpListener::bind(opts.http_addr).await?;
    let ctrl_listener = TcpListener::bind("0.0.0.0:4443").await?;
    let https_listener = match opts.https_addr {
        Some(https_addr) => Some(TcpListener::bind(https_addr).await?),

        None => None,
    };

    service::run(
        tls_acceptor,
        ctrl_listener,
        data_listener,
        // https_listener,
        tokio::signal::ctrl_c(),
    )
    .await;
    registery::dump_control_registery().await;
    registery::dump_tunnel_registery().await;
    log::info!("server shutdown");

    Ok(())
}
