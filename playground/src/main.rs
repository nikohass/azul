use std::collections::HashMap;

use game::{match_::run_match, *};
#[allow(unused_imports)]
use rand::{rngs::SmallRng, Rng, SeedableRng};

// fn perft() {
//     let mut move_list = MoveList::default();
//     if NUM_PLAYERS != 2 {
//         panic!("NUM_PLAYERS must be 2");
//     }
//     loop {
//         let num_runs = 100_000; // Or any number you'd like
//         let mut total_moves = 0;
//         let mut total_duration = std::time::Duration::new(0, 0);

//         let mut rng: SmallRng = SeedableRng::seed_from_u64(0);
//         for _ in 0..num_runs {
//             // println!("Starting new game");
//             let mut game_state = GameState::new(&mut rng);
//             game_state.check_integrity().unwrap();

//             let start_time = std::time::Instant::now();
//             let mut moves_made = 0;
//             loop {
//                 let (is_game_over, _) = game_state.get_possible_moves(&mut move_list, &mut rng);
//                 // println!("{}", game_state);
//                 if is_game_over {
//                     break;
//                 }
//                 // println!("Number of possible moves: {}", move_list.len());
//                 let move_ = move_list[rng.gen_range(0..move_list.len())];
//                 // println!("{}", move_);
//                 game_state.do_move(move_);
//                 moves_made += 1;
//                 // println!("{}", game_state);
//                 game_state.check_integrity().unwrap();
//             }
//             // println!("Done with game");

//             let end_time = std::time::Instant::now();
//             total_moves += moves_made;
//             total_duration += end_time - start_time;
//         }
//         // println!("Done with {} runs", num_runs);

//         let avg_moves_per_second = total_moves as f64 / total_duration.as_secs_f64();

//         //println!("After {} runs:", num_runs);
//         println!("Average moves per second: {:.2}", avg_moves_per_second);

//         static CURRENT_BEST: f64 = 624626.73;
//         //println!("Current best: {:.2}", CURRENT_BEST);
//         println!(
//             "Improvement: {:.2}%",
//             (avg_moves_per_second - CURRENT_BEST) / CURRENT_BEST * 100.0
//         );
//     }
// }

#[allow(unused_imports)]
use player::{mcts::MonteCarloTreeSearch, random_player::RandomPlayer};

// #[tokio::main]
// async fn main() {
//     // perft();
//     let mut rng = SmallRng::from_entropy();
//     let game_state = GameState::new(&mut rng);
//     let mut player_one = MonteCarloTreeSearch::default();
//     let mut player_two = MonteCarloTreeSearch::default();
//     let mut player_three = MonteCarloTreeSearch::default();
//     let mut player_four = MonteCarloTreeSearch::default();

//     player_one.set_time(2000).await;
//     player_two.set_time(2000).await;
//     player_three.set_time(2000).await;
//     player_four.set_time(2000).await;

//     let mut players: Vec<Box<dyn Player>> = vec![
//         Box::new(player_one),
//         Box::new(player_two),
//         Box::new(player_three),
//         Box::new(player_four),
//     ];
//     let _stats = run_match(game_state, &mut players, true).await.unwrap();
//     // println!("{:#?}", stats);
// }

fn hash_factory(factory: &[u8; 5]) -> u16 {
    let mut hash = 0u16; // Ein u16 hat genug Platz für bis zu 16 Bits, also ausreichend für unsere 10 Bits.

    // Gehe durch jedes Element im Array und schiebe das aktuelle Bit um seine Position
    // Multipliziert mit 2, weil jedes Element 2 Bits benötigt
    for (index, &value) in factory.iter().enumerate() {
        hash |= (value as u16) << (index * 2);
    }

    hash
}

fn hash_factories(factories: &mut Factories, factory_lookup: &HashMap<u16, u16>) -> u128 {
    let mut factory_ids: [u16; NUM_FACTORIES] = [0; NUM_FACTORIES];
    for (index, factory) in factories.iter().enumerate().take(NUM_FACTORIES - 1) {
        // Ignore center
        let hash = hash_factory(factory);
        factory_ids[index] = factory_lookup[&hash];
    }

    // Sort the factories to make sure the order doesn't change the final hash
    factory_ids.sort_unstable();

    let mut hash = 0u128;
    for (index, &factory_id) in factory_ids.iter().enumerate() {
        hash |= (factory_id as u128) << (index * 10); // 10 bits reichen aus pro factory. Maximal 10 Factories bei 4 Spielern -> 100 bits
    }

    hash
}

fn find_single_factory_combinations(
    start: usize,                    // Startindex für die Änderung des aktuellen Elements
    sum: u8,                         // Die aktuelle Summe der Elemente im Array
    factory: &mut [u8; 5],           // Das Array, das gefüllt wird
    combinations: &mut Vec<[u8; 5]>, // Speichert die gültigen Kombinationen
) {
    // Wenn die Summe 4 erreicht und wir am Ende des Arrays sind, fügen wir die Kombination hinzu
    if sum == 4 && start == factory.len() {
        combinations.push(*factory);
        return;
    }

    // Wenn wir am Ende des Arrays angekommen sind oder die Summe bereits 4 überschritten hat, brechen wir ab
    if start >= factory.len() || sum > 4 {
        return;
    }

    for i in 0..=4 {
        // Durchläuft die möglichen Werte für das aktuelle Element (0 bis 4)
        factory[start] = i;
        // Rekursiver Aufruf
        find_single_factory_combinations(start + 1, sum + i, factory, combinations);
    }
}

fn main() {
    // print!("{}", bag_to_string(&bag));

    // let mut factory = [TileColor::Red; 4];
    // let mut all_possible_factory_combinations = Vec::new();
    let mut factory = [0; 5]; // Initialisiert das Array mit 0
    let mut combinations = Vec::new(); // Ein Vektor zum Speichern der Kombinationen

    // Startet die Suche nach Kombinationen
    find_single_factory_combinations(0, 0, &mut factory, &mut combinations);

    println!(
        "Number of possible factory combinations: {}",
        combinations.len()
    );
    println!("{:#?}", combinations);

    let factory_lookup: HashMap<u16, u16> = combinations
        .iter()
        .enumerate()
        .map(|(index, factory)| (hash_factory(factory), index as u16))
        .collect();

    for combination in combinations.iter() {
        let hash = hash_factory(combination);
        println!("Hash: {}, Index: {}", hash, factory_lookup[&hash]);
    }

    let mut rng = SmallRng::from_entropy();
    for _ in 0..10 {
        let mut bag = Bag::from([20, 20, 20, 20, 20]);
        let mut factories = Factories::empty();
        let mut out_of_bag = [0; 5];
        factories.refill_by_drawing_from_bag(&mut bag, &mut out_of_bag, &mut rng);
        // print!("{}", bag_to_string(&bag));
        print!("{}", factories_to_string(&factories));

        let hash = hash_factories(&mut factories, &factory_lookup);
        println!("Hash: {}", hash);
    }
}

// use rand::seq::SliceRandom;
// use rand::thread_rng;
// use std::collections::HashMap;

// struct Bag {
//     out_of_bag: Vec<u32>,
//     tiles: Vec<u32>,
// }

// impl Bag {
//     fn new(tile_counts: Vec<u32>, out_of_bag: Vec<u32>) -> Self {
//         let mut tiles = Vec::new();
//         for (i, &count) in tile_counts.iter().enumerate() {
//             tiles.extend(vec![i as u32; count as usize]);
//         }
//         let mut rng = thread_rng();
//         tiles.shuffle(&mut rng);

//         Bag { out_of_bag, tiles }
//     }

//     fn pop(&mut self) -> u32 {
//         if self.tiles.is_empty() {
//             self.tiles = vec![0; 5];
//             // Or any other logic for refilling the bag
//         }
//         self.tiles.pop().unwrap()
//     }
// }

// fn draw_factories(bag: &mut Bag, num_factories: usize) -> Vec<Vec<u32>> {
//     let mut factories = vec![Vec::with_capacity(4); num_factories];
//     for factory in &mut factories {
//         for _ in 0..4 {
//             factory.push(bag.pop());
//         }
//         factory.sort_unstable();
//     }
//     factories.sort_unstable();
//     factories
// }

// fn simulate(num_runs: usize, num_factories: usize) -> Vec<(Vec<Vec<u32>>, u32)> {
//     let mut outcomes = HashMap::new();
//     for _ in 0..num_runs {
//         let mut bag = Bag::new(vec![20; 5], vec![0; 5]);
//         let factory_layout = draw_factories(&mut bag, num_factories);
//         *outcomes.entry(factory_layout).or_insert(0) += 1;
//     }

//     let mut outcome_counts: Vec<_> = outcomes.into_iter().collect();
//     outcome_counts.sort_unstable_by(|a, b| b.1.cmp(&a.1));

//     outcome_counts.into_iter().take(5).collect()
// }

// fn main() {
//     let result = simulate(100000000, 5);
//     for (layout, count) in result {
//         println!("{:?}: {}", layout, count);
//     }
// }
