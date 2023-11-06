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
            "{}{}{} -> {}",
            taken_tile_count, color_string, factory, destination_string
        )
    }
}
