use game::{GameState, Move, MoveList, Player, TileColor};
use rand::{rngs::SmallRng, SeedableRng};
use std::collections::HashSet;

pub struct HumanCommandLinePlayer {
    move_list: MoveList,
}

impl Default for HumanCommandLinePlayer {
    fn default() -> Self {
        let move_list = MoveList::default();
        Self { move_list }
    }
}

#[async_trait::async_trait]
impl Player for HumanCommandLinePlayer {
    fn get_name(&self) -> &str {
        "Human"
    }

    async fn get_move(&mut self, game_state: &GameState) -> Move {
        let mut game_state = game_state.clone();
        let mut rng = SmallRng::from_entropy();
        game_state.get_possible_moves(&mut self.move_list, &mut rng);

        loop {
            let mut remaining_moves = self.move_list.into_iter().cloned().collect::<Vec<_>>();
            self.prompt_for_factory_number(&mut remaining_moves);
            self.prompt_for_tile_color(&mut remaining_moves);
            self.prompt_for_pattern_line(&mut remaining_moves);

            if remaining_moves.len() == 1 {
                return remaining_moves[0];
            } else {
                println!("There are multiple moves that match your input. Please provide more information.");
            }
        }
    }
}

impl HumanCommandLinePlayer {
    fn prompt_for_factory_number(&self, remaining_moves: &mut Vec<Move>) {
        loop {
            let mut available_factories = HashSet::new();
            for move_ in remaining_moves.iter() {
                available_factories.insert(move_.take_from_factory_index);
            }
            println!("Select a factory to take tiles from:");
            let mut options = available_factories
                .iter()
                .map(|factory| factory + 1)
                .collect::<Vec<_>>();
            options.sort();
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
            remaining_moves.retain(|move_| move_.take_from_factory_index == factory_number - 1);
            break;
        }
    }

    fn prompt_for_tile_color(&self, remaining_moves: &mut Vec<Move>) {
        loop {
            let mut available_colors = HashSet::new();
            for move_ in remaining_moves.iter() {
                available_colors.insert(move_.color);
            }
            let options = available_colors
                .iter()
                .map(|color| color.to_string())
                .collect::<Vec<_>>();

            let color = if options.len() > 1 {
                let mut input = String::new();
                let options = options.join(", ");
                println!("Select a tile color:");
                println!("Options: {}", options);
                if std::io::stdin().read_line(&mut input).is_err() {
                    println!("Failed to read input");
                    continue;
                }
                match input.trim().to_uppercase().as_str() {
                    "R" => TileColor::Red,
                    "G" => TileColor::Green,
                    "W" => TileColor::White,
                    "B" => TileColor::Blue,
                    "Y" => TileColor::Yellow,
                    _ => {
                        println!("Invalid tile color");
                        continue;
                    }
                }
            } else {
                println!("Only one color available: {}", options[0]);
                *available_colors.iter().next().unwrap()
            };

            if !available_colors.contains(&color) {
                println!("Color {} is not available", color);
                continue;
            }

            // Remove all moves with other colors
            remaining_moves.retain(|move_| move_.color == color);
            break;
        }
    }

    fn prompt_for_pattern_line(&self, remaining_moves: &mut Vec<Move>) {
        loop {
            println!("Select pattern line(s) to place tiles on:");
            let mut options = HashSet::new();
            for move_ in remaining_moves.iter() {
                let pattern = move_.pattern;
                for i in 0..6 {
                    if pattern[i] > 0 {
                        options.insert(i);
                    }
                }
            }
            let line_description = ["1st", "2nd", "3rd", "4th", "5th", "floor"];
            for i in 0..6 {
                if options.contains(&i) {
                    println!("{}: {}", i + 1, line_description[i]);
                }
            }
            println!("Enter all pattern lines you want to place tiles on in the format '122d' (for the 1st and 2nd pattern line discarding one):");

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                println!("Failed to read input");
                continue;
            }

            let pattern_line = match parse_pattern_line(&input, remaining_moves) {
                Some(pattern_line) => pattern_line,
                None => continue,
            };

            // Remove all moves with other pattern lines
            remaining_moves.retain(|move_| move_.pattern == pattern_line);
            break;
        }
    }
}

fn parse_pattern_line(input: &str, remaining_moves: &[Move]) -> Option<[u8; 6]> {
    let mut pattern_line = [0; 6];
    for character in input.chars() {
        let index = match character.to_digit(10) {
            Some(index) => index,
            None => {
                continue;
            }
        };
        let index = index as usize - 1;
        if index >= 6 {
            println!("Invalid pattern line {}", index + 1);
            return None;
        }
        pattern_line[index] += 1;
    }

    let move_ = remaining_moves
        .iter()
        .find(|move_| move_.pattern == pattern_line);
    if move_.is_none() {
        println!("No moves with pattern line {:?} from factory", pattern_line);
        return None;
    }

    Some(pattern_line)
}
