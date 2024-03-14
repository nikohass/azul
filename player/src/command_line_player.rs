use game::{GameState, Move, MoveList, Player, TileColor};
use rand::{rngs::SmallRng, SeedableRng};

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
            println!("Enter move with the format {{factory_no}}{{tile_color}}{{pattern_line}}{{pattern_line}}{{pattern_line}}...\n-> ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            let move_ = parse_move(input, &self.move_list);
            if let Some(move_) = move_ {
                return move_;
            }
        }
    }
}

fn parse_move(input: &str, move_list: &MoveList) -> Option<Move> {
    let mut remaining_moves = Vec::new();
    let mut chars = input.chars();
    let factory_index = chars.next()?.to_digit(10)? as u8 - 1;
    for i in 0..move_list.len() {
        if move_list[i].take_from_factory_index == factory_index {
            remaining_moves.push(i);
        }
    }
    if remaining_moves.is_empty() {
        println!("No moves from factory {}", factory_index + 1);
        return None;
    }

    let tile_color = match chars.next()?.to_uppercase().to_string().as_str() {
        "R" => TileColor::Red,
        "G" => TileColor::Green,
        "W" => TileColor::White,
        "B" => TileColor::Blue,
        "Y" => TileColor::Yellow,
        _ => {
            println!("Invalid tile color");
            return None;
        }
    };

    remaining_moves.retain(|move_| move_list[*move_].color == tile_color);
    if remaining_moves.is_empty() {
        println!(
            "No moves with color {} from factory {}",
            tile_color,
            factory_index + 1
        );
        return None;
    }

    let mut pattern_line = [0; 6];
    for character in chars {
        let index = character.to_digit(10)? as usize - 1;
        if index >= 6 {
            println!("Invalid pattern line {}", index + 1);
            return None;
        }
        pattern_line[index] += 1;
    }
    let move_ = remaining_moves
        .iter()
        .find(|move_| move_list[**move_].pattern == pattern_line);
    if move_.is_none() {
        println!(
            "No moves with pattern line {:?} from factory {}",
            pattern_line, factory_index
        );
        return None;
    }

    Some(move_list[*move_.unwrap()])
}
