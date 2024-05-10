use game::{GameState, Move, Player as _};
use player::mcts::MonteCarloTreeSearch as Player;

// use player::random_player::RandomPlayer as Player;
// use player::mcts::HeuristicMoveGenerationPlayer as Player;

#[tokio::main]
async fn main() {
    let mut player = Player::default();

    loop {
        // Get command from stdin
        let mut command = String::new();
        std::io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        let entries = command.split_whitespace().collect::<Vec<_>>();
        let command_type = entries[0];
        match command_type {
            "get_move" => {
                let game_state = GameState::deserialize_string(entries.get(1).unwrap()).unwrap();
                let move_ = player.get_move(&game_state);
                println!("move_response {}", move_.serialize_string());
            }
            "notify_move" => {
                let game_state = GameState::deserialize_string(entries.get(1).unwrap()).unwrap();
                let move_ = Move::deserialize_string(entries.get(2).unwrap());
                player.notify_move(&game_state, move_);
            }
            "reset" => {
                player = Player::default();
            }
            "time" => {
                let time = entries.get(1).unwrap().parse::<u64>().unwrap();
                player.set_time(time);
            }
            "pondering" => {
                let pondering = entries.get(1).unwrap().parse::<bool>().unwrap();
                player.set_pondering(pondering);
            }
            _ => {
                println!("Invalid command, got: {}", command);
                // Ignore to allow backwards compatibility
            }
        }

        // Parse command
    }
}
