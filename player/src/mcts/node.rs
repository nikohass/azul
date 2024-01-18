use game::{
    GameState, Move, MoveList, Player, SharedState, TileColor, NUM_FACTORIES, NUM_PLAYERS,
    NUM_TILE_COLORS,
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::time::Instant;

use std::sync::{Arc, Mutex};

const C: f32 = 0.2;
const C_BASE: f32 = 20120.0;
const C_FACTOR: f32 = std::f32::consts::SQRT_2;

const MOVE_GENERATION_RETRIES: usize = 5;

#[derive(Debug, Clone, Copy)] // TODO: Default just for swapping root
struct ProbabilisticOutcome {
    pub factories: [[u8; NUM_TILE_COLORS]; NUM_FACTORIES],
    pub out_of_bag: [u8; NUM_TILE_COLORS],
    pub bag: [u8; NUM_TILE_COLORS],
}

impl ProbabilisticOutcome {
    pub fn apply_to_game_state(&self, game_state: &mut GameState) {
        game_state.evaluate_round(); // This will move the tiles from the factories to the pattern lines
        game_state.set_factories(self.factories); // Overwrite the factories with the outcome of the event

        // The number of tiles in and out of bag also changes when the factories are refilled, so overwrite those as well
        game_state.set_out_of_bag(self.out_of_bag);
        game_state.set_bag(self.bag);
    }
}

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

        // Divide by the sum of all values to normalize them
        let sum: f32 = value.iter().sum();
        for value in value.iter_mut() {
            *value /= sum;
        }

        Self(value)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        for (i, &value) in self.0.iter().enumerate() {
            if i > 0 {
                string.push(' ');
            }
            string.push_str(&format!("{:.2}", value));
        }
        write!(f, "{}", string)
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

#[derive(Debug, Clone)]
enum Event {
    Deterministic(Move),
    Probabilistic(ProbabilisticOutcome),
}

impl Event {
    pub fn apply_to_game_state(&self, game_state: &mut GameState) {
        match self {
            Event::Deterministic(move_) => game_state.do_move(*move_),
            Event::Probabilistic(outcome) => outcome.apply_to_game_state(game_state),
        }
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Deterministic(move_) => write!(f, "{}", move_),
            Event::Probabilistic(outcome) => {
                let string = outcome
                    .factories
                    .iter()
                    .take(NUM_FACTORIES - 1)
                    .enumerate()
                    .map(|(_factory_index, factory)| {
                        factory
                            .iter()
                            .enumerate()
                            .map(|(color, number_of_tiles)| {
                                TileColor::from(color)
                                    .to_string()
                                    .repeat(*number_of_tiles as usize)
                            })
                            .collect::<Vec<String>>()
                            .join("")
                    })
                    .collect::<Vec<String>>()
                    .join(" ");

                write!(f, "{}", string)
            }
        }
    }
}

struct Node {
    children: Vec<Arc<Mutex<Node>>>,
    previous_event: Event, // The edge from the parent to this node
    n: f32,
    q: Value,
    is_game_over: bool,
    has_probabilistic_children: bool,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            children: Vec::new(),
            previous_event: Event::Deterministic(Move::DUMMY),
            n: 0.,
            q: Value::default(),
            is_game_over: false,
            has_probabilistic_children: false,
        }
    }
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
    fn get_value(&self) -> Value {
        if self.n > 0. {
            self.q / self.n
        } else {
            Value([std::f32::NEG_INFINITY; NUM_PLAYERS])
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

    fn child_with_max_uct_value(&mut self, player_index: usize) -> Arc<Mutex<Node>> {
        let c_adjusted = C + C_FACTOR * ((1. + self.n + C_BASE) / C_BASE).ln();

        let mut best_child_index = 0;
        let mut best_chuld_uct_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value = child
                .lock()
                .unwrap()
                .get_uct_value(player_index, self.n, c_adjusted);
            if value > best_chuld_uct_value {
                best_child_index = i;
                best_chuld_uct_value = value;
            }
        }

        self.children[best_child_index].clone()
    }

    fn backpropagate(&mut self, value: Value) {
        self.n += 1.;
        self.q += value;
    }

    fn expand(&mut self, game_state: &mut GameState, move_list: &mut MoveList, rng: &mut SmallRng) {
        let (is_game_over, probabilistic_event) = game_state.get_possible_moves(move_list, rng);
        self.is_game_over = is_game_over;

        if is_game_over {
            // If the game is over, we don't need to expand any children
            return;
        }

        // Create children nodes for each possible move
        let mut children = Vec::with_capacity(move_list.len());
        for i in 0..move_list.len() {
            children.push(Arc::new(Mutex::new(Node::new_deterministic(move_list[i]))))
        }

        if probabilistic_event {
            // Create a probabilistic child for the probabilistic event that just happend during the move generation
            // Since it is not possible to expand all possible outcomes of a probabilistic event, we will only expand one of them
            // and dynamically expand the other outcomes later
            self.expand_probabilistic_child(game_state, children);
        } else {
            // Expand the current node with the children we just created
            self.children = children;
        }
    }

    fn expand_probabilistic_child(
        &mut self,
        game_state: &mut GameState,
        children: Vec<Arc<Mutex<Node>>>,
    ) {
        let outcome = ProbabilisticOutcome {
            factories: *game_state.get_factories(),
            out_of_bag: game_state.get_out_of_bag(),
            bag: game_state.get_bag(),
        };
        let mut child = Node::new_probabilistic(outcome);
        child.children = children;
        self.children.push(Arc::new(Mutex::new(child)));
        self.has_probabilistic_children = true;
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
                let mut children: Vec<Arc<Mutex<Node>>> = Vec::with_capacity(move_list.len());
                for i in 0..move_list.len() {
                    children.push(Arc::new(Mutex::new(Node::new_deterministic(move_list[i]))))
                }

                let outcome = ProbabilisticOutcome {
                    factories: *game_state_clone.get_factories(),
                    out_of_bag: game_state_clone.get_out_of_bag(),
                    bag: game_state_clone.get_bag(),
                };
                let mut child = Node::new_probabilistic(outcome);
                child.children.append(&mut children);
                self.children.push(Arc::new(Mutex::new(child)));
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
            let next_child = self.child_with_max_uct_value(current_player as usize);
            let mut next_child = next_child.lock().unwrap();
            next_child.previous_event.apply_to_game_state(game_state);
            next_child.iteration(game_state, move_list, rng)
        };

        self.backpropagate(delta);

        delta
    }

    fn build_pv(&mut self, game_state: &mut GameState, pv: &mut Vec<Event>) {
        if self.children.is_empty() {
            return;
        }

        let player_index = usize::from(game_state.get_current_player());
        let child = self.best_child(player_index);
        let mut child = child.lock().unwrap();

        child.previous_event.apply_to_game_state(game_state);
        pv.push(child.previous_event.clone());

        child.build_pv(game_state, pv);
    }

    fn best_child(&mut self, player_index: usize) -> Arc<Mutex<Node>> {
        let mut best_child_index = 0;
        let mut best_child_value = std::f32::NEG_INFINITY;

        for (i, child) in self.children.iter().enumerate() {
            let value: Value = child.lock().unwrap().get_value();
            if value[player_index] > best_child_value {
                best_child_index = i;
                best_child_value = value[player_index];
            }
        }

        self.children[best_child_index].clone()
    }

    pub fn best_move(&mut self, player_index: usize) -> Option<Move> {
        if self.children.is_empty() {
            return None;
        }

        let child = self.best_child(player_index);
        let child = child.lock().unwrap();
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

#[inline]
fn get_not_empty_factories(factories: &[[u8; 5]; NUM_FACTORIES]) -> Vec<u8> {
    factories
        .iter()
        .enumerate()
        .filter(|(_, factory)| factory.iter().any(|&tile| tile != 0))
        .map(|(i, _)| i as u8)
        .collect()
}

#[inline]
fn get_fallback_move(
    game_state: &mut GameState,
    rng: &mut SmallRng,
    move_list: &mut MoveList,
) -> Option<Move> {
    let (is_game_over, _) = game_state.get_possible_moves(move_list, rng);
    if is_game_over {
        None
    } else {
        Some(move_list[rng.gen_range(0..move_list.len())])
    }
}

#[inline]
fn get_player_wall_data(
    game_state: &GameState,
    player_index: usize,
) -> ([u8; 5], [Option<game::TileColor>; 5], u32) {
    (
        game_state.get_pattern_lines_occupancy()[player_index],
        game_state.get_pattern_lines_colors()[player_index],
        game_state.get_wall_ocupancy()[player_index],
    )
}

fn attempt_move_creation(
    rng: &mut SmallRng,
    factories: &[[u8; 5]; NUM_FACTORIES],
    not_empty_factories: &Vec<u8>,
    pattern_lines: &[u8; 5],
    pattern_line_colors: &[Option<game::TileColor>; 5],
    wall_occupancy: u32,
) -> Option<Move> {
    // Select a random factory and color to take
    let factory_index = not_empty_factories[rng.gen_range(0..not_empty_factories.len())] as usize;
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
        return None; // Skip this iteration and try again
    }

    // TODO: Distribute the tiles over the pattern lines in a smart way

    None
}

#[inline]
pub fn get_random_move(
    game_state: &mut GameState,
    rng: &mut SmallRng,
    move_list: &mut MoveList,
) -> Option<Move> {
    // Returns a random move from the given game state or None if there are no possible moves
    // We use the game state to generate one possible random move.
    // In case we fail MOVE_GENERATION_RETRIES times, we fall back to the entire move generation and pick a random move from there.
    // By doing this kind of move generation we will alter the probabilities of the moves
    // we should make sure that the probabilities are altered in a way that is beneficial to us
    // e.g. we should pick moves that are more likely to be good for us, to make the playout more realistic

    let factories: &[[u8; 5]; NUM_FACTORIES] = game_state.get_factories();
    let not_empty_factories = get_not_empty_factories(factories);
    if not_empty_factories.is_empty() {
        // There are no non-empty factories, we can't generate a move. Fall back to default move generation
        return get_fallback_move(game_state, rng, move_list);
    }

    // There are non-empty factories, we can proceed to pick a random move
    let current_player: usize = usize::from(game_state.get_current_player());
    let (pattern_lines, pattern_line_colors, wall_occupancy) =
        get_player_wall_data(game_state, current_player);

    for _ in 0..MOVE_GENERATION_RETRIES {
        if let Some(move_) = attempt_move_creation(
            rng,
            factories,
            &not_empty_factories,
            &pattern_lines,
            &pattern_line_colors,
            wall_occupancy,
        ) {
            return Some(move_);
        }
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
        let start_time = Instant::now();
        self.set_root(game_state).await;
        let mut rng = SmallRng::from_entropy();
        let mut pv: Vec<Event> = Vec::with_capacity(100);
        let mut iterations_per_ms = 1.; // Initial guess on the lower end for four players, will be adjusted later
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
        println!("    Left Depth Iterations          Value PV");

        loop {
            pv.truncate(0);
            root_node.build_pv(&mut root_game_state.clone(), &mut pv);

            let time_left: i64 = self.time_limit as i64 - start_time.elapsed().as_millis() as i64;

            println!(
                "{:6}ms {:5} {:10} {:7} {}",
                time_left,
                pv.len(),
                completed_iterations,
                root_node.get_value(),
                pv.iter()
                    .map(|event| event.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
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
            "Search finished after {}ms. Value: {:7} PV-Depth: {} Iterations: {} Iterations/s: {:.2} PV: {}",
            start_time.elapsed().as_millis(),
            root_node.get_value(),
            pv.len(),
            completed_iterations,
            iterations_per_ms * 1000.,
            pv.iter()
                .map(|event| event.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        );
        let player_index = usize::from(game_state.get_current_player());
        root_node.best_move(player_index).unwrap()
    }
}

impl Default for MonteCarloTreeSearch {
    fn default() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self {
            root_node: SharedState::new(Node::new_deterministic(Move::DUMMY)),
            root_game_state: SharedState::new(GameState::new(&mut rng)),
            time_limit: 6000,
        }
    }
}

#[async_trait::async_trait]
impl Player for MonteCarloTreeSearch {
    fn name(&self) -> &str {
        "MCTS"
    }

    async fn get_move(&mut self, game_state: &GameState) -> Move {
        let move_ = self.search(game_state).await;
        self.root_game_state.lock().await.do_move(move_);
        let mut root_node = self.root_node.lock().await;
        let player_index = usize::from(game_state.get_current_player());
        println!("Setting child as new root node.");
        let new_root_node = std::mem::replace(
            &mut *root_node.best_child(player_index).lock().unwrap(),
            Node::new_deterministic(Move::DUMMY),
        );
        *root_node = new_root_node;
        println!("New root node set.");
        drop(root_node);
        println!("Move: {}", move_);
        move_
    }

    async fn notify_move(&mut self, new_game_state: &GameState, move_: Move) {
        let mut invalid = false;
        let mut root_node_guard = self.root_node.lock().await;
        let mut root_game_state_guard = self.root_game_state.lock().await;

        // Find the child node that matches the move
        if let Some((i, _)) = root_node_guard
            .children
            .iter()
            .enumerate()
            .find(|(_, child)| {
                matches!(
                    child.lock().unwrap().previous_event,
                    Event::Deterministic(child_move) if child_move == move_
                )
            })
        {
            // Found the matching child, now make it the new root
            println!("Setting child as new root node.");

            // Swap the root node with the matching child node
            let child_arc = root_node_guard.children.remove(i);
            let mut child_node = child_arc.lock().unwrap();

            // Take the data out of the child to avoid cloning the whole subtree
            let new_root_node = std::mem::take(&mut *child_node);

            // Update the root node with the data from the matching child
            *root_node_guard = new_root_node;
            *root_game_state_guard = new_game_state.clone();
            println!("New root node set.");
        } else {
            // No matching child was found, or there was a probabilistic event
            for child in &root_node_guard.children {
                if let Event::Probabilistic(_) = child.lock().unwrap().previous_event {
                    invalid = true;
                    break;
                }
            }

            if invalid {
                // Replace the root node with a new dummy node if an invalid state was encountered
                *root_node_guard = Node::new_deterministic(Move::DUMMY); // Assuming a new method for creating a Node
            }
            *root_game_state_guard = new_game_state.clone();
        }

        // Explicitly drop the guards to release the locks
        drop(root_node_guard);
        drop(root_game_state_guard);
    }

    async fn set_time(&mut self, time: u64) {
        self.time_limit = time;
    }
}
