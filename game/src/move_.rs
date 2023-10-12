
pub enum Move {
    Factory(u8, TileColor), // Take all tiles of a color from a factory u8: index of factory, TileColor: color of tiles to take
    Center(TileColor), // Take all tiles of a color from the center. Penalty if the player is the first to do so.
    Pattern(u8), // Place a tile on a row in the Pattern lines. u8: index of row 
    Floor(TileColor),
}