use game::{
    GameState, Move, MoveList, Player, SharedState, TileColor, NUM_FACTORIES, NUM_PLAYERS,
    NUM_TILE_COLORS,
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::time::Instant;

/*
Average factory refills per game: 4.0235295
Player 1 - Average score: 25.4, Wins: 49, Draws: 0, Losses: 36
Player 2 - Average score: 24.84706, Wins: 35, Draws: 0, Losses: 50
Player 1 made an illegal move: Move { take_from_factory_index: 5, color: Yellow, pattern: [0, 0, 0, 0, 0, 1] }
Move list: [Move { take_from_factory_index: 5, color: Yellow, pattern: [0, 0, 0, 0, 0, 11] }]
Game state: BAG       B 15      Y 5     R 14    G 6     W 13
FACTORIES 1-.... 2-.... 3-.... 4-.... 5-.... Center-YYYYYYYYYYY
PLAYER 0               23 |  PLAYER 1              -18 |
1         B  -> . . R G W |  1         .  -> . Y R G W |
2       B B  -> W . Y R G |  2       . .  -> W . Y R G |
3     G G G  -> . . B . . |  3     G G G  -> . . . . . |
4   . . . .  -> . . W B . |  4   G G G G  -> . . . . Y |
5 . . . . .  -> . R . . . |  5 . . . W W  -> . R . . . |
Floor line:  0            |  Floor line:  0            |
2_1_0_55936156943_0_0-0-0-0-0-2816_64357375_0_36718428-37750622_197121-8657240064_1099495047168-17230462975_0
*/

#[derive(Debug, Clone, Default, Copy)] // TODO: Default just for swapping root
struct ProbabilisticOutcome {
    pub factories: [[u8; NUM_TILE_COLORS]; NUM_FACTORIES],
    pub out_of_bag: [u8; NUM_TILE_COLORS],
    pub bag: [u8; NUM_TILE_COLORS],
}

impl ProbabilisticOutcome {
    pub fn apply_to_game_state(&self, game_state: &mut GameState) {
        // let orig = game_state.clone();
        // Before each factory refill, the last round ends, so evaluate the round
        game_state.evaluate_round();
        // let orig_out_of_bag = game_state.get_out_of_bag();
        // let after_eval = game_state.clone();

        // Overwrite the factories with the outcome of the event
        game_state.set_factories(self.factories);
        // The number of tiles in and out of bag also changes when the factories are refilled, so overwrite those as well
        game_state.set_out_of_bag(self.out_of_bag);
        game_state.set_bag(self.bag);

        // For debugging purposes, check the integrity of the game state
        // if game_state.check_integrity().is_err() {
        //     println!("Original game state:");
        //     println!("{}", orig);
        //     println!("After evaluation:");
        //     println!("{:?}", orig_out_of_bag);
        //     println!("{}", after_eval);
        //     println!("Probabilistic outcome:");
        //     println!("{:?}", game_state.get_out_of_bag());
        //     println!("{}", game_state);

        //     println!("Out of bag would have been {:?}", self.out_of_bag);
        //     panic!("Probabilistic outcome was applied to an invalid game state.");
        // }
    }
}

// fn get_game_result(game_state: &GameState) -> f32 {
//     let scores = game_state.get_scores();
//     let score_difference = scores[0] as f32 - scores[1] as f32;
//     let normalized_score_difference =
//         score_difference.abs() / MAX_SCORE_DIFFERENCE * BASE_SCORE_ADJUSTMENT;
//     match score_difference {
//         x if x > 0. => COMPLEMENTARY_SCORE_ADJUSTMENT + normalized_score_difference, // Basically 1 + normalized_score_difference
//         x if x < 0. => BASE_SCORE_ADJUSTMENT - normalized_score_difference, // Basically 0 - normalized_score_difference
//         _ => 0.5,
//     }
// }

#[derive(Debug, Clone, Copy, Default)]
pub struct Value([f32; NUM_PLAYERS]);

impl Value {
    pub fn from_game_scores(game_scores: [i16; NUM_PLAYERS]) -> Self {
        let max_score = game_scores.iter().cloned().fold(i16::MIN, i16::max);
        let min_score = game_scores.iter().cloned().fold(i16::MAX, i16::min);

        let score_range = max_score - min_score;
        if score_range == 0 {
            // If all scores are the same, return 1 / NUM_PLAYERS for each player
            // e.g. if there are 2 players, return [0.5, 0.5] for each player
            return Self([1.0 / NUM_PLAYERS as f32; NUM_PLAYERS]);
        }

        let mut value = [0.0; NUM_PLAYERS];
        let score_range = score_range as f32;
        for (i, &score) in game_scores.iter().enumerate() {
            let normalized_score = (score - min_score) as f32 / score_range;
            value[i] = normalized_score;
        }

        // // Divide by the sum of all values to normalize them
        // let sum: f32 = value.iter().sum();
        // for value in value.iter_mut() {
        //     *value /= sum;
        // }

        Self(value)
    }
}

// Add two values together
impl std::ops::AddAssign for Value {
    fn add_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.iter()) {
            *lhs += *rhs;
        }
    }
}

impl std::ops::DivAssign<f32> for Value {
    fn div_assign(&mut self, rhs: f32) {
        for value in self.0.iter_mut() {
            *value /= rhs;
        }
    }
}

impl std::ops::Div<f32> for Value {
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self /= rhs;
        self
    }
}

impl std::ops::Index<usize> for Value {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

const C: f32 = 0.2;
const C_BASE: f32 = 6120.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;

#[derive(Debug, Clone)]
enum Event {
    Deterministic(Move),
    Probabilistic(ProbabilisticOutcome),
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Deterministic(move_) => write!(f, "{}", move_),
            Event::Probabilistic(outcome) => {
                let mut string = String::new();
                for (factory_index, factory) in
                    outcome.factories.iter().take(NUM_FACTORIES - 1).enumerate()
                {
                    if factory_index > 0 {
                        string.push(' ');
                    }
                    for (color, number_of_tiles) in factory.iter().enumerate() {
                        string.push_str(
                            &TileColor::from(color)
                                .to_string()
                                .repeat(*number_of_tiles as usize),
                        );
                    }
                }
                write!(f, "{}", string)
            }
        }
    }
}

struct Node {
    children: Vec<Node>,   // The children of this node
    previous_event: Event, // The edge from the parent to this node
    n: f32,
    q: Value, // Value can handle 2 to 4 players
    is_game_over: bool,
    has_probabilistic_children: bool,
}

impl Node {
    pub fn new_deterministic(previous_move: Move) -> Self {
        Node {
            children: Vec::new(),
            previous_event: Event::Deterministic(previous_move),
            n: 0.,
            q: Value::default(),
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }

    pub fn new_probabilistic(outcome: ProbabilisticOutcome) -> Self {
        Node {
            children: Vec::new(),
            previous_event: Event::Probabilistic(outcome),
            n: 0.,
            q: Value::default(),
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }

    #[inline]
    fn get_value(&self) -> Option<Value> {
        if self.n > 0. {
            Some(self.q / self.n)
        } else {
            None
        }
    }

    fn get_uct_value(&self, player_index: usize, parent_n: f32, c: f32) -> f32 {
        if self.n > 0. {
            let mean_value = self.q[player_index] / self.n;
            mean_value + c * (parent_n.ln() / self.n).sqrt()
        } else {
            std::f32::INFINITY
        }
    }

    fn child_with_max_uct_value(&mut self, player_index: usize) -> &mut Node {
        let c_adjusted = C + C_FACTOR * ((1. + self.n + C_BASE) / C_BASE).ln();

        let mut best_child_index = 0;
        let mut best_chuld_uct_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value = child.get_uct_value(player_index, self.n, c_adjusted);
            if value > best_chuld_uct_value {
                best_child_index = i;
                best_chuld_uct_value = value;
            }
        }

        &mut self.children[best_child_index]
    }

    fn backpropagate(&mut self, value: Value) {
        self.n += 1.;
        self.q += value;
    }

    fn expand(&mut self, game_state: &mut GameState, move_list: &mut MoveList, rng: &mut SmallRng) {
        let (is_game_over, probabilistic_event) = game_state.get_possible_moves(move_list, rng);
        self.is_game_over = is_game_over;

        if is_game_over {
            return;
        }

        // Create a child for each possible move
        let mut children: Vec<Node> = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            children.push(Node::new_deterministic(move_list[i]))
        }

        if probabilistic_event {
            // If we have a probabilistic event, create a child for this outcome.
            // We do not expand all possible outcomes, because that would be too expensive.
            let outcome = ProbabilisticOutcome {
                factories: *game_state.get_factories(),
                out_of_bag: game_state.get_out_of_bag(),
                bag: game_state.get_bag(),
            };
            let mut child = Node::new_probabilistic(outcome);
            self.has_probabilistic_children = true;
            // We have already generated the moves for this outcome, so we can just copy them over
            child.children.append(&mut children);
            self.children.push(child);
        } else {
            // Otherwise, just add the children we generated
            std::mem::swap(&mut self.children, &mut children);
        }
    }

    fn iteration(
        &mut self,
        game_state: &mut GameState,
        move_list: &mut MoveList,
        rng: &mut SmallRng,
    ) -> Value {
        let current_player = u8::from(game_state.get_current_player());

        if self.has_probabilistic_children {
            // All children of this node are probabilistic. When this node was "expanded", we only expanded one probabilistic outcome.
            // There would be too many possible outcomes to expand all of them, so we just expanded one.
            // Now we need to adjust for this and dynamically expand the other outcomes.
            // Here we also need to balance exploration and exploitation.
            // If we only visit the only child and never expand further, our strategy will be quite bad because we basically assume that the probabilistic event will always happen.
            // If we expand a new child every time we iterate this node, we would never visit the same child twice. This would cause our estimations of the value of the child to be very inaccurate.

            // Let's just try this:
            let desired_number_of_children = self.n.sqrt().ceil() as usize / 2;
            if desired_number_of_children > self.children.len() {
                // We will expand a new child
                let mut game_state_clone = game_state.clone(); // Clone here because we don't want to modify the game state
                let (_is_game_over, probabilistic_event) =
                    game_state_clone.get_possible_moves(move_list, rng);

                assert!(probabilistic_event); // A probabilistic event must have happened, otherwise we wouldn't have any probabilistic children

                // Create a child for each possible move
                let mut children: Vec<Node> = Vec::with_capacity(move_list.len());
                for i in 0..move_list.len() {
                    children.push(Node::new_deterministic(move_list[i]))
                }

                let outcome = ProbabilisticOutcome {
                    factories: *game_state_clone.get_factories(),
                    out_of_bag: game_state_clone.get_out_of_bag(),
                    bag: game_state_clone.get_bag(),
                };
                let mut child = Node::new_probabilistic(outcome);
                child.children.append(&mut children);
                self.children.push(child);
            }
        }

        let delta: Value = if self.children.is_empty() {
            self.expand(game_state, move_list, rng);
            if !self.is_game_over {
                playout(&mut game_state.clone(), rng, move_list)
            } else if self.n == 0. {
                let game_result = Value::from_game_scores(game_state.get_scores());
                self.q = Value::from_game_scores(game_state.get_scores());
                self.n = 1.;
                game_result
            } else {
                self.q / self.n
            }
        } else {
            let next_child: &mut Node = self.child_with_max_uct_value(current_player as usize);
            match next_child.previous_event {
                Event::Deterministic(move_) => {
                    game_state.do_move(move_);
                }
                Event::Probabilistic(outcome) => {
                    outcome.apply_to_game_state(game_state);
                }
            }
            next_child.iteration(game_state, move_list, rng)
        };

        // let delta = if invert_delta { 1. - delta } else { delta };

        self.backpropagate(delta);

        delta
    }

    fn build_pv(&mut self, game_state: &mut GameState, pv: &mut Vec<Event>) {
        if self.children.is_empty() {
            return;
        }
        let player_index = usize::from(game_state.get_current_player());
        let child: &mut Node = self.best_child(player_index);
        match child.previous_event {
            Event::Deterministic(move_) => {
                game_state.do_move(move_);
            }
            Event::Probabilistic(outcome) => {
                outcome.apply_to_game_state(game_state);
            }
        }
        pv.push(child.previous_event.clone());
        child.build_pv(game_state, pv);
    }

    fn best_child(&mut self, player_index: usize) -> &mut Node {
        let mut best_child_index = 0;
        let mut best_child_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value: Option<Value> = child.get_value();
            let value = match value {
                Some(value) => value,
                None => continue,
            };
            if value[player_index] > best_child_value {
                best_child_index = i;
                best_child_value = value[player_index];
            }
        }

        &mut self.children[best_child_index]
    }

    pub fn best_move(&mut self, player_index: usize) -> Option<Move> {
        if self.children.is_empty() {
            return None;
        }

        let child: &mut Node = self.best_child(player_index);
        match child.previous_event {
            Event::Deterministic(move_) => Some(move_),
            Event::Probabilistic(_) => None,
        }
    }
}

pub fn playout(game_state: &mut GameState, rng: &mut SmallRng, move_list: &mut MoveList) -> Value {
    loop {
        match get_random_move(game_state, rng, move_list) {
            None => {
                return Value::from_game_scores(game_state.get_scores());
            }
            Some(move_) => game_state.do_move(move_),
        }
    }
}

pub fn get_random_move(
    game_state: &mut GameState,
    rng: &mut SmallRng,
    move_list: &mut MoveList,
) -> Option<Move> {
    // Returns (Move, is_game_over)
    // We use the game state to generate one possible random move.
    // In case we fail N times, we fall back to the entire move generation and pick a random move from there.
    let factories: &[[u8; 5]; NUM_FACTORIES] = game_state.get_factories();
    let not_empty_factories: Vec<u8> = factories
        .iter()
        .enumerate()
        .filter(|(_, factory)| factory.iter().any(|&tile| tile != 0))
        .map(|(i, _)| i as u8)
        .collect();
    if !not_empty_factories.is_empty() {
        // There are non-empty factories, we can proceed to pick a random move
        // By doing this kind of move generation we will alter the probabilities of the moves
        // we should make sure that the probabilities are altered in a way that is beneficial to us
        // e.g. we should pick moves that are more likely to be good for us, to make the playout more realistic
        // First, take a look at our pattern lines
        let current_player: usize = usize::from(game_state.get_current_player());
        let pattern_lines: [u8; 5] = game_state.get_pattern_lines_occupancy()[current_player];
        let pattern_line_colors: [Option<game::TileColor>; 5] =
            game_state.get_pattern_lines_colors()[current_player];
        let wall_occupancy: u32 = game_state.get_wall_ocupancy()[current_player];
        for _ in 0..5 {
            // Retry up to 5 times
            // Select a random factory and color to take
            let factory_index =
                not_empty_factories[rng.gen_range(0..not_empty_factories.len())] as usize;
            let mut tile_color = rng.gen_range(0..5);
            while factories[factory_index][tile_color] == 0 {
                tile_color = rng.gen_range(0..5);
            }
            let number_of_tiles_to_distribute = factories[factory_index][tile_color];
            // Calculate the remaining space in our pattern lines for each color
            let mut remaining_space: [u8; 6] = [1, 2, 3, 4, 5, 255]; // 255 is a placeholder for the floor line
            let mut total_remaining_space = 0;
            for (pattern_line_index, number_of_tiles) in pattern_lines.iter().enumerate() {
                remaining_space[pattern_line_index] -= *number_of_tiles; // We subtract the number of tiles already in the pattern line from the total space
                                                                         // If there are tiles of a different color in the patternline already, we can't put any more tiles in it, so we set the remaining space to 0
                if let Some(existing_color) = pattern_line_colors[pattern_line_index] {
                    if tile_color != usize::from(existing_color) {
                        remaining_space[pattern_line_index] = 0;
                    }
                } else {
                    // If the pattern line did not have a color yet, we need to check whether we are allowed to place this color here
                    // It is not possible to place a tile in a pattern line if the corresponding row in the wall is already full
                    let wall_mask = game::wall::WALL_COLOR_MASKS[tile_color];
                    let row_mask = game::wall::get_row_mask(pattern_line_index);
                    if wall_occupancy & row_mask & wall_mask > 0 {
                        remaining_space[pattern_line_index] = 0;
                    }
                }
                if remaining_space[pattern_line_index] == number_of_tiles_to_distribute {
                    // Heuristic: If we find a pattern line that we can fill completely, we will do that
                    let mut pattern = [0; 6];
                    pattern[pattern_line_index] = number_of_tiles_to_distribute;
                    return Some(Move {
                        take_from_factory_index: factory_index as u8,
                        color: game::TileColor::from(tile_color),
                        pattern,
                    });
                }
                total_remaining_space += remaining_space[pattern_line_index];
            }
            if total_remaining_space < number_of_tiles_to_distribute {
                // We do not want to be in this situation, because it means that we will have to put tiles on the floor line
                continue; // Skip this iteration and try again
            }
            // let mut best_pattern_index: Option<usize> = None;
            // let mut best_pattern_fit: u8 = 255; // Arbitrary high number
            // for (index, &space) in remaining_space.iter().enumerate() {
            //     if space >= number_of_tiles_to_distribute && space < best_pattern_fit {
            //         best_pattern_fit = space;
            //         best_pattern_index = Some(index);
            //     }
            // }
            // if let Some(index) = best_pattern_index {
            //     let mut pattern: [u8; 6] = [0; 6];
            //     pattern[index] = number_of_tiles_to_distribute;
            //     return Some(Move {
            //         take_from_factory_index: factory_index as u8,
            //         color: game::TileColor::from(tile_color),
            //         pattern,
            //     });
            // }
            // TODO: Distribute the tiles over the pattern lines in a smart way
        }
        // // We will use this to keep track of the remaining space in our pattern lines for each color
        // let mut remaining_space: [[u8; 6]; game::NUM_TILE_COLORS] = [
        //     [1, 2, 3, 4, 5, 255],
        //     [1, 2, 3, 4, 5, 255],
        //     [1, 2, 3, 4, 5, 255],
        //     [1, 2, 3, 4, 5, 255],
        //     [1, 2, 3, 4, 5, 255],
        // ]; // 255 is a placeholder for the floor line
        // for (tile_color, remaining_space_for_color) in remaining_space.iter_mut().enumerate() {
        //     for (pattern_line_index, number_of_tiles) in pattern_lines.iter().enumerate() {
        //         remaining_space_for_color[pattern_line_index] -= *number_of_tiles; // We subtract the number of tiles already in the pattern line from the total space
        //         // If there are tiles of a different color in the patternline already, we can't put any more tiles in it, so we set the remaining space to 0
        //         if let Some(existing_color) = pattern_line_colors[pattern_line_index] {
        //             if tile_color != usize::from(existing_color) {
        //                 remaining_space_for_color[pattern_line_index] = 0;
        //             }
        //         } else {
        //             // If the pattern line did not have a color yet, we need to check whether we are allowed to place this color here
        //             // It is not possible to place a tile in a pattern line if the corresponding row in the wall is already full
        //             let wall_mask = game::wall::WALL_COLOR_MASKS[tile_color];
        //             let row_mask = game::wall::get_row_mask(pattern_line_index);
        //             if wall_occupancy & row_mask & wall_mask > 0 {
        //                 remaining_space_for_color[pattern_line_index] = 0;
        //             }
        //         }
        //     }
        // }
        // Now we know how many tiles of each color we can place in each pattern line.
        // We will use this information to generate a reasonable move.
        // At this point we basically need a heuristic to select a:
        // - Factory index
        // - Color to take from the factory
        // - Pattern line(s) to place the tiles in
        // Generally it is better to put tiles in the same pattern line and not distribute them over multiple pattern lines
        // because this would block other pattern lines. By placing them in a single pattern line we can complete it faster.
        // Avoiding Negative Points: Try to avoid moves that would result in a significant number of tiles being placed on the floor line.
    }
    // Fallback to default move generation
    let (is_game_over, _) = game_state.get_possible_moves(move_list, rng);
    if is_game_over {
        None
    } else {
        Some(move_list[rng.gen_range(0..move_list.len())])
    }
}

pub struct MonteCarloTreeSearch {
    root_node: SharedState<Node>,
    root_game_state: SharedState<GameState>,
    time_limit: u64,
    stop_pondering: Option<tokio::sync::oneshot::Sender<()>>,
    use_pondering: bool,
}

fn do_iterations(
    root_node: &mut Node,
    root_game_state: &GameState,
    iterations: usize,
    rng: &mut SmallRng,
) {
    let mut move_list = MoveList::new();
    for _ in 0..iterations {
        root_node.iteration(&mut root_game_state.clone(), &mut move_list, rng);
    }
}

impl MonteCarloTreeSearch {
    async fn set_root(&mut self, game_state: &GameState) {
        let mut root_game_state = self.root_game_state.lock().await;
        let mut root_node = self.root_node.lock().await;
        // Check game state for equality. TODO: Implement PartialEq for GameState
        let is_current_player_equal =
            game_state.get_current_player() == root_game_state.get_current_player();
        let is_next_round_starting_player_equal = game_state.get_next_round_starting_player()
            == root_game_state.get_next_round_starting_player();
        let is_pattern_line_equal =
            game_state.get_pattern_lines_colors() == root_game_state.get_pattern_lines_colors();
        let is_pattern_line_equal = is_pattern_line_equal
            && game_state.get_pattern_lines_occupancy()
                == root_game_state.get_pattern_lines_occupancy();
        let is_bag_equal = game_state.get_bag() == root_game_state.get_bag();
        let is_factory_equal = game_state.get_factories() == root_game_state.get_factories();
        let is_discard_equal =
            game_state.get_floor_line_progress() == root_game_state.get_floor_line_progress();
        let states_equal = is_current_player_equal
            && is_next_round_starting_player_equal
            && is_pattern_line_equal
            && is_bag_equal
            && is_factory_equal
            && is_discard_equal;

        if root_node.children.is_empty() || !states_equal {
            println!("Could not find the given game state in the tree. Falling back to the default root node.");
            //root_node = Node::new_deterministic(Move::DUMMY);
            *root_node = Node::new_deterministic(Move::DUMMY);
        } else {
            println!("Found the given game state in the tree. Setting it as the new root node.");
        }
        // self.root_node = Node::new_deterministic(Move::DUMMY);
        *root_game_state = game_state.clone();
    }

    async fn search(&mut self, game_state: &GameState) -> Move {
        println!(
            "Searching move using MCTS. Fen: {}",
            game_state.serialize_string()
        );
        println!("    Left Depth Iterations Value PV");
        let start_time = Instant::now();
        self.set_root(game_state).await;
        let mut rng = SmallRng::from_entropy();
        let mut pv: Vec<Event> = Vec::with_capacity(100);
        let mut iterations_per_ms = 5.;
        let mut completed_iterations: usize = 0;
        let search_start_time = Instant::now();

        let factories: &[[u8; 5]; NUM_FACTORIES] = game_state.get_factories();
        let all_empty = factories
            .iter()
            .all(|factory| factory.iter().all(|&tile| tile == 0));
        if all_empty {
            panic!("Monte Carlo Tree search was started in a position where it is not possible to make a move.");
        }

        let mut root_node = self.root_node.lock().await;
        let root_game_state = self.root_game_state.lock().await;

        loop {
            pv.truncate(0);
            root_node.build_pv(&mut root_game_state.clone(), &mut pv);

            let time_left: i64 = self.time_limit as i64 - start_time.elapsed().as_millis() as i64;

            println!(
                "{:6}ms {:5} {:10} {:?} {}",
                time_left,
                pv.len(),
                completed_iterations,
                root_node.get_value(),
                pv.iter()
                    .map(|event| event.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            if time_left < 30 {
                break;
            }

            let iterations =
                ((time_left as f32 / 6.).min(5000.) * iterations_per_ms).max(1.) as usize;
            do_iterations(&mut root_node, &root_game_state, iterations, &mut rng);
            completed_iterations += iterations;

            let elapsed_time = search_start_time.elapsed().as_micros() as f32 / 1000.;
            if elapsed_time > 0. {
                iterations_per_ms = completed_iterations as f32 / elapsed_time
            }
        }

        println!(
            "Search finished after {}ms. Value: {:?}% PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
            start_time.elapsed().as_millis(),
            root_node.get_value(),
            pv.len(),
            completed_iterations,
            iterations_per_ms * 1000.,
            pv.iter()
                .map(|event| event.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
        let player_index = usize::from(game_state.get_current_player());
        root_node.best_move(player_index).unwrap()
    }

    async fn start_pondering(&mut self) {
        if !self.use_pondering {
            return;
        }
        println!("Pondering...");
        let (stop_sender, mut stop_receiver) = tokio::sync::oneshot::channel();
        let root_node_mutex = self.root_node.clone();
        let root_game_state_mutex = self.root_game_state.clone();

        tokio::spawn(async move {
            let mut rng = SmallRng::from_entropy();
            let start_time = Instant::now();
            let pv: &mut Vec<Event> = &mut Vec::with_capacity(100);
            println!("  Elapsed Depth Iterations Value PV");

            let mut root_node = root_node_mutex.lock().await;
            let root_game_state = root_game_state_mutex.lock().await;

            let mut completed_iterations: usize = 0;
            let mut iterations_per_ms = 5.;

            let mut last_print_time = Instant::now();
            loop {
                if stop_receiver.try_recv().is_ok() {
                    println!(
                        "{:6}ms {:5} {:10} {:?} {}",
                        "",
                        pv.len(),
                        completed_iterations,
                        root_node.get_value(),
                        pv.iter()
                            .map(|event| event.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    println!("Pondering stopped.");
                    break; // Stop if a message has been received
                }

                pv.truncate(0);
                let new_iterations = (iterations_per_ms * 10.) as usize; // Do 10ms worth of iterations
                do_iterations(&mut root_node, &root_game_state, new_iterations, &mut rng);
                completed_iterations += new_iterations;
                root_node.build_pv(&mut root_game_state.clone(), pv);

                let elapsed_time = start_time.elapsed().as_micros() as f32 / 1000.;
                if elapsed_time > 0. && last_print_time.elapsed().as_millis() > 300 {
                    last_print_time = Instant::now();
                    println!(
                        "{:6}ms {:5} {:10} {:?} {}",
                        elapsed_time as i64,
                        pv.len(),
                        completed_iterations,
                        root_node.get_value(),
                        pv.iter()
                            .map(|event| event.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
                if elapsed_time > 0. {
                    iterations_per_ms = completed_iterations as f32 / elapsed_time;
                }
            }
            println!(
                "Pondering finished after {}ms. Value: {:?}% PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
                start_time.elapsed().as_millis(),
                root_node.get_value(),
                pv.len(),
                completed_iterations,
                iterations_per_ms * 1000.,
                pv.iter()
                    .map(|event| event.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        });

        self.stop_pondering = Some(stop_sender);
    }

    async fn stop_pondering(&mut self) {
        if let Some(stop_sender) = self.stop_pondering.take() {
            let result = stop_sender.send(());
            if result.is_err() {
                println!("Failed to stop pondering.");
            }
        }
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self {
            root_node: SharedState::new(Node::new_deterministic(Move::DUMMY)),
            root_game_state: SharedState::new(GameState::new(&mut rng)),
            time_limit: 2000,
            stop_pondering: None,
            use_pondering: false,
        }
    }
}

#[async_trait::async_trait]
impl Player for MonteCarloTreeSearch {
    fn name(&self) -> &str {
        "MCTS"
    }

    async fn get_move(&mut self, game_state: &GameState) -> Move {
        if self.use_pondering {
            println!("Stopping pondering, it's my turn now.");
        }
        self.stop_pondering().await;
        let move_ = self.search(game_state).await;
        self.root_game_state.lock().await.do_move(move_);
        let mut root_node = self.root_node.lock().await;
        let player_index = usize::from(game_state.get_current_player());
        let new_root_node = std::mem::replace(
            root_node.best_child(player_index),
            Node::new_deterministic(Move::DUMMY),
        );
        *root_node = new_root_node;
        drop(root_node);
        if self.use_pondering {
            println!("Turn finished, starting pondering again.");
            self.start_pondering().await;
        }
        move_
    }

    async fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {
        if self.use_pondering {
            println!("Stopping pondering, opponent made a move.");
        }
        self.stop_pondering().await;
        let mut invalid = false;
        let mut root_node = self.root_node.lock().await;
        let mut root_game_state = self.root_game_state.lock().await;
        for child in &mut root_node.children {
            match child.previous_event.clone() {
                Event::Deterministic(child_move) => {
                    if child_move == move_ {
                        let mut temp_node =
                            Node::new_probabilistic(ProbabilisticOutcome::default()); // Assuming a new method for creating a Node
                        std::mem::swap(child, &mut temp_node);
                        let new_root_node = temp_node;
                        *root_node = new_root_node;
                        *root_game_state = new_game_state.clone();
                        return;
                    }
                }
                Event::Probabilistic(_) => {
                    invalid = true;
                    // let mut move_list = MoveList::default();
                    // let mut rng = SmallRng::from_entropy();
                    // // self.root_node = Node::new_deterministic(Move::DUMMY); // Assuming a new method for creating a Node
                    // self.root_game_state.get_possible_moves(&mut move_list, &mut rng);
                    // Get all possible moves to trigger the refilling of the factories
                }
            }
        }
        if invalid {
            *root_node = Node::new_deterministic(Move::DUMMY); // Assuming a new method for creating a Node
        }
        *root_game_state = new_game_state.clone();
        drop(root_node);
        drop(root_game_state);
        if self.use_pondering {
            println!("Turn finished, starting pondering again.");
            self.start_pondering().await;
        }
    }

    async fn set_time(&mut self, time: u64) {
        self.time_limit = time;
    }

    async fn set_pondering(&mut self, pondering: bool) {
        self.use_pondering = pondering;
    }
}
