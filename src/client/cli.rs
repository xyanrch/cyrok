
use clap::{App, AppSettings, Arg, SubCommand};
use std::env;
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Options {
    pub configpath: String,
    pub logpath: String,
    pub loglevel: String,
    /*authtoken String
    httpauth  String
    hostname  String
    protocol  String
    subdomain String
    command   String
    */
    pub command: (Option<String>, Option<Vec<String>>),
}

impl Options {
    pub fn parse() -> Options {
        let matches = App::new("cyrok_client")
            .version("0.1")
            .author("cyr")
            .about("cyrok client app")
            .setting(AppSettings::ColoredHelp)
            .arg(Arg::from_usage("--config [config] 'Path to ngrok configuration file. (default: .cyrok)'"))
            .arg(Arg::from_usage("--log_path [log_path] 'Write log messages to this file. (default: none)"))
            .arg(Arg::from_usage("--log_level [log_level] 'The level of messages to log. One of: DEBUG, NOTICE, ERROR (default: NOTICE)"))
            .subcommand(SubCommand::with_name("list")
                            .about("List tunnel names from config file"))
            .subcommand(SubCommand::with_name("start")
                            .about("Start tunnels by name from config file")
                            .arg(Arg::with_name("names")
                            .long("names")
                            .takes_value(true)
                            .multiple(true)
                            .help("tunnel names defined in config file")))
            .subcommand(SubCommand::with_name("start_all")
                            .about("Start all tunnels defined in config file"))   
            .get_matches();

        Options {
            configpath: matches
                .value_of("config")
                .unwrap_or({
                    let mut dir = env::current_exe().unwrap();
                    dir.pop();
                    dir.join("client.json").to_str().unwrap()
                })
                .parse::<String>()
                .unwrap(),
            logpath: matches
                .value_of("log_path")
                .unwrap_or("none")
                .parse::<String>()
                .unwrap(),
            loglevel: matches
                .value_of("log_level")
                .unwrap_or("NOTICE")
                .parse::<String>()
                .unwrap(),
            command: match matches.subcommand_name() {
                Some("list") => (Some(String::from("list")), None),
                Some("start") => (Some(String::from("start")), {
                    if let Some(sub_m) = matches.subcommand_matches("start") {
                        Some(
                            sub_m
                                .values_of("names")
                                .unwrap()
                                .map(|x| x.to_string())
                                .collect(),
                        )
                    } else {
                        None
                    }
                }),
                Some("start_all") => (Some(String::from("start_all")), None),
                None => (None, None),
                Some(&_) => (None, None),
            },
        }
    }
}
