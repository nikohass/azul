use game::{GameState, Move, MoveList, Player, TileColor};
use rand::{rngs::SmallRng, SeedableRng};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptResult {
    Continue,      // User has provided valid input
    Reset,         // User wants to start over
    SkipSelection, // Skip this step if there is only one option
}

pub struct HumanCommandLinePlayer {
    name: String,
    move_list: MoveList,
}

impl Default for HumanCommandLinePlayer {
    fn default() -> Self {
        let move_list = MoveList::default();
        Self {
            move_list,
            name: "Human".to_string(),
        }
    }
}

impl Player for HumanCommandLinePlayer {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_move(&mut self, game_state: &GameState) -> Move {
        let mut game_state = game_state.clone();
        let mut rng = SmallRng::from_entropy();
        game_state.get_possible_moves(&mut self.move_list, &mut rng);

        loop {
            let mut remaining_moves = self.move_list.into_iter().cloned().collect::<Vec<_>>();
            let result1 = self.prompt_for_factory_number(&mut remaining_moves);
            if result1 == PromptResult::Reset {
                continue;
            }
            let result2 = self.prompt_for_tile_color(&mut remaining_moves);
            if result2 == PromptResult::Reset {
                continue;
            }
            let result3 = self.prompt_for_pattern_line(&mut remaining_moves);
            if result3 == PromptResult::Reset {
                continue;
            }

            if result1 == PromptResult::SkipSelection
                && result2 == PromptResult::SkipSelection
                && result3 == PromptResult::SkipSelection
            {
                println!("Only one move available: {}", remaining_moves[0]);
                println!("Press enter to submit the move");
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_err() {
                    println!("Failed to read input");
                    continue;
                }
                return remaining_moves[0];
            }

            if remaining_moves.len() == 1 {
                return remaining_moves[0];
            } else {
                println!("There are multiple moves that match your input. Please provide more information.");
            }
        }
    }
}

impl HumanCommandLinePlayer {
    fn prompt_for_factory_number(&self, remaining_moves: &mut Vec<Move>) -> PromptResult {
        loop {
            let mut available_factories = HashSet::new();
            for move_ in remaining_moves.iter() {
                available_factories.insert(move_.factory_index);
            }
            let mut options = available_factories
                .iter()
                .map(|factory| factory + 1)
                .collect::<Vec<_>>();
            options.sort();

            if options.len() == 1 {
                println!("Only one factory available: {}", options[0]);
                return PromptResult::SkipSelection;
            }

            println!("Select a factory to take tiles from:");
            let options = options
                .iter()
                .map(|option| option.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!("Options: {}", options);

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!("Failed to read input");
                continue;
            }
            if input.trim().is_empty() {
                return PromptResult::Reset;
            }

            let factory_number = match input.trim().parse::<u8>() {
                Ok(factory_number) => factory_number,
                Err(_) => {
                    println!("Invalid factory number");
                    continue;
                }
            };

            if !available_factories.contains(&(factory_number - 1)) {
                println!("Factory {} is not available", factory_number);
                continue;
            }

            // Remove all moves from all other factories
            remaining_moves.retain(|move_| move_.factory_index == factory_number - 1);
            return PromptResult::Continue;
        }
    }

    fn prompt_for_tile_color(&self, remaining_moves: &mut Vec<Move>) -> PromptResult {
        loop {
            let mut available_colors = HashSet::new();
            for move_ in remaining_moves.iter() {
                available_colors.insert(move_.color);
            }
            let options = available_colors
                .iter()
                .map(|color| color.to_string())
                .collect::<Vec<_>>();

            if options.len() == 1 {
                println!("Only one color available: {}", options[0]);
                return PromptResult::SkipSelection;
            }

            let mut input = String::new();
            let options = options.join(", ");
            println!("Select a tile color:");
            println!("Options: {}", options);
            if std::io::stdin().read_line(&mut input).is_err() {
                println!("Failed to read input");
                continue;
            }
            if input.trim().is_empty() {
                return PromptResult::Reset;
            }

            let color = match input.trim().to_uppercase().as_str() {
                "R" => TileColor::Red,
                "G" => TileColor::Green,
                "W" => TileColor::White,
                "B" => TileColor::Blue,
                "Y" => TileColor::Yellow,
                _ => {
                    println!("Invalid tile color");
                    continue;
                }
            };

            if !available_colors.contains(&color) {
                println!("Color {} is not available", color);
                continue;
            }

            // Remove all moves with other colors
            remaining_moves.retain(|move_| move_.color == color);
            return PromptResult::Continue;
        }
    }

    fn prompt_for_pattern_line(&self, remaining_moves: &mut Vec<Move>) -> PromptResult {
        loop {
            if remaining_moves.len() == 1 {
                return PromptResult::SkipSelection;
            }
            let mut options = HashSet::new();
            for move_ in remaining_moves.iter() {
                options.insert(move_.pattern_line_index as usize);
            }
            println!("Select pattern line to place tiles on:");
            let line_description = ["1st", "2nd", "3rd", "4th", "5th", "floor"];
            for (i, description) in line_description.iter().enumerate() {
                if options.contains(&i) {
                    println!("{}: {}", i + 1, description);
                }
            }

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!("Failed to read input");
                continue;
            }
            if input.trim().is_empty() {
                return PromptResult::Reset;
            }

            let pattern_line = match input.trim().parse::<u8>() {
                Ok(pattern_line) => pattern_line,
                Err(_) => {
                    println!("Invalid pattern line number");
                    continue;
                }
            };

            // Remove all moves with other pattern lines
            remaining_moves.retain(|move_| move_.pattern_line_index == pattern_line - 1);
            return PromptResult::Continue;
        }
    }
}
