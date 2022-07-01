use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::net::IpAddr;
use std::io::Write;
//use serde_json::Result;
/*#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub server_addr: IpAddr,    
    /*Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    */
    pub log:LogConfiguration,
    pub tunnels: Vec<TunnelConfiguration>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct LogConfiguration{
    pub loglevel:String,
    pub to_file:bool,
    pub base_name:String,
    pub dir:String

}
#[derive(Serialize, Deserialize, Debug)]
pub struct TunnelConfiguration {
    pub sub_domain: String,
    pub host_name: String,
    pub protocal: String,
    pub httpauth: String,
    pub remote_port: u16,
}
impl Configuration {
    pub fn load_config(path: &str) -> Configuration {
        println!("path {}", path);
        let reader = BufReader::new(File::open(path).unwrap());
        serde_json::from_reader(reader).expect("file not found")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_config_from_json() {
        let data = r#"
        {
            "server_addr": "1.1.1.1",
            "log": {
                "loglevel": "info",
                "to_file": true,
                "base_name":"xxlog",
                "dir":"./log"
               
        
            },
            "tunnels":[
                {
                    "sub_domain":"/test11",
                    "host_name":"host1",
                    "protocal":"tcp",
                    "httpauth":"auth1",
                    "remote_port":22
                },
                {
                    "sub_domain":"/test12",
                    "host_name":"host2",
                    "protocal":"http",
                    "httpauth":"auth2",
                    "remote_port":80

                }

            ]

        }"#;
        let v: Configuration = serde_json::from_str(data).expect("parse json failed");
        assert_eq!(v.log.to_file, true);
        assert_eq!(v.server_addr.to_string(), "1.1.1.1");
        assert_eq!(v.log.loglevel, "info");
        let outstr = serde_json::to_string(&v).unwrap();
        let mut file = File::create("test.json").unwrap();
        file.write_all(outstr.as_bytes()).expect("Write error");
        let newcfg: Configuration = Configuration::load_config("test.json");
        assert_eq!(v.log.loglevel, newcfg.log.loglevel);
        assert_eq!(v.log.to_file, newcfg.log.to_file);

        println!("configure after {:?}", v);
    }
}
