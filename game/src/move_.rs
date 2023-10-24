use crate::factories::CENTER_FACTORY_INDEX;
use crate::tile_color::TileColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub take_from_factory_index: u8,
    pub color: TileColor,
    pub pattern: [u8; 6],
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let factory = if self.take_from_factory_index == CENTER_FACTORY_INDEX as u8 {
            "center".to_string()
        } else {
            format!("factory {}", self.take_from_factory_index + 1)
        };
        let taken_tile_count = self.pattern.iter().sum::<u8>();
        let discards = self.pattern[5];
        let mut color_string = String::new();
        for _ in 0..taken_tile_count {
            color_string.push_str(&self.color.to_string());
        }
        write!(
            f,
            "Take {} from {} discarding {}",
            color_string, factory, discards
        )
    }
}

// In a game with 2 players, there are 5 factories and 1 center factory
// 6 take * 5 color = up to 30 different combinations of picking colors from factories
// might be less if there are no tiles of a certain color in a factory or if factories are empty / duplicated
