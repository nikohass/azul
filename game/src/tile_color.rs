
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TileColor {
    Blue,
    Yellow,
    Red,
    Black,
    White,
}

pub const NUM_TILE_COLORS: usize = 5;