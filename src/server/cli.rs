
use clap::{App, AppSettings, Arg};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
#[derive(Debug)]
pub struct  Options  {
	pub http_addr :  SocketAddr,
	//httpsAddr  string
	//tunnelAddr string
	pub domain :    String
	//tlsCrt     string
	//tlsKey     string
	//logto      string
	//loglevel   string
}
impl Options {
    pub fn parse() -> Options {
        let matches = App::new("cyrok_server")
            .version("0.1")
            .author("cyr")
            .about("cyrok server app")
            .setting(AppSettings::ColoredHelp)
            .arg(Arg::from_usage("--httaddr [httpaddr] 'Public address for HTTP connections, empty string to disable'"))
            .arg(Arg::from_usage("--domain [domain] 'Domain where the tunnels are hosted'"))
            .get_matches();

            Options {
                http_addr: matches
                    .value_of("httpaddr")
                    .unwrap_or("0.0.0.0:7777")
                    .parse::<SocketAddr>()
                    .unwrap(),
                domain: matches
                    .value_of("domain")
                    .unwrap_or("cyrok.com")
                    .parse::<String>()
                    .unwrap(),
                }
    }
}

