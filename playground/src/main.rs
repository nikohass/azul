use game::{match_::run_match, *};
use player::mcts::MonteCarloTreeSearch;
use rand::{rngs::SmallRng, SeedableRng};

#[tokio::main]
async fn main() {
    let mut rng = SmallRng::from_entropy();
    let game_state = GameState::new(&mut rng);
    let mut player_one = MonteCarloTreeSearch::default();
    let mut player_two = MonteCarloTreeSearch::default();
    let mut player_three = MonteCarloTreeSearch::default();
    let mut player_four = MonteCarloTreeSearch::default();

    player_one.set_time(2000).await;
    player_two.set_time(2000).await;
    player_three.set_time(2000).await;
    player_four.set_time(2000).await;

    let mut players: Vec<Box<dyn Player>> = vec![
        Box::new(player_one),
        Box::new(player_two),
        Box::new(player_three),
        Box::new(player_four),
    ];
    let _stats = run_match(game_state, &mut players, true).await.unwrap();
}
