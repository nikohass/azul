/*
        .  0
      . .  1
    . . .  2
  . . . .  3
. . . . .  4
*/

pub const PATTERN_MASKS: [u32; 5] = [
    // 0b1,
    // 0b11_0,
    // 0b111_00_0,
    // 0b1111_000_00_0,
    // 0b11111_0000_000_00_0,
    0b1,
    0b11_0_0,
    0b111_0_00_0_0,
    0b1111_0_000_0_00_0_0,
    0b11111_0_0000_0_000_0_00_0_0,
];

pub fn is_pattern_empty(pattern_bitboard: u32, pattern_index: usize) -> bool {
    pattern_bitboard & PATTERN_MASKS[pattern_index] == 0
}

pub fn is_pattern_full(pattern_bitboard: u32, pattern_index: usize) -> bool {
    pattern_bitboard & PATTERN_MASKS[pattern_index] == PATTERN_MASKS[pattern_index]
}

pub fn remaining_space(pattern_bitboard: u32, pattern_index: usize) -> u32 {
    let remaining_space =
        PATTERN_MASKS[pattern_index] & !(pattern_bitboard & PATTERN_MASKS[pattern_index]);
    remaining_space.count_ones()
}
