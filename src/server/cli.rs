use clap::{App, AppSettings, Arg};
use once_cell::sync::OnceCell;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
#[derive(Debug)]
pub struct Options {
    pub http_addr: SocketAddr,
    pub https_addr: Option<SocketAddr>,
    //httpsAddr  string
    //tunnelAddr string
    pub domain: String, //tlsCrt     string
                        //tlsKey     string
                        //logto      string
                        //loglevel   string
}
static INSTANCE: OnceCell<Options> = OnceCell::new();
impl Options {
    pub fn instance() -> &'static Options {
        INSTANCE.get_or_init(|| Options::parse())
    }
    fn parse() -> Options {
        let matches = App::new("cyrok_server")
            .version("0.1")
            .author("cyr")
            .about("cyrok server app")
            .setting(AppSettings::ColoredHelp)
            .arg(Arg::from_usage("--httpaddr [httpaddr] 'Public address for HTTP connections, empty string to disable'"))
            .arg(Arg::from_usage("--httpsaddr [httpsaddr] 'Public address for HTTPs connections, empty string to disable'"))
            .arg(Arg::from_usage("--domain [domain] 'Domain where the tunnels are hosted'"))
            .get_matches();

        Options {
            http_addr: matches
                .value_of("httpaddr")
                .unwrap_or("0.0.0.0:7777")
                .parse::<SocketAddr>()
                .unwrap(),
            https_addr: match matches
                .value_of("httpsaddr")
                .unwrap_or("0.0.0.0:4444")
                .parse::<SocketAddr>()
            {
                Ok(addr) => Some(addr),
                Err(err) => {
                    print!("error:{}", err);
                    None
                }
            },

            domain: matches
                .value_of("domain")
                .unwrap_or("ngrok.me")
                .parse::<String>()
                .unwrap(),
        }
    }
}
