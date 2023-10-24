use game::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};

// Given a square index (0-24) and an occupancy board, this will return the count of neighbors.

// fn pick_row(bitboard: u32, pos: u8) -> u32 {
//     let row_index: u8 = pos / 6;
//     println!("row_index: {}", row_index);
//     let row: u32 = bitboard >> (row_index * 6) & 0b11111;
//     row
// }

// fn count_row_neighbors(bitboard: u32, new_tile_pos: u8) -> u32 {
//     //let row_index: u8 = new_tile_pos / 6;
//     //let back_shift = row_index * 6;
//     //let mut row: u32 = bitboard >> back_shift & 0b11111;
//     let mut row = bitboard;
//     //let new_tile: u32 = 1 << (new_tile_pos - back_shift);
//     let new_tile = 1 << new_tile_pos;

//     let mut neighbors = 0b0;
//     let mut bit = new_tile;
//     row |= bit;
//     while bit & row > 0 {
//         neighbors |= bit;
//         bit <<= 1;
//     }
//     bit = new_tile;
//     while bit & row > 0 {
//         neighbors |= bit;
//         bit >>= 1;
//     }
//     neighbors.count_ones()
// }

// fn count_column_neighbors(bitboard: u32, new_tile_pos: u8) -> u32 {
//     //let column_index: u8 = new_tile_pos % 6;
//     // let mut column: u32 = bitboard >> column_index;//) & 0b00_0_00001_0_00001_0_00001_0_00001_0_00001;
//     let new_tile: u32 = 1 << new_tile_pos;
//     let bitboard = bitboard | new_tile;
//     //let column = bitboard >> column_index & 0b00_0_00001_0_00001_0_00001_0_00001_0_00001;
//     let column = bitboard;

//     let mut neighbors = 0b0;
//     let mut bit = new_tile;
//     while bit & column > 0 {
//         neighbors |= bit;
//         bit <<= 6;
//     }
//     bit = new_tile;
//     while bit & column > 0 {
//         neighbors |= bit;
//         bit >>= 6;
//     }

//     neighbors.count_ones()
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
            // println!("Selected move: {}", move_);

            game_state.do_move(move_);
            // println!("{}", game_state);
            game_state.check_integrity();
        }
        // println!("Round finished");
        // println!("{}", game_state);
        let is_game_over = game_state.evaluate_round();
        // println!("{}", game_state);
        if is_game_over {
            // println!("The game ended after round evaluation");
            break;
        }
    }
    let end_time = std::time::Instant::now();
    println!("Game finished after {:?}", end_time - start_time);

    // println!("Possible moves: {:#?}", possible_moves);
    // let mut mv = Move {
    //     take: 1,
    //     color: TileColor::Red,
    //     pattern: 0b110_0_00_0_0,
    // };
    // println!("{}", mv);

    // game_state.check_integrity();
    // println!("Fill factories");
    //game_state.walls[0][0] = 0b00_0_10000_0_01000_0_00100_0_00010_0_00001;

    // println!("{}", game_state);
    // game_state.do_move(mv);
    // println!("{}", game_state);

    // let board = 0b00_0_11111_0_01000_0_00100_0_00010_0_00001;
    // // print_32_bit_bitboard(board);
    // // println!("{}", check_complete_row_exists(board));

    // print_32_bit_bitboard(PATTERN_MASKS[4]);

    // println!("{}", is_pattern_full(PATTERN_MASKS[3], 3));

    // println!("{}", remaining_space(0b0011_0_000_0_00_0_0, 3));
    // let mut game_state = GameState::default();
    // game_state.fill_factories();
    // game_state.walls[0][0] = 0b00_0_10000_0_01000_0_00100_0_00010_0_00001;
    // game_state.wall_occupancy[0] = 0b00_0_10000_0_01000_0_00100_0_00010_0_00001;
    // println!("{}", game_state);

    // let pattern = 0b11111_1111_111_11_1;
    // let patterns: [u32; 5] = [
    //     0b1,
    //     0b11_0,
    //     0b111_00_0,
    //     0b1111_000_00_0,
    //     0b11111_0000_000_00_0,
    // ];

    // println!("{}", NUM_PLAYERS);
    // println!("{}", NUM_FACTORIES);
    // let mut bitboard = 0b00_0_00100_0_00000_0_00100_0_01011_0_00100;
    // println!("Belegte felder");
    // print_bitboard(bitboard);
    // let new_tile_pos = 8;

    // // board |= new_tile_bit;
    // println!("Hier kommt die neue Fliese hin");
    // print_bitboard(1 << new_tile_pos);

    // let neighbours = count_row_neighbors(bitboard, new_tile_pos);
    // print_bitboard(neighbours);

    // let col = count_column_neighbors(bitboard, new_tile_pos);
    // let row = count_row_neighbors(bitboard, new_tile_pos);
    // println!("Column neighbors: {}", col);
    // println!("Row neighbors: {}", row);
    // println!("Total neighbors: {}", col + row);
}
