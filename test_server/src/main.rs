use std::sync::{Arc, Mutex, MutexGuard};

use clap::Parser;
use config::{Config, File, FileFormat};
use rand::{rngs::SmallRng, SeedableRng};
use serde::Deserialize;

mod client;
use client::Client;
use game::{
    match_::{self, MatchStatistcs},
    GameError, GameState, Player, TimeControl, NUM_PLAYERS,
};

use shared::logging::init_logging;

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
    pub verbose: bool,
    pub constant_ordering: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct PlayerConfig {
    pub executable: String,
    pub time_control: TimeControl,
    pub allow_pondering: bool,
}

#[derive(Debug, Deserialize)]
struct AppConfig {
    pub game: GameConfig,
    pub player_one: PlayerConfig,
    pub player_two: PlayerConfig,
    pub player_three: Option<PlayerConfig>,
    pub player_four: Option<PlayerConfig>,
}

fn run_match(players: &mut [Box<dyn Player>], verbose: bool) -> Result<MatchStatistcs, GameError> {
    match_::run_match(
        GameState::new(&mut SmallRng::from_entropy()),
        players,
        verbose,
    )
}

fn main() {
    init_logging("test_server");
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

    log::debug!("{:#?}", app_config);

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

    let player_combinations = if app_config.game.constant_ordering {
        constant_player_ordering(NUM_PLAYERS)
    } else {
        rotating_player_ordering(NUM_PLAYERS)
    };
    // Set num_games to a multiple of the number of player combinations
    let remainder = app_config.game.num_games % player_combinations.len() as u64;
    let num_games = app_config.game.num_games - remainder;

    let mut game_queue: Vec<Vec<usize>> = Vec::new();
    for _ in 0..num_games / player_combinations.len() as u64 {
        game_queue.extend(player_combinations.clone());
    }

    log::info!("Length of game queue: {}", game_queue.len());

    let game_queue: Arc<Mutex<Vec<Vec<usize>>>> = Arc::new(Mutex::new(game_queue));
    let game_results: Arc<Mutex<Vec<MatchStatistcs>>> = Arc::new(Mutex::new(Vec::new()));

    let verbose = app_config.game.verbose;

    let mut handles = Vec::new();
    for _ in 0..app_config.game.num_simultaneous_games {
        let game_queue_clone = game_queue.clone();
        let game_results_clone = game_results.clone();
        let players_clone = players.clone();

        let handle = std::thread::spawn(move || {
            loop {
                let mut game_queue_locked = game_queue_clone.lock().unwrap();
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
                    let mut client = Client::from_path(&players_clone[i - 1].executable, verbose);
                    client.set_time(players_clone[i - 1].time_control.clone());
                    client.set_pondering(players_clone[i - 1].allow_pondering);
                    ordered_clients.push(Box::new(client));
                }

                let stats = run_match(&mut ordered_clients, verbose);
                let mut stats = match stats {
                    Ok(stats) => stats,
                    Err(e) => {
                        log::error!("Game ended with an error: {:?}", e);
                        continue;
                    }
                };

                // Reordering player_statistics to match the original order
                let mut reordered_stats: Vec<match_::PlayerStatistics> =
                    vec![match_::PlayerStatistics::default(); NUM_PLAYERS];
                for (index, &original_index) in next_order.iter().enumerate() {
                    reordered_stats[original_index - 1] = stats.player_statistics[index].clone();
                }
                stats.player_statistics = reordered_stats.try_into().expect("Incorrect length");
                let mut game_results_lock = game_results_clone.lock().unwrap();
                game_results_lock.push(stats);

                print_stats(game_results_lock);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.join() {
            Ok(_) => log::debug!("Task completed successfully"),
            Err(e) => log::error!("Game ended with an error: {:?}", e),
        }
    }
}

fn print_stats(game_results_lock: MutexGuard<Vec<MatchStatistcs>>) {
    // Calculate and print aggregated statistics
    let total_games = game_results_lock.len() as u32;
    let avg_moves = game_results_lock
        .iter()
        .map(|stats| stats.state_action_pairs.len() as u32)
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

    let mut average_branching_factor_per_ply: [f32; 100] = [0.0; 100];
    let mut sum_branching_factor_per_ply: [u32; 100] = [0; 100];
    for stats in game_results_lock.iter() {
        for (i, &branching_factor) in stats.branching_factor.iter().enumerate() {
            average_branching_factor_per_ply[i] += branching_factor as f32;
            sum_branching_factor_per_ply[i] += 1;
        }
    }

    for (i, &sum) in sum_branching_factor_per_ply.iter().enumerate() {
        if sum == 0 {
            average_branching_factor_per_ply[i] = 0.0;
        } else {
            average_branching_factor_per_ply[i] /= sum as f32;
        }
    }

    log::debug!(
        "Average branching factor per ply: {:?}",
        average_branching_factor_per_ply
    );
    log::debug!(
        "Sum branching factor per ply: {:?}",
        sum_branching_factor_per_ply
    );

    log::info!("Total games: {}", total_games);
    log::debug!("Average executed moves per game: {}", avg_moves);
    log::debug!("Average factory refills per game: {}", avg_refills);
    for i in 0..NUM_PLAYERS {
        log::debug!(
            "Player {} - Average score: {}, Wins: {}, Draws: {}, Losses: {}",
            i + 1,
            avg_scores[i],
            wins[i],
            draws[i],
            losses[i]
        );
    }
}

fn rotating_player_ordering(num_players: usize) -> Vec<Vec<usize>> {
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

fn constant_player_ordering(num_players: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();

    for _ in 0..num_players {
        let order = 1..(num_players + 1);
        result.push(order.to_owned().collect());
    }

    result
}
