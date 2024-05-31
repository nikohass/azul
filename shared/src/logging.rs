use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::collections::HashMap;

lazy_static::lazy_static!(
    pub static ref CREATE_FILTERS: HashMap<&'static str, LevelFilter> = {
        let mut filters = HashMap::new();
        filters.insert("game", LevelFilter::Debug);
        filters.insert("player", LevelFilter::Debug);
        filters.insert("replay_buffer", LevelFilter::Debug);
        filters.insert("python_package", LevelFilter::Debug);
        filters.insert("playground", LevelFilter::Debug);
        filters.insert("azul", LevelFilter::Debug);
        filters.insert("test_server", LevelFilter::Debug);
        filters.insert("test_client", LevelFilter::Debug);
        filters
    };
);

pub fn init_logging(log_file: &str) {
    // If the log directory doesn't exist, create it
    if !std::path::Path::new("logs").exists() {
        std::fs::create_dir("logs").unwrap();
    }

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S%.3f%Z)} - {h({l})} - {m}{n}",
        )))
        .build();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S%.3f%Z)} - {l} - {m}{n}",
        )))
        .build(format!("logs/{}.log", log_file))
        .unwrap();

    let mut config_builder = Config::builder();
    config_builder = config_builder
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)));

    for (crate_name, filter_level) in CREATE_FILTERS.iter() {
        config_builder = config_builder.logger(Logger::builder().build(*crate_name, *filter_level));
    }

    let config = config_builder
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Info),
        )
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}
