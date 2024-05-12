#![allow(unused_imports)]

use std::time::Duration;

use game::{match_::run_match, *};
use player::{
    command_line_player::HumanCommandLinePlayer,
    mcts::{HeuristicMoveGenerationPlayer, MonteCarloTreeSearch},
    random_player::RandomPlayer,
};
use rand::{rngs::SmallRng, SeedableRng};

fn main() {
    let mut rng = SmallRng::from_entropy();
    // let game_state = GameState::deserialize_string("2_1_1_30266756869_0_4099-8449-196864-196609-65539-0_65864703_0_1099599-68440990_12901679104-8590065664_50331647-12868452351_0").unwrap();
    // let game_state = GameState::new(&mut rng);
    // let mut players: Vec<Box<dyn Player>> = Vec::new();
    // // players.push(Box::<HumanCommandLinePlayer>::default());
    // for _ in 0..NUM_PLAYERS {
    //     let mut player = MonteCarloTreeSearch::default();
    //     player.set_time(25000);
    //     player.set_pondering(true);
    //     players.push(Box::new(player));
    // }
    // run_match(game_state, &mut players, true).unwrap();
    let mut game_state = GameState::deserialize_string("2_0_0_68955345170_0_65809-4354-0-0-0-8590000384_65537000_1_0-0_33751553-12918456832_1095216858113-16712447_1").unwrap();
    println!("{}", game_state);

    // 1G4->1@1

    let move_ = Move {
        take_from_factory_index: 3,
        color: TileColor::Green,
        pattern: [1, 0, 0, 0, 0, 0],
    };
    println!("{}", move_);

    game_state.do_move(move_);

    let result = game_state.check_integrity().unwrap();

    // Sleep 5s
    std::thread::sleep(Duration::from_secs(50));

    // let _stats = run_match(game_state, &mut players, true).await.unwrap();
    // let scores = stats
    //     .player_statistics
    //     .iter()
    //     .map(|s| s.final_score)
    //     .collect::<Vec<_>>();

    // // let game_state = GameState::deserialize_string("4_0_3_3_16777218_0-0-0-0-0-65584-0-0-0-512_294990263901094930_33554434_140551134-68478855-34203535-70308830_4295164417-8590131712-50397697-17247175168_17163092736-8573289727-1095250412548-17230332927_1").unwrap();
    // // let game_state = GameState::deserialize_string("4_0_3_3_16777216_0-0-0-0-0-65584-0-0-259-8606712320_294990263901094930_33554432_140551134-68478855-34203535-70308830_4295098881-8590066176-50332161-8657240576_17163092736-8573289727-1095266927620-17230332927_1").unwrap();
    // // let game_state = GameState::deserialize_string("4_2_3_3_16777218_0-0-0-0-0-65584-0-0-0-8606712320_294990263901094930_33554434_140551134-68478855-34203535-70308830_4295164417-8590131712-50332161-8657240576_17163092736-8573289727-1095266927620-17230332927_1").unwrap();

    // // let game_state = GameState::deserialize_string("2_1_1_25954879495_30098588674_4609-66064-289-274-135184-0_65864678_0_1081740-8856_12885032960-4345364480_17163157503-17180000255_0").unwrap();
    // // let game_state = GameState::deserialize_string("2_0_1_56086956810_197377_0-65554-8209-0-0-8623554817_65799146_0_132-295172_4311875840-12918456832_12952011263-17196581631_0").unwrap();
    // let game_state = GameState::deserialize_string("2_0_0_64694194190_0_0-0-65569-0-0-8623555072_65537000_0_0-0_4295163905-33554944_17163157251-1095300481279_0").unwrap();
    // let mut mcts = MonteCarloTreeSearch::default();
    // mcts.set_pondering(true).await;
    // mcts.get_move(&game_state).await;
    // // mcts.start_pondering();

    // tokio::time::sleep(Duration::from_secs(10)).await;

    // mcts.stop_pondering();

    // let pv = mcts.get_principal_variation().await;
    // for event in pv.iter() {
    //     println!("{}", event);
    // }

    // mcts.set_time(60000 * 30).await;
    // mcts.get_move(&game_state).await;

    // mcts.store_tree(0.0);
    // // let value = Value::from_game_scores([71_i16, 54_i16, 71_i16, 81_i16]);
    // // println!("{}", value);
}
