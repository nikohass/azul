use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

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

    config_builder = config_builder.logger(Logger::builder().build("log_file", LevelFilter::Debug));

    let config = config_builder
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}
