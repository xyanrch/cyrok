mod cli;
mod config;
use flexi_logger::FileSpec;
fn main() {
    let options = cli::Options::parse();
    let config = config::Configuration::load_config(options.configpath.as_str());
    if config.log.to_file {
        flexi_logger::Logger::try_with_str("info")
            .unwrap()
            .log_to_file(
                FileSpec::default()
                    .basename(config.log.base_name.as_str())
                    .directory(config.log.dir.as_str()),
            )
            .start()
            .expect("the logger should start");
    } else {
        flexi_logger::Logger::try_with_str(config.log.loglevel.as_str())
            .unwrap()
            .start()
            .expect("the logger should start");
    }
    log::debug!("test");
    log::error!("this is a error");
    log::info!("options:{:?}", options);
    log::info!("options:{:?}", config);
}
