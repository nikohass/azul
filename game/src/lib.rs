mod error;
mod factories;
mod game_state;
pub mod match_;
mod move_;
mod move_list;
mod player;
mod shared_state;
mod tile_color;
pub mod wall;

pub use error::RuntimeError;
pub use factories::{Factories, Factory};
pub use factories::{CENTER_FACTORY_INDEX, NUM_FACTORIES};
pub use game_state::GameState;
pub use game_state::{bag_to_string, factories_to_string, Bag};
pub use game_state::{FLOOR_LINE_PENALTY, NUM_PLAYERS};
pub use move_::Move;
pub use move_list::MoveList;
pub use player::{Player, PlayerMarker};
pub use shared_state::SharedState;
pub use tile_color::TileColor;
pub use tile_color::NUM_TILE_COLORS;
pub use wall::field_at;

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
