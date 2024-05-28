use serde::{Deserialize, Serialize};

use crate::factories::CENTER_FACTORY_INDEX;
use crate::tile_color::TileColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move {
    pub factory_index: u8, // The factory from which the tiles are taken
    pub color: TileColor,  // What color of tiles are taken

    // 0..5 for the factory, 5 for the floor line
    pub pattern_line_index: u8, // Where the tiles are placed

    // Additional information for the move
    pub discards: u8, // How many tiles are discarded
    pub places: u8,   // How many tiles are placed
}

impl Move {
    pub const DUMMY: Self = Self {
        factory_index: 0,
        color: TileColor::Red,
        pattern_line_index: 0,
        discards: 0,
        places: 0,
    };

    pub fn serialize_string(&self) -> String {
        format!(
            "{:1}{:1}{:1}{:02}{:02}",
            self.factory_index,
            usize::from(self.color),
            self.pattern_line_index,
            self.discards,
            self.places,
        )
    }

    pub fn deserialize_string(string: &str) -> Result<Self, String> {
        if string.len() < 7 {
            return Err("String too short".to_owned());
        }

        let factory_index = string[0..1]
            .parse::<u8>()
            .map_err(|_| "Invalid factory index")?;
        let color = TileColor::from(string[1..2].parse::<u8>().map_err(|_| "Invalid color")?);
        let pattern_line_index = string[2..3]
            .parse::<u8>()
            .map_err(|_| "Invalid pattern line index")?;
        let discards = string[3..5].parse::<u8>().map_err(|_| "Invalid discards")?;
        let places = string[5..7].parse::<u8>().map_err(|_| "Invalid places")?;

        Ok(Self {
            factory_index,
            color,
            pattern_line_index,
            discards,
            places,
        })
    }

    pub fn is_discard_only(&self) -> bool {
        self.places == 0
    }

    pub fn tiles_taken(&self) -> u8 {
        self.discards + self.places
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &Self::DUMMY {
            return write!(f, "Dummy Move");
        }
        let factory = if self.factory_index == CENTER_FACTORY_INDEX as u8 {
            "c".to_string()
        } else {
            format!("{}", self.factory_index + 1)
        };

        write!(
            f,
            "{}{}{}->{}{}{}",
            self.discards + self.places,
            self.color,
            factory,
            if self.places > 0 {
                format!("{}@{}", self.places, self.pattern_line_index + 1)
            } else {
                "".to_string()
            },
            if self.places > 0 && self.discards > 0 {
                " "
            } else {
                ""
            },
            if self.discards > 0 {
                format!("D{}", self.discards)
            } else {
                "".to_string()
            }
        )
    }
}
