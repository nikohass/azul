use game::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};

fn main() {
    loop {
        let num_runs = 100_000; // Or any number you'd like
        let mut total_moves = 0;
        let mut total_duration = std::time::Duration::new(0, 0);

        let mut rng: SmallRng = SeedableRng::seed_from_u64(0);
        for _ in 0..num_runs {
            //game_state.check_integrity();
            let mut game_state = GameState::with_seed(0);

            let start_time = std::time::Instant::now();
            let mut moves_made = 0;

            loop {
                game_state.fill_factories();
                //game_state.check_integrity();

                loop {
                    let possible_moves = game_state.get_possible_moves();
                    if possible_moves.is_empty() {
                        break;
                    }
                    let move_ = possible_moves[rng.gen_range(0..possible_moves.len())];
                    game_state.do_move(move_);
                    moves_made += 1;
                    //game_state.check_integrity();
                }

                let is_game_over = game_state.evaluate_round();
                if is_game_over {
                    break;
                }
            }

            let end_time = std::time::Instant::now();
            total_moves += moves_made;
            total_duration += end_time - start_time;
        }

        let avg_moves_per_second = total_moves as f64 / total_duration.as_secs_f64();

        //println!("After {} runs:", num_runs);
        println!("Average moves per second: {:.2}", avg_moves_per_second);

        static CURRENT_BEST: f64 = 841353.15;
        //println!("Current best: {:.2}", CURRENT_BEST);
        println!(
            "Improvement: {:.2}%",
            (avg_moves_per_second - CURRENT_BEST) / CURRENT_BEST * 100.0
        );
    }
}
//     let mut game_state = GameState::default();
//     //game_state.fill_factories();
//     game_state.check_integrity();

//     let mut rng: SmallRng = SeedableRng::seed_from_u64(0);

//     // println!("{}", game_state);
//     let start_time = std::time::Instant::now();
//     loop {
//         // println!("Round: {}", round);
//         game_state.fill_factories();
//         // println!("{}", game_state);

//         game_state.check_integrity();

//         loop {
//             let possible_moves = game_state.get_possible_moves();
//             if possible_moves.is_empty() {
//                 break;
//             }
//             let move_ = possible_moves[rng.gen_range(0..possible_moves.len())];

//             //println!("Number of possible moves: {}", possible_moves.len());
//             // println!("Selected move: {}", move_);

//             game_state.do_move(move_);
//             // println!("{}", game_state);
//             game_state.check_integrity();
//         }
//         // println!("Round finished");
//         // println!("{}", game_state);
//         let is_game_over = game_state.evaluate_round();
//         // println!("{}", game_state);
//         if is_game_over {
//             // println!("The game ended after round evaluation");
//             break;
//         }
//     }
//     let end_time = std::time::Instant::now();
//     println!("Game finished after {:?}", end_time - start_time);

//     // println!("Possible moves: {:#?}", possible_moves);
//     // let mut mv = Move {
//     //     take: 1,
//     //     color: TileColor::Red,
//     //     pattern: 0b110_0_00_0_0,
//     // };
//     // println!("{}", mv);

//     // game_state.check_integrity();
//     // println!("Fill factories");
//     //game_state.walls[0][0] = 0b00_0_10000_0_01000_0_00100_0_00010_0_00001;

//     // println!("{}", game_state);
//     // game_state.do_move(mv);
//     // println!("{}", game_state);

//     // let board = 0b00_0_11111_0_01000_0_00100_0_00010_0_00001;
//     // // print_32_bit_bitboard(board);
//     // // println!("{}", check_complete_row_exists(board));

//     // print_32_bit_bitboard(PATTERN_MASKS[4]);

//     // println!("{}", is_pattern_full(PATTERN_MASKS[3], 3));

//     // println!("{}", remaining_space(0b0011_0_000_0_00_0_0, 3));
//     // let mut game_state = GameState::default();
//     // game_state.fill_factories();
//     // game_state.walls[0][0] = 0b00_0_10000_0_01000_0_00100_0_00010_0_00001;
//     // game_state.wall_occupancy[0] = 0b00_0_10000_0_01000_0_00100_0_00010_0_00001;
//     // println!("{}", game_state);

//     // let pattern = 0b11111_1111_111_11_1;
//     // let patterns: [u32; 5] = [
//     //     0b1,
//     //     0b11_0,
//     //     0b111_00_0,
//     //     0b1111_000_00_0,
//     //     0b11111_0000_000_00_0,
//     // ];

//     // println!("{}", NUM_PLAYERS);
//     // println!("{}", NUM_FACTORIES);
//     // let mut bitboard = 0b00_0_00100_0_00000_0_00100_0_01011_0_00100;
//     // println!("Belegte felder");
//     // print_bitboard(bitboard);
//     // let new_tile_pos = 8;

//     // // board |= new_tile_bit;
//     // println!("Hier kommt die neue Fliese hin");
//     // print_bitboard(1 << new_tile_pos);

//     // let neighbours = count_row_neighbors(bitboard, new_tile_pos);
//     // print_bitboard(neighbours);

//     // let col = count_column_neighbors(bitboard, new_tile_pos);
//     // let row = count_row_neighbors(bitboard, new_tile_pos);
//     // println!("Column neighbors: {}", col);
//     // println!("Row neighbors: {}", row);
//     // println!("Total neighbors: {}", col + row);
// }
