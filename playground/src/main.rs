use game::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};

fn perft() {
    let mut move_list = MoveList::default();
    assert!(NUM_PLAYERS == 2);
    loop {
        let num_runs = 100_000; // Or any number you'd like
        let mut total_moves = 0;
        let mut total_duration = std::time::Duration::new(0, 0);

        let mut rng: SmallRng = SeedableRng::seed_from_u64(0);
        for _ in 0..num_runs {
            //game_state.check_integrity();
            // println!("Starting new game");
            let mut game_state = GameState::with_seed(0);

            let start_time = std::time::Instant::now();
            let mut moves_made = 0;

            loop {
                game_state.fill_factories();
                //game_state.check_integrity();

                loop {
                    game_state.get_possible_moves(&mut move_list);
                    if move_list.is_empty() {
                        break;
                    }
                    let move_ = move_list[rng.gen_range(0..move_list.len())];
                    game_state.do_move(move_);
                    moves_made += 1;
                    // game_state.check_integrity();
                }

                let is_game_over = game_state.evaluate_round();
                // println!("Done with round {}", is_game_over);
                // println!("{}", game_state);
                if is_game_over {
                    break;
                }
            }
            // println!("Done with game");

            let end_time = std::time::Instant::now();
            total_moves += moves_made;
            total_duration += end_time - start_time;
            // println!(
            //     "Game finished after {:?} with {} moves",
            //     end_time - start_time,
            //     moves_made
            // );
        }
        // println!("Done with {} runs", num_runs);

        let avg_moves_per_second = total_moves as f64 / total_duration.as_secs_f64();

        //println!("After {} runs:", num_runs);
        println!("Average moves per second: {:.2}", avg_moves_per_second);

        static CURRENT_BEST: f64 = 756983.55;
        //println!("Current best: {:.2}", CURRENT_BEST);
        println!(
            "Improvement: {:.2}%",
            (avg_moves_per_second - CURRENT_BEST) / CURRENT_BEST * 100.0
        );
    }
}

fn main() {
    perft();
    //loop {
    /*let mut game_state = GameState::default();
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
            // std::thread::sleep(std::time::Duration::from_millis(500));
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

    println!("\n\n");
    let string = game_state.serialize_string();
    println!("ORIGINAL:");
    println!("{}", game_state);
    println!("DESERIALIZED:");
    let game_state = GameState::deserialize_string(&string).unwrap();
    println!("{}", game_state);
    println!("INTEGRITY CHECK:");
    // println!("\n\n");

    // let string: String = game_state.serialize_string();
    // println!("{}", string);

    game_state.check_integrity();
    //}*/
}
