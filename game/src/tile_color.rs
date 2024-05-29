use serde::{Deserialize, Serialize};

pub const NUM_TILE_COLORS: usize = 5;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileColor {
    Blue,
    Yellow,
    Red,
    Green,
    White,
}

impl std::fmt::Display for TileColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (color_code, char_repr) = match *self {
            TileColor::Blue => ("34", 'B'),   // Blue
            TileColor::Yellow => ("33", 'Y'), // Yellow
            TileColor::Red => ("31", 'R'),    // Red
            TileColor::Green => ("32", 'G'),  // Green (Black)
            TileColor::White => ("97", 'W'),  // White (Bright White)
        };

        write!(f, "\x1b[{}m{}\x1b[0m", color_code, char_repr)
    }
}

impl TileColor {
    pub fn get_color_string(&self) -> (String, String) {
        let color_code = match *self {
            TileColor::Blue => "34",   // Blue
            TileColor::Yellow => "33", // Yellow
            TileColor::Red => "31",    // Red
            TileColor::Green => "32",  // Green (Black)
            TileColor::White => "97",  // White (Bright White)
        };
        let start = format!("\x1b[{}m", color_code);
        let end = "\x1b[0m".to_string();
        (start, end)
    }
}

impl From<TileColor> for char {
    fn from(color: TileColor) -> Self {
        match color {
            TileColor::Blue => 'B',
            TileColor::Yellow => 'Y',
            TileColor::Red => 'R',
            TileColor::Green => 'G',
            TileColor::White => 'W',
        }
    }
}

impl From<char> for TileColor {
    fn from(color: char) -> Self {
        match color {
            'B' => Self::Blue,
            'Y' => Self::Yellow,
            'R' => Self::Red,
            'G' => Self::Green,
            'W' => Self::White,
            _ => panic!("Invalid color: {}", color),
        }
    }
}

impl From<u8> for TileColor {
    fn from(color: u8) -> Self {
        match color {
            0 => Self::Blue,
            1 => Self::Yellow,
            2 => Self::Red,
            3 => Self::Green,
            4 => Self::White,
            _ => panic!("Invalid color: {}", color),
        }
    }
}

impl From<usize> for TileColor {
    fn from(color: usize) -> Self {
        match color {
            0 => Self::Blue,
            1 => Self::Yellow,
            2 => Self::Red,
            3 => Self::Green,
            4 => Self::White,
            _ => panic!("Invalid color: {}", color),
        }
    }
}

impl From<TileColor> for u8 {
    fn from(color: TileColor) -> Self {
        match color {
            TileColor::Blue => 0,
            TileColor::Yellow => 1,
            TileColor::Red => 2,
            TileColor::Green => 3,
            TileColor::White => 4,
        }
    }
}

impl From<TileColor> for usize {
    fn from(color: TileColor) -> Self {
        match color {
            TileColor::Blue => 0,
            TileColor::Yellow => 1,
            TileColor::Red => 2,
            TileColor::Green => 3,
            TileColor::White => 4,
        }
    }
}
