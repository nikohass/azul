use crate::factories::CENTER_FACTORY_INDEX;
use crate::tile_color::TileColor;
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub take_from_factory_index: u8,
    pub color: TileColor,
    pub pattern: [u8; 6],
}

impl Move {
    pub const DUMMY: Self = Self {
        take_from_factory_index: 0,
        color: TileColor::Red,
        pattern: [255; 6],
    };

    pub fn serialize_string(&self) -> String {
        let mut result = String::new();
        write!(
            result,
            "{}{}",
            self.take_from_factory_index,
            char::from(self.color)
        )
        .unwrap();

        self.pattern.iter().fold(&mut result, |acc, &x| {
            write!(acc, "{:02}", x).unwrap();
            acc
        });

        result
    }

    pub fn deserialize_string(string: &str) -> Self {
        let mut chars = string.chars();
        let take_from_factory_index = chars.next().unwrap().to_digit(10).unwrap() as u8;
        let color = TileColor::from(chars.next().unwrap());
        let mut pattern = [0; 6];
        let pattern_str: String = chars.collect();
        for (i, chunk) in pattern_str.as_bytes().chunks(2).enumerate() {
            pattern[i] = std::str::from_utf8(chunk).unwrap().parse::<u8>().unwrap();
        }
        Self {
            take_from_factory_index,
            color,
            pattern,
        }
    }

    pub fn is_discard_only(&self) -> bool {
        self.pattern.iter().take(5).all(|&x| x == 0)
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &Self::DUMMY {
            return write!(f, "Dummy Move");
        }
        let factory = if self.take_from_factory_index == CENTER_FACTORY_INDEX as u8 {
            "c".to_string()
        } else {
            format!("{}", self.take_from_factory_index + 1)
        };
        let taken_tile_count = self.pattern.iter().sum::<u8>();
        let discards = self.pattern[5];
        let color_string = self.color.to_string();

        let mut destination_string = String::new();
        for i in 0..5 {
            if self.pattern[i] > 0 {
                destination_string.push_str(&format!("{}@{}", self.pattern[i], i + 1));
                destination_string.push(' ');
            }
        }
        if discards > 0 {
            destination_string.push_str(&format!("D{}", discards));
        }
        let destination_string = destination_string.trim_end();

        write!(
            f,
            "{}{}{}->{}",
            taken_tile_count, color_string, factory, destination_string
        )
    }
}
