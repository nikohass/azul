use game::{GameState, Player, NUM_PLAYERS};
use player::{
    command_line_player::HumanCommandLinePlayer,
    greedy_player::GreedyPlayer,
    mcts::{HeuristicMoveGenerationPlayer, MonteCarloTreeSearch},
    random_player::RandomPlayer,
};
use rand::{rngs::SmallRng, SeedableRng as _};

async fn configure_players() -> Vec<Box<dyn Player>> {
    let mut players = Vec::new();
    println!("Select player types:");
    println!("1: Human");
    println!("2: Greedy AI");
    println!("3: Random AI");
    println!("4: Heuristic AI");
    println!("5: Monte Carlo Tree Search AI");

    loop {
        let mut string = String::new();
        let mut read_line = || {
            string.clear();
            std::io::stdin().read_line(&mut string).unwrap();
            string.trim().to_string()
        };
        let choice = read_line().parse::<u32>();
        let choice = match choice {
            Ok(choice) => choice,
            Err(_) => continue,
        };
        let player: Box<dyn Player> = match choice {
            1 => Box::<HumanCommandLinePlayer>::default(),
            2 => Box::<GreedyPlayer>::default(),
            3 => Box::<RandomPlayer>::default(),
            4 => Box::<HeuristicMoveGenerationPlayer>::default(),
            5 => {
                let mut mcts = MonteCarloTreeSearch::default();
                println!("Set thinking time for MCTS (ms):");
                let time = read_line().parse::<u64>().unwrap_or(1000);

                mcts.set_time(time).await;
                Box::new(mcts)
            }
            _ => {
                println!("Invalid choice");
                continue;
            }
        };
        players.push(player);
        if players.len() == NUM_PLAYERS {
            break;
        }
    }
    players
}

fn display_rules() {
    println!("Rules:");
}

#[tokio::main]
async fn main() {
    loop {
        println!("1: Start new game");
        println!("2: Display rules");
        println!("3: Exit");

        let read_line = || {
            let mut string = String::new();
            std::io::stdin().read_line(&mut string).unwrap();
            string
        };
        let mut rng = SmallRng::from_entropy();
        match read_line().trim() {
            "1" => {
                let mut players = configure_players().await;
                let game_state = GameState::new(&mut rng);
                if let Err(err) = game::match_::run_match(game_state, &mut players, true).await {
                    println!("Error: {:?}", err);
                }
            }
            "2" => display_rules(),
            "3" => break,
            _ => println!("Invalid option, please try again."),
        }
    }
}
