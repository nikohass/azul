use super::*;

pub enum Move {
    TakeFactory(u8, TileColor), // Take all tiles of a color from a factory u8: index of factory, TileColor: color of tiles to take
    TakeCenter(TileColor), // Take all tiles of a color from the center. Penalty if the player is the first to do so.
    PlacePattern(u8),      // Place a tile on a row in the Pattern lines. u8: index of row
    PlaceFloor(TileColor),
}
