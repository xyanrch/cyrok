use clap::{App, AppSettings, Arg, SubCommand};
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    DEBUG = 0,
    NOTICE = 1,
    ERROR = 2,
}
impl From<&str> for LogLevel {
    fn from(raw: &str) -> LogLevel {
        match raw {
            "DEBUG" => LogLevel::DEBUG,
            "NOTICE" => LogLevel::NOTICE,
            "ERROR" => LogLevel::ERROR,
            &_ => LogLevel::NOTICE,
        }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Options {
    pub config: String,
    pub logto: String,
    pub loglevel: LogLevel,
    /*authtoken String
    httpauth  String
    hostname  String
    protocol  String
    subdomain String
    command   String
    */
    command: Option<String>,
    //command_args: Option<Vec<String>>,
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
        // let mut subarg: Option<Vec<&'static str>> = None;
        let command = match matches.subcommand_name() {
            Some("list") => Some(String::from("list")),
            Some("start") => {
                if let Some(sub_m) = matches.subcommand_matches("start") {
                    let tmp: Vec<&str> = sub_m.values_of("names").unwrap().collect();
                    print! {"tmp{:?}",tmp}
                }
                // subarg = Some(tmp);
                Some(String::from("start"))
            }
            Some("start_all") => Some(String::from("start_all")),
            None => None,
            Some(&_) => None,
        };

        Options {
            config: matches
                .value_of("config")
                .unwrap_or(".cyrok")
                .parse::<String>()
                .unwrap(),
            logto: matches
                .value_of("log_path")
                .unwrap_or("none")
                .parse::<String>()
                .unwrap(),
            loglevel: matches
                .value_of("log_level")
                .unwrap_or("NOTICE")
                .parse::<String>()
                .unwrap()
                .as_str()
                .into(),
            // command:matches.subcommand_matches("list").unwrap(),
            command: command,
            // command_args:subarg
        }
    }
}
