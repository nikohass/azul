use game::*;
#[allow(unused_imports)]
use rand::{rngs::SmallRng, Rng, SeedableRng};

// fn perft() {
//     let mut move_list = MoveList::default();
//     if NUM_PLAYERS != 2 {
//         panic!("NUM_PLAYERS must be 2");
//     }
//     loop {
//         let num_runs = 100_000; // Or any number you'd like
//         let mut total_moves = 0;
//         let mut total_duration = std::time::Duration::new(0, 0);

//         let mut rng: SmallRng = SeedableRng::seed_from_u64(0);
//         for _ in 0..num_runs {
//             // println!("Starting new game");
//             let mut game_state = GameState::with_seed(0);
//             game_state.check_integrity();

//             let start_time = std::time::Instant::now();
//             let mut moves_made = 0;
//             game_state.fill_factories();
//             loop {
//                 let (is_game_over, _) = game_state.get_possible_moves(&mut move_list);
//                 println!("{}", game_state);
//                 if is_game_over {
//                     break;
//                 }
//                 // println!("Number of possible moves: {}", move_list.len());
//                 let move_ = move_list[rng.gen_range(0..move_list.len())];
//                 println!("{}", move_);
//                 game_state.do_move(move_);
//                 moves_made += 1;
//                 game_state.check_integrity();
//             }
//             // println!("Done with game");

//             let end_time = std::time::Instant::now();
//             total_moves += moves_made;
//             total_duration += end_time - start_time;
//         }
//         // println!("Done with {} runs", num_runs);

//         let avg_moves_per_second = total_moves as f64 / total_duration.as_secs_f64();

//         //println!("After {} runs:", num_runs);
//         println!("Average moves per second: {:.2}", avg_moves_per_second);

//         static CURRENT_BEST: f64 = 624626.73;
//         //println!("Current best: {:.2}", CURRENT_BEST);
//         println!(
//             "Improvement: {:.2}%",
//             (avg_moves_per_second - CURRENT_BEST) / CURRENT_BEST * 100.0
//         );
//     }
// }

#[allow(unused_imports)]
use player::{mcts::node::*, random_player::RandomPlayer};

#[tokio::main]
async fn main() {
    let game_state = GameState::with_seed(0);

    let mut player_one = MonteCarloTreeSearch::default(); //RandomPlayer::new("Random player".to_string());
    let mut player_two = MonteCarloTreeSearch::default();

    player_one.set_time(400).await;
    player_two.set_time(400).await;

    let mut players: Vec<Box<dyn Player>> = vec![Box::new(player_one), Box::new(player_two)];
    let stats = game_manager::run_match(game_state, &mut players)
        .await
        .unwrap();
    println!("{:#?}", stats);
}
