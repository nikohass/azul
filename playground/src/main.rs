use game::{match_::run_match, *};
#[allow(unused_imports)]
use player::{
    command_line_player::HumanCommandLinePlayer,
    mcts::{HeuristicMoveGenerationPlayer, MonteCarloTreeSearch},
    random_player::RandomPlayer,
};
use rand::{rngs::SmallRng, SeedableRng};

#[tokio::main]
async fn main() {
    let mut rng = SmallRng::from_entropy();

    let mut players: Vec<Box<dyn Player>> = Vec::new();
    for _ in 0..NUM_PLAYERS {
        let mut player = MonteCarloTreeSearch::default();
        player.set_time(10_000).await;
        players.push(Box::new(player));
    }

    let game_state = GameState::new(&mut rng);
    run_match(game_state, &mut players, true).await.unwrap();
}
