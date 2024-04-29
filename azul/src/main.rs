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

    loop {
        println!("Select player types:");
        println!("1: Human");
        println!("2: Greedy AI");
        println!("3: Random AI");
        println!("4: Heuristic AI");
        println!("5: Monte Carlo Tree Search AI");

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

        println!("Enter name (leave empty for default):");
        let name = read_line();

        let mut player: Box<dyn Player> = match choice {
            1 => Box::<HumanCommandLinePlayer>::default(),
            2 => Box::<GreedyPlayer>::default(),
            3 => Box::<RandomPlayer>::default(),
            4 => Box::<HeuristicMoveGenerationPlayer>::default(),
            5 => {
                let mut mcts = MonteCarloTreeSearch::default();
                println!("Set thinking time for MCTS (ms):");
                let time = read_line().parse::<u64>().unwrap_or(1000).max(100);

                mcts.set_time(time).await;
                Box::new(mcts)
            }
            _ => {
                println!("Invalid choice");
                continue;
            }
        };

        if !name.is_empty() {
            player.set_name(&name);
        }

        players.push(player);
        if players.len() == NUM_PLAYERS {
            break;
        }
    }
    players
}

fn display_rules() {
    println!("\x1b[1mObjective\x1b[0m");
    println!(
        "The objective of Azul is to score the most points by the end of the game, \
              which occurs when at least one player completes a horizontal line of \
              5 consecutive tiles on their board.\n"
    );

    println!("\x1b[1mSetup\x1b[0m");
    println!(
        "Each player receives a player board, which includes a 5x5 grid on the right \
              side, a pattern lines area on the left side with rows of increasing lengths \
              from 1 to 5, and a floor line below the pattern lines. Factories are set up \
              in the center of the table, and each factory is filled with 4 randomly drawn \
              tiles from a bag. Tiles come in five colors.\n"
    );

    println!("\x1b[1mGameplay\x1b[0m");
    println!(
        "Azul is played over multiple rounds, each consisting of three phases: \
              Factory Offer, Wall-Tiling, and Preparation for the next round.\n"
    );

    println!("\x1b[1m1. Factory Offer Phase\x1b[0m");
    println!("Players take turns choosing tiles from the factories:");
    println!("- \x1b[1mChoosing Tiles:\x1b[0m On your turn, you may choose all tiles of one color from any \
              factory or from the center of the table. Any remaining tiles from the factory \
              are moved to the center.");
    println!(
        "- \x1b[1mPlacing Tiles:\x1b[0m Place the chosen tiles on one row of your pattern lines. \
              If the row already has tiles, you can only place tiles of the same color. If a \
              row is full, additional tiles of that color go to the floor line."
    );
    println!(
        "- \x1b[1mFloor Line:\x1b[0m Tiles placed in the floor line cause point penalties at the \
              end of the round.\n"
    );

    println!("\x1b[1m2. Wall-Tiling Phase\x1b[0m");
    println!("After all tiles have been taken:");
    println!(
        "- \x1b[1mMoving Tiles:\x1b[0m For each of your filled pattern lines, move the rightmost \
              tile to the corresponding space in the grid on the wall. Any excess tiles in \
              that line are discarded."
    );
    println!("- \x1b[1mScoring:\x1b[0m Score points immediately for each tile placed on the wall:");
    println!("  - 1 point for a tile placed with no adjacent tiles.");
    println!(
        "  - Additional points for tiles placed in a contiguous vertical or horizontal line at \
              the tile’s location.\n"
    );

    println!("\x1b[1m3. Preparation for the Next Round\x1b[0m");
    println!(
        "- \x1b[1mRefill Factories:\x1b[0m Return all discarded tiles to the bag and refill the \
              factories for the next round. If the bag is empty, refill it with tiles from \
              the discard pile.\n"
    );

    println!("\x1b[1mScoring\x1b[0m");
    println!(
        "Points are scored during the Wall-Tiling phase as described above, with additional \
              end-of-game bonuses for complete horizontal and vertical lines and sets of \
              colors.\n"
    );

    println!("\x1b[1mEnding the Game\x1b[0m");
    println!(
        "The game ends after the round in which at least one player completes a horizontal \
              line of 5 tiles on their wall. Players tally their final scores, including any \
              end-of-game bonuses. The player with the highest score wins."
    );
    let example_game_state = GameState::new(&mut SmallRng::from_entropy());
    println!("\n\x1b[1mExample Game State\x1b[0m");
    println!("{}", example_game_state);
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
