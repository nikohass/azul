use rand::{rngs::SmallRng, SeedableRng};

use crate::{GameState, Move, MoveList, Player, PlayerMarker, RuntimeError, NUM_PLAYERS};

#[derive(Default, Debug, Clone)]
pub struct MatchStatistcs {
    pub num_turns: u32,
    pub num_factory_refills: u32,
    pub executed_moves: Vec<(GameState, PlayerMarker, Move, u64)>,
    pub player_statistics: [PlayerStatistics; NUM_PLAYERS],
}

#[derive(Debug, Clone, Default)]
pub struct PlayerStatistics {
    pub executed_moves: Vec<(GameState, Move, u64)>,
    pub num_moves: u32,
    pub final_score: i16,
}

pub async fn run_match(
    mut game_state: GameState,
    players: &mut Vec<Box<dyn Player>>,
) -> Result<MatchStatistcs, RuntimeError> {
    let num_players = players.len();
    if num_players != NUM_PLAYERS {
        return Err(RuntimeError::PlayerCountMismatch);
    }

    game_state.check_integrity()?;
    let mut stats = MatchStatistcs::default();

    let mut move_list = MoveList::default();
    let mut rng = SmallRng::from_entropy();
    loop {
        println!("{}", game_state);
        let (is_game_over, refilled_factories) =
            game_state.get_possible_moves(&mut move_list, &mut rng);
        if is_game_over {
            break;
        }
        stats.num_factory_refills += refilled_factories as u32;
        stats.num_turns += 1;

        let current_player_marker: PlayerMarker = game_state.get_current_player();
        let current_player = usize::from(current_player_marker);

        let request_time = std::time::Instant::now();
        let players_move: Move = players[current_player].get_move(&game_state).await;
        let response_time = std::time::Instant::now();
        let response_time = response_time.duration_since(request_time).as_millis() as u64;

        if !move_list.contains(players_move) {
            // If the move is not legal, return an error
            println!(
                "Player {} made an illegal move: {:?}",
                current_player, players_move
            );
            println!("Move list: {:?}", move_list);
            println!("Game state: {}", game_state);
            return Err(RuntimeError::IllegalMove);
        }

        stats.executed_moves.push((
            game_state.clone(),
            current_player_marker,
            players_move,
            response_time,
        ));
        stats.player_statistics[current_player]
            .executed_moves
            .push((game_state.clone(), players_move, response_time));
        game_state.do_move(players_move);
        stats.player_statistics[current_player].num_moves += 1;

        for player in players.iter_mut() {
            player.notify_move(&game_state, players_move).await;
        }

        game_state.check_integrity()?;
    }
    println!("{}", game_state);

    // The game is over, we can get the scores
    let scores: [i16; NUM_PLAYERS] = game_state.get_scores();
    for (i, score) in scores.iter().enumerate() {
        stats.player_statistics[i].final_score = *score;
    }

    Ok(stats)
}
