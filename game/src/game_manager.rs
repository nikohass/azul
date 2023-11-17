use crate::{GameState, Move, MoveList, Player, PlayerMarker, RuntimeError, NUM_PLAYERS};

#[derive(Default, Debug, Clone)]
pub struct MatchStatistcs {
    pub num_turns: u32,
    pub num_factory_refills: u32,
    pub num_moves: [u32; NUM_PLAYERS],
    pub final_scores: [i16; NUM_PLAYERS],
    pub executed_moves: Vec<(GameState, PlayerMarker, Move, u64)>,
}

// #[derive(Debug, Clone)]
// pub struct PlayerStatisics {
//     marker: PlayerMarker,
//     executed_moves: Vec<(GameState, Move, u64)>,
// }

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
    loop {
        println!("{}", game_state);
        let (is_game_over, refilled_factories) = game_state.get_possible_moves(&mut move_list);
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
        game_state.do_move(players_move);
        stats.num_moves[current_player] += 1;

        for player in players.iter_mut() {
            player.notify_move(&game_state, players_move).await;
        }

        game_state.check_integrity()?;
    }
    println!("{}", game_state);

    // The game is over, we can get the scores
    let scores: [i16; NUM_PLAYERS] = game_state.get_scores();
    stats.final_scores = scores;

    Ok(stats)
}