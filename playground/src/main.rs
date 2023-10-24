use game::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};

// fn perft() {
//     loop {
//         let num_runs = 100_000; // Or any number you'd like
//         let mut total_moves = 0;
//         let mut total_duration = std::time::Duration::new(0, 0);

//         let mut rng: SmallRng = SeedableRng::seed_from_u64(0);
//         for _ in 0..num_runs {
//             //game_state.check_integrity();
//             let mut game_state = GameState::with_seed(0);

//             let start_time = std::time::Instant::now();
//             let mut moves_made = 0;

//             loop {
//                 game_state.fill_factories();
//                 //game_state.check_integrity();

//                 loop {
//                     let possible_moves = game_state.get_possible_moves();
//                     if possible_moves.is_empty() {
//                         break;
//                     }
//                     let move_ = possible_moves[rng.gen_range(0..possible_moves.len())];
//                     game_state.do_move(move_);
//                     moves_made += 1;
//                     //game_state.check_integrity();
//                 }

//                 let is_game_over = game_state.evaluate_round();
//                 if is_game_over {
//                     break;
//                 }
//             }

//             let end_time = std::time::Instant::now();
//             total_moves += moves_made;
//             total_duration += end_time - start_time;
//         }

//         let avg_moves_per_second = total_moves as f64 / total_duration.as_secs_f64();

//         //println!("After {} runs:", num_runs);
//         println!("Average moves per second: {:.2}", avg_moves_per_second);

//         static CURRENT_BEST: f64 = 841353.15;
//         //println!("Current best: {:.2}", CURRENT_BEST);
//         println!(
//             "Improvement: {:.2}%",
//             (avg_moves_per_second - CURRENT_BEST) / CURRENT_BEST * 100.0
//         );
//     }
// }

fn main() {
    let mut game_state = GameState::default();
    //game_state.fill_factories();
    game_state.check_integrity();

    let mut rng: SmallRng = SeedableRng::seed_from_u64(0);

    // println!("{}", game_state);
    let start_time = std::time::Instant::now();
    loop {
        // println!("Round: {}", round);
        game_state.fill_factories();
        // println!("{}", game_state);

        game_state.check_integrity();

        loop {
            let possible_moves = game_state.get_possible_moves();
            if possible_moves.is_empty() {
                break;
            }
            let move_ = possible_moves[rng.gen_range(0..possible_moves.len())];

            //println!("Number of possible moves: {}", possible_moves.len());
            println!("Selected move: {}", move_);

            game_state.do_move(move_);
            println!("{}", game_state);
            game_state.check_integrity();

            // sleep 0.5 seconds
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        // println!("Round finished");
        // println!("{}", game_state);
        let is_game_over = game_state.evaluate_round();
        // println!("{}", game_state);
        if is_game_over {
            println!("The game ended after round evaluation");
            break;
        }
    }
    let end_time = std::time::Instant::now();
    println!("{}", game_state);
    println!("Game finished after {:?}", end_time - start_time);
}
