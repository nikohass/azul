use std::collections::HashMap;

use game::{match_::run_match, *};
use player::mcts::MonteCarloTreeSearch;
use rand::{rngs::SmallRng, SeedableRng};

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

#[allow(dead_code)]
fn hash_factory(factory: &[u8; 5]) -> u16 {
    let mut hash = 0u16; // Ein u16 hat genug Platz für bis zu 16 Bits, also ausreichend für unsere 10 Bits.

    // Gehe durch jedes Element im Array und schiebe das aktuelle Bit um seine Position
    // Multipliziert mit 2, weil jedes Element 2 Bits benötigt
    for (index, &value) in factory.iter().enumerate() {
        hash |= (value as u16) << (index * 2);
    }

    hash
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

// fn main() {
//     // print!("{}", bag_to_string(&bag));

//     // let mut factory = [TileColor::Red; 4];
//     // let mut all_possible_factory_combinations = Vec::new();
//     let mut factory = [0; 5]; // Initialisiert das Array mit 0
//     let mut combinations = Vec::new(); // Ein Vektor zum Speichern der Kombinationen

//     // Startet die Suche nach Kombinationen
//     find_single_factory_combinations(0, 0, &mut factory, &mut combinations);

//     println!(
//         "Number of possible factory combinations: {}",
//         combinations.len()
//     );
//     println!("{:#?}", combinations);

//     let factory_lookup: HashMap<u16, u16> = combinations
//         .iter()
//         .enumerate()
//         .map(|(index, factory)| (hash_factory(factory), index as u16))
//         .collect();

//     for combination in combinations.iter() {
//         let hash = hash_factory(combination);
//         println!("Hash: {}, Index: {}", hash, factory_lookup[&hash]);
//     }
// }
#[tokio::main]
async fn main() {
    let mut rng = SmallRng::from_entropy();
    let mut player_one = MonteCarloTreeSearch::default();
    let mut player_two = MonteCarloTreeSearch::default();
    player_one.set_time(3000).await;
    player_two.set_time(3000).await;
    let game_state = GameState::new(&mut rng);

    let mut players: Vec<Box<dyn Player>> = vec![Box::new(player_one), Box::new(player_two)];
    let _stats = run_match(game_state, &mut players, true).await.unwrap();
}
