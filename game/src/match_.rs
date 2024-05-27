use rand::{rngs::SmallRng, SeedableRng};

use crate::{
    formatting::display_gamestate, game_state::MoveGenerationResult, GameError, GameState, Move,
    MoveList, Player, PlayerMarker, NUM_PLAYERS,
};

#[derive(Default, Debug, Clone)]
pub struct MatchStatistcs {
    pub num_turns: u32,
    pub num_factory_refills: u32,
    pub state_action_pairs: Vec<(GameState, Move)>,
    pub player_statistics: [PlayerStatistics; NUM_PLAYERS],
    pub branching_factor: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerStatistics {
    pub executed_moves: Vec<(GameState, Move, u64)>,
    pub num_moves: u32,
    pub final_score: i16,
}

pub fn run_match(
    mut game_state: GameState,
    players: &mut [Box<dyn Player>],
    verbose: bool,
) -> Result<MatchStatistcs, GameError> {
    let num_players = players.len();
    if num_players != NUM_PLAYERS {
        return Err(GameError::PlayerCountMismatch);
    }

    let player_names = players
        .iter()
        .map(|player| player.get_name().to_string())
        .collect::<Vec<_>>();

    game_state.check_integrity()?;
    for player in players.iter_mut() {
        player.notify_factories_refilled(&game_state);
    }
    let mut stats = MatchStatistcs::default();

    let mut move_list = MoveList::default();
    let mut rng = SmallRng::from_entropy();
    loop {
        if verbose {
            println!("{}", display_gamestate(&game_state, Some(&player_names)));
            println!("{}", game_state.to_fen());
        }
        let result = game_state.get_possible_moves(&mut move_list, &mut rng);
        let is_game_over = matches!(result, MoveGenerationResult::GameOver);
        let refilled_factories = matches!(result, MoveGenerationResult::RoundOver);
        if is_game_over {
            break;
        }
        if refilled_factories && verbose {
            println!("Factories refilled");
            println!("{}", display_gamestate(&game_state, Some(&player_names)));
        }
        if refilled_factories {
            for player in players.iter_mut() {
                player.notify_factories_refilled(&game_state);
            }
        }

        stats.num_factory_refills += refilled_factories as u32;
        stats.num_turns += 1;

        let current_player_marker: PlayerMarker = game_state.current_player;
        let current_player = usize::from(current_player_marker);

        let request_time = std::time::Instant::now();
        let players_move: Move = players[current_player].get_move(&game_state);
        if verbose {
            println!(
                "{}: {} {:?}",
                player_names[current_player], players_move, players_move
            );
        }
        let response_time = std::time::Instant::now();
        let response_time = response_time.duration_since(request_time).as_millis() as u64;

        if !move_list.contains(players_move) {
            // If the move is not legal, return an error
            println!(
                "Player {} made an illegal move: {:?}",
                current_player, players_move
            );
            println!("Move list: {:?}", move_list);
            println!("{}", display_gamestate(&game_state, Some(&player_names)));
            println!("{}", game_state.to_fen());
            return Err(GameError::IllegalMove);
        }

        stats.branching_factor.push(move_list.len() as u32);
        stats
            .state_action_pairs
            .push((game_state.clone(), players_move));
        stats.player_statistics[current_player]
            .executed_moves
            .push((game_state.clone(), players_move, response_time));
        game_state.do_move(players_move);
        stats.player_statistics[current_player].num_moves += 1;

        for player in players.iter_mut() {
            player.notify_move(&game_state, players_move);
        }

        game_state.check_integrity()?;
    }
    if verbose {
        println!("{}", display_gamestate(&game_state, Some(&player_names)));
        println!("{}", game_state.to_fen());
    }

    // The game is over, we can get the scores
    let scores: [i16; NUM_PLAYERS] = game_state.scores;
    for (i, score) in scores.iter().enumerate() {
        stats.player_statistics[i].final_score = *score;
    }

    // Reset the players
    for player in players.iter_mut() {
        player.notify_game_over(&game_state);
        player.reset();
    }

    Ok(stats)
}
