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
