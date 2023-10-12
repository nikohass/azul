use game::*;


// Given a square index (0-24) and an occupancy board, this will return the count of neighbors.

// fn pick_row(bitboard: u32, pos: u8) -> u32 {
//     let row_index: u8 = pos / 6;
//     println!("row_index: {}", row_index);
//     let row: u32 = bitboard >> (row_index * 6) & 0b11111;
//     row
// }

fn count_row_neighbors(bitboard: u32, new_tile_pos: u8) -> u32 {
    //let row_index: u8 = new_tile_pos / 6;
    //let back_shift = row_index * 6;
    //let mut row: u32 = bitboard >> back_shift & 0b11111;
    let mut row = bitboard;
    //let new_tile: u32 = 1 << (new_tile_pos - back_shift);
    let new_tile = 1 << new_tile_pos;

    let mut neighbors = 0b0;
    let mut bit = new_tile;
    row |= bit;
    while bit & row > 0 {
        neighbors |= bit;
        bit <<= 1;
    }
    bit = new_tile;
    while bit & row > 0 {
        neighbors |= bit;
        bit >>= 1;
    }
    neighbors.count_ones()
}

fn count_column_neighbors(bitboard: u32, new_tile_pos: u8) -> u32 {
    //let column_index: u8 = new_tile_pos % 6;
    // let mut column: u32 = bitboard >> column_index;//) & 0b00_0_00001_0_00001_0_00001_0_00001_0_00001;
    let new_tile: u32 = 1 << new_tile_pos;
    let bitboard = bitboard | new_tile;
    //let column = bitboard >> column_index & 0b00_0_00001_0_00001_0_00001_0_00001_0_00001;
    let column = bitboard;

    let mut neighbors = 0b0;
    let mut bit = new_tile;
    while bit & column > 0 {
        neighbors |= bit;
        bit <<= 6;
    }
    bit = new_tile;
    while bit & column > 0 {
        neighbors |= bit;
        bit >>= 6;
    }

    neighbors.count_ones()
}

fn main() {
    println!("{}", NUM_PLAYERS);
    println!("{}", NUM_FACTORIES);
    let mut bitboard = 0b00_0_00100_0_00000_0_00100_0_01011_0_00100;
    println!("Belegte felder");
    print_bitboard(bitboard);
    let new_tile_pos = 8;

    // board |= new_tile_bit;
    println!("Hier kommt die neue Fliese hin");
    print_bitboard(1 << new_tile_pos);

    // let neighbours = count_row_neighbors(bitboard, new_tile_pos);
    // print_bitboard(neighbours);


    let col = count_column_neighbors(bitboard, new_tile_pos);
    let row = count_row_neighbors(bitboard, new_tile_pos);
    println!("Column neighbors: {}", col);
    println!("Row neighbors: {}", row);
    println!("Total neighbors: {}", col + row);
}