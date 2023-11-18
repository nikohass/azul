use std::vec;

use clap::Parser;
use config::{Config, File, FileFormat};
use serde::Deserialize;

mod client;
use client::Client;
use game::{
    game_manager::{self, MatchStatistcs},
    GameState, Player, RuntimeError, SharedState, NUM_PLAYERS,
};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Cli {
    /// Sets a custom config file
    #[clap(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GameConfig {
    pub num_games: u64,
    pub num_simultaneous_games: u64,
}

#[derive(Debug, Deserialize, Clone)]
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

async fn run_match(players: &mut Vec<Box<dyn Player>>) -> Result<MatchStatistcs, RuntimeError> {
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

    let mut players: Vec<PlayerConfig> = vec![app_config.player_one, app_config.player_two];
    if let Some(player) = app_config.player_three {
        players.push(player);
    }

    if let Some(player) = app_config.player_four {
        players.push(player);
    }

    if players.len() != NUM_PLAYERS {
        panic!(
            "Invalid number of players. Expected {}, got {}",
            NUM_PLAYERS,
            players.len()
        );
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
    let game_results: SharedState<Vec<MatchStatistcs>> = SharedState::new(Vec::new());

    let mut handles = Vec::new();
    for _ in 0..app_config.game.num_simultaneous_games {
        let game_queue_clone = game_queue.clone();
        let game_results_clone = game_results.clone();
        let players_clone = players.clone();

        let handle = tokio::spawn(async move {
            loop {
                let mut game_queue_locked = game_queue_clone.lock().await;
                if game_queue_locked.is_empty() {
                    break;
                }
                let next_order = match game_queue_locked.pop() {
                    Some(order) => order,
                    None => break,
                };
                drop(game_queue_locked);

                let mut ordered_clients: Vec<Box<dyn Player>> = Vec::new();
                for &i in &next_order {
                    let mut client = Client::from_path(&players_clone[i - 1].executable);
                    client.set_time(players_clone[i - 1].think_time).await;
                    ordered_clients.push(Box::new(client));
                }

                let stats = run_match(&mut ordered_clients).await;
                let mut stats = match stats {
                    Ok(stats) => stats,
                    Err(e) => {
                        eprintln!("Game ended with an error: {:?}", e);
                        continue;
                    }
                };

                // Reordering player_statistics to match the original order
                let mut reordered_stats: Vec<game_manager::PlayerStatistics> =
                    vec![game_manager::PlayerStatistics::default(); NUM_PLAYERS];
                for (index, &original_index) in next_order.iter().enumerate() {
                    reordered_stats[original_index - 1] = stats.player_statistics[index].clone();
                }
                stats.player_statistics = reordered_stats.try_into().expect("Incorrect length");
                let mut game_results_lock = game_results_clone.lock().await;
                game_results_lock.push(stats);

                // Calculate and print aggregated statistics
                let total_games = game_results_lock.len() as u32;
                let avg_moves = game_results_lock
                    .iter()
                    .map(|stats| stats.executed_moves.len() as u32)
                    .sum::<u32>() as f32
                    / total_games as f32;
                let avg_refills = game_results_lock
                    .iter()
                    .map(|stats| stats.num_factory_refills)
                    .sum::<u32>() as f32
                    / total_games as f32;

                let mut avg_scores = vec![0f32; NUM_PLAYERS];
                let mut wins = [0; NUM_PLAYERS];
                let mut draws = [0; NUM_PLAYERS];
                let mut losses = [0; NUM_PLAYERS];

                for stats in game_results_lock.iter() {
                    for (i, player_stats) in stats.player_statistics.iter().enumerate() {
                        avg_scores[i] += player_stats.final_score as f32;

                        // determine wins, draws, losses
                        // Update wins[i], draws[i], losses[i] accordingly
                        let mut max_score = 0;
                        let mut max_score_count = 0;
                        for (j, other_player_stats) in stats.player_statistics.iter().enumerate() {
                            if i == j {
                                continue;
                            }
                            match other_player_stats.final_score.cmp(&max_score) {
                                std::cmp::Ordering::Greater => {
                                    max_score = other_player_stats.final_score;
                                    max_score_count = 1;
                                }
                                std::cmp::Ordering::Equal => {
                                    max_score_count += 1;
                                }
                                _ => {}
                            }
                        }

                        match player_stats.final_score.cmp(&max_score) {
                            std::cmp::Ordering::Greater => wins[i] += 1,
                            std::cmp::Ordering::Equal => {
                                if max_score_count == 1 {
                                    draws[i] += 1;
                                } else {
                                    losses[i] += 1;
                                }
                            }
                            std::cmp::Ordering::Less => losses[i] += 1,
                        }
                    }
                }

                for score in &mut avg_scores {
                    *score /= total_games as f32;
                }

                println!("Total games: {}", total_games);
                println!("Average executed moves per game: {}", avg_moves);
                println!("Average factory refills per game: {}", avg_refills);
                for i in 0..NUM_PLAYERS {
                    println!(
                        "Player {} - Average score: {}, Wins: {}, Draws: {}, Losses: {}",
                        i + 1,
                        avg_scores[i],
                        wins[i],
                        draws[i],
                        losses[i]
                    );
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.await {
            Ok(_) => println!("Task completed successfully"),
            Err(e) => eprintln!("Game ended with an error: {:?}", e),
        }
    }
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
