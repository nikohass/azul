use std::vec;

use clap::Parser;
use config::{Config, File, FileFormat};
use serde::Deserialize;

mod client;
use game::{game_manager::{self, MatchStatistcs}, GameState, Player, NUM_PLAYERS, RuntimeError, SharedState};
use client::Client;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Cli {
    /// Sets a custom config file
    #[clap(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GameConfig {
    pub alternate_starting_player: bool,
    pub num_games: u64,
    pub num_simultaneous_games: u64,
}

#[derive(Debug, Deserialize)]
struct PlayerConfig {
    pub executable: String,
    pub think_time: u64,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    pub game: GameConfig,
    pub player_one: PlayerConfig,
    pub player_two: PlayerConfig,
    pub player_three: Option<PlayerConfig>,
    pub player_four: Option<PlayerConfig>,
}

async fn run_match(players: &mut Vec<Box<dyn Player>>, ) -> Result<MatchStatistcs, RuntimeError> {
    game_manager::run_match(GameState::default(), players).await
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config_file = cli
        .config
        .unwrap_or_else(|| "default_config.toml".to_string());

    let builder = Config::builder()
        .add_source(File::new(&config_file, FileFormat::Toml))
        .build()
        .expect("Failed to build config");

    let app_config: AppConfig = builder
        .try_deserialize()
        .expect("Configuration file format error");

    println!("{:#?}", app_config);

    let mut players: Vec<PlayerConfig> = vec![
        app_config.player_one,
        app_config.player_two,
    ];
    if let Some(player) = app_config.player_three {
        players.push(player);
    }

    if let Some(player) = app_config.player_four {
        players.push(player);
    }

    if players.len() != NUM_PLAYERS {
        panic!("Invalid number of players. Expected {}, got {}", NUM_PLAYERS, players.len());
    }

    let mut clients:Vec<Box<dyn Player>>  = Vec::new();
    for player in players {
        let mut client = Client::from_path(player.executable);
        client.set_time(player.think_time).await;
        clients.push(Box::new(client));
    }

    let player_combinations = player_combinations(NUM_PLAYERS);
    // Set num_games to a multiple of the number of player combinations
    let remainder = app_config.game.num_games % player_combinations.len() as u64;
    let num_games = app_config.game.num_games - remainder;

    let mut game_queue: Vec<Vec<usize>> = Vec::new();
    for _ in 0..num_games / player_combinations.len() as u64 {
        game_queue.extend(player_combinations.clone());
    }

    println!("Length of game queue: {}", game_queue.len());

    let game_queue = SharedState::new(game_queue);

}

fn player_combinations(num_players: usize) -> Vec<Vec<usize>> {
    fn permute(players: &mut Vec<usize>, start: usize, result: &mut Vec<Vec<usize>>) {
        if start == players.len() {
            result.push(players.clone());
            return;
        }
        for i in start..players.len() {
            players.swap(start, i);
            permute(players, start + 1, result);
            players.swap(start, i);
        }
    }

    let mut players = (1..=num_players).collect::<Vec<_>>();
    let mut result = Vec::new();
    permute(&mut players, 0, &mut result);
    result
}