use super::layers::InputLayer;
use factories::{CenterFactoryEncoding, NonCenterFactoryEncoding};
use game::{GameState, CENTER_FACTORY_INDEX, NUM_PLAYERS};
use pattern_lines::{LowerPatternLinesEncoding, UpperPatternLinesEncoding};
use score::ScoreEncoding;
use wall::WallEncoding;

pub mod factories;
pub mod pattern_lines;
pub mod score;
pub mod wall;

pub trait OneHotFeature {
    // Is this a feature that each player has their own version of or is it shared?
    const PLAYER_FEATURE: bool;

    // Size of the feature. If PLAYER_FEATURE is true, this is the size of the feature for a single player
    const SIZE: usize;

    // Max num features that can be set to 1 (for each player if PLAYER_FEATURE is true)
    const MAX_ONES: usize;

    const TOTAL_SIZE: usize = feature_size(Self::SIZE, Self::PLAYER_FEATURE);
    const START: usize;
    const END: usize = Self::START + Self::TOTAL_SIZE;

    // fn reset(&mut self) {
    //     *self = Self::default();
    // }

    fn initialize(layer: &mut impl InputLayer) -> Self;
}

const fn feature_size(size: usize, player_feature: bool) -> usize {
    if player_feature {
        size * NUM_PLAYERS
    } else {
        size
    }
}

pub struct Accumulator<L: InputLayer> {
    center_factory_encoding: CenterFactoryEncoding,
    factory_encoding: NonCenterFactoryEncoding,
    upper_pattern_lines_encoding: UpperPatternLinesEncoding,
    lower_pattern_lines_encoding: LowerPatternLinesEncoding,
    score_encoding: ScoreEncoding,
    wall_encoding: WallEncoding,

    layer: L,
}

impl<L: InputLayer> Accumulator<L> {
    pub fn new(mut layer: L) -> Self {
        layer.reset();
        let center_factory_encoding = CenterFactoryEncoding::initialize(&mut layer);
        let factory_encoding = NonCenterFactoryEncoding::initialize(&mut layer);
        let upper_pattern_lines_encoding = UpperPatternLinesEncoding::initialize(&mut layer);
        let lower_pattern_lines_encoding = LowerPatternLinesEncoding::initialize(&mut layer);
        let score_encoding = ScoreEncoding::initialize(&mut layer);
        let wall_encoding = WallEncoding::initialize(&mut layer);

        Self {
            center_factory_encoding,
            factory_encoding,
            upper_pattern_lines_encoding,
            lower_pattern_lines_encoding,
            score_encoding,
            wall_encoding,
            layer,
        }
    }

    pub fn set_game_state(&mut self, game_state: &GameState) {
        self.layer.reset();

        println!("Setting center factory");
        self.center_factory_encoding = CenterFactoryEncoding::initialize(&mut self.layer);
        self.center_factory_encoding
            .set(&game_state.factories[CENTER_FACTORY_INDEX], &mut self.layer);

        println!("Setting non-center factories");
        self.factory_encoding = NonCenterFactoryEncoding::initialize(&mut self.layer);
        self.factory_encoding
            .set_factories(&game_state.factories, &mut self.layer);

        println!("Initializing upper pattern lines");
        self.upper_pattern_lines_encoding = UpperPatternLinesEncoding::initialize(&mut self.layer);
        println!("Initializing lower pattern lines");
        self.lower_pattern_lines_encoding = LowerPatternLinesEncoding::initialize(&mut self.layer);
        println!("Initializing score");
        self.score_encoding = ScoreEncoding::initialize(&mut self.layer);
        println!("Initializing wall");
        self.wall_encoding = WallEncoding::initialize(&mut self.layer);

        for player in 0..NUM_PLAYERS {
            println!("Setting upper pattern lines for player {}", player);
            self.upper_pattern_lines_encoding.set(
                game_state.pattern_lines_occupancy[player],
                game_state.pattern_lines_colors[player],
                player,
                &mut self.layer,
            );
            println!("Setting lower pattern lines for player {}", player);
            self.lower_pattern_lines_encoding.set(
                game_state.pattern_lines_occupancy[player],
                game_state.pattern_lines_colors[player],
                player,
                &mut self.layer,
            );
            println!("Setting score for player {}", player);
            self.score_encoding.set(
                game_state.scores[player],
                game_state.floor_line_progress[player],
                player,
                &mut self.layer,
            );
            println!("Setting wall for player {}", player);
            self.wall_encoding
                .set(game_state.walls[player], player, &mut self.layer);
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::factories::NonCenterFactoryEncoding;
    use super::pattern_lines::UpperPatternLinesEncoding;
    use super::*;
    use game::{MoveGenerationResult, MoveList};
    use pattern_lines::LowerPatternLinesEncoding;
    use rand::rngs::SmallRng;
    use rand::{Rng as _, SeedableRng as _};
    use tests::MockLayer;
    use wall::WallEncoding;

    #[test]
    fn test() {
        /*
        CenterFactoryEncoding:     0 - 50 (50)
        NonCenterFactoryEncoding:  50 - 405 (355)
        UpperPatternLinesEncoding: 405 - 2517 (2112)
        LowerPatternLinesEncoding: 2517 - 3609 (1092)
        ScoreEncoding:             3609 - 3865 (256)
        WallEncoding:              3865 - 8025 (4160)
        */
        println!(
            "CenterFactoryEncoding:     {} - {} ({})",
            CenterFactoryEncoding::START,
            CenterFactoryEncoding::END,
            CenterFactoryEncoding::TOTAL_SIZE,
        );
        println!(
            "NonCenterFactoryEncoding:  {} - {} ({})",
            NonCenterFactoryEncoding::START,
            NonCenterFactoryEncoding::END,
            NonCenterFactoryEncoding::TOTAL_SIZE,
        );
        println!(
            "UpperPatternLinesEncoding: {} - {} ({})",
            UpperPatternLinesEncoding::START,
            UpperPatternLinesEncoding::END,
            UpperPatternLinesEncoding::TOTAL_SIZE,
        );
        println!(
            "LowerPatternLinesEncoding: {} - {} ({})",
            LowerPatternLinesEncoding::START,
            LowerPatternLinesEncoding::END,
            LowerPatternLinesEncoding::TOTAL_SIZE,
        );
        println!(
            "ScoreEncoding:             {} - {} ({})",
            ScoreEncoding::START,
            ScoreEncoding::END,
            ScoreEncoding::TOTAL_SIZE,
        );
        println!(
            "WallEncoding:              {} - {} ({})",
            WallEncoding::START,
            WallEncoding::END,
            WallEncoding::TOTAL_SIZE,
        );

        // let mut layer = MockLayer::default();
        // let mut accumulator = Accumulator::new(layer);
        // let mut game_state = GameState::empty();
        // accumulator.set_game_state(&game_state);
        // println!("{:?}", accumulator.layer.input());
        let mut rng = SmallRng::from_seed([0; 32]);
        let mut accumulator = Accumulator::new(MockLayer::default());
        let mut move_list = MoveList::default();
        for _ in 0..10_000 {
            let mut game_state = GameState::new(&mut rng);

            loop {
                println!("{game_state}");
                accumulator.set_game_state(&game_state);
                if game_state.get_possible_moves(&mut move_list, &mut rng) == MoveGenerationResult::GameOver {
                    break;
                }

                let move_index = rng.gen_range(0..move_list.len());
                let mov = move_list[move_index];
                game_state.do_move(mov);
            }
        }

    }
}

// pub const INPUT_SIZE: usize = ENCODING_SIZE + 8 - (ENCODING_SIZE % 8);
/*
const WALL_OFFSET: usize = LOWER_PATTERN_LINES_ENCODING_SIZE + UPPER_PATTERN_LINES_ENCODING_SIZE;
const SCORE_ENCODING_OFFSET: usize = WALL_OFFSET + WALL_ENCODING_SIZE;

// Number of input features for all non-player related attributes of the game state
pub const COMMON_INPUT_SIZE: usize = FACTORY_ENCODING_SIZE; // TODO: Add the other inputs
                                                            // Number of input features for every single player
pub const PLAYER_INPUT_SIZE: usize =
    WALL_ENCODING_SIZE + UPPER_PATTERN_LINES_ENCODING_SIZE + LOWER_PATTERN_LINES_ENCODING_SIZE + SCORE_ENCODING_SIZE; // TODO: Add the other inputs

pub const TOTAL_ENCODING_SIZE: usize = COMMON_INPUT_SIZE + NUM_PLAYERS * PLAYER_INPUT_SIZE;

#[cfg(not(any(feature = "three_players", feature = "four_players")))]
pub const PLAYER_ENCODING_OFFSET: [usize; NUM_PLAYERS] =
    [COMMON_INPUT_SIZE, COMMON_INPUT_SIZE + PLAYER_INPUT_SIZE];

#[cfg(feature = "three_players")]
pub const PLAYER_ENCODING_OFFSET: [usize; NUM_PLAYERS] = [
    COMMON_INPUT_SIZE,
    COMMON_INPUT_SIZE + PLAYER_INPUT_SIZE,
    COMMON_INPUT_SIZE + 2 * PLAYER_INPUT_SIZE,
];

#[cfg(feature = "four_players")]
pub const PLAYER_ENCODING_OFFSET: [usize; NUM_PLAYERS] = [
    COMMON_INPUT_SIZE,
    COMMON_INPUT_SIZE + PLAYER_INPUT_SIZE,
    COMMON_INPUT_SIZE + 2 * PLAYER_INPUT_SIZE,
    COMMON_INPUT_SIZE + 3 * PLAYER_INPUT_SIZE,
];

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerEncodingState {
    pub upper_pattern_lines: usize,
    pub lower_pattern_lines: usize,
    pub wall_indices: [usize; 3],
    pub score_indices: [usize; 2],
}

pub struct Accumulator<L: InputLayer> {
    player_states: [PlayerEncodingState; NUM_PLAYERS],
    factory_counts: [usize; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
    center_factory_counts: [usize; NUM_TILE_COLORS],
    layer: L,
}

impl<L: InputLayer> Accumulator<L> {
    pub fn new(mut layer: L) -> Self {
        layer.reset();
        let player_states = [PlayerEncodingState::default(); NUM_PLAYERS];

        Self {
            // player_states: [PlayerEncodingState {
            //     upper_pattern_lines: [0; 5],
            //     lower_pattern_lines: [0; 5],
            //     wall_indices: [WALL_OFFSET, WALL_OFFSET + 1024, WALL_OFFSET + 2048],
            // }; NUM_PLAYERS]
            player_states,
            center_factory_counts: [0; NUM_TILE_COLORS],
            factory_counts: [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS],
            layer,
        }
    }

    // pub fn reset(&mut self) {
    //     self.layer.reset();
    //     self.factory_counts = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];

    //     // // Set all factories to empty
    //     // for _ in 0..NUM_NON_CENTER_FACTORIES {
    //     //     self.add_factory(&[0; 5]);
    //     // }

    //     // Set center factory to empty
    //     for tile_color in 0..NUM_TILE_COLORS {
    //         self.set_center_factory(0, tile_color);
    //     }
    // }

    pub fn add_factory(&mut self, factory: &[u8; 5]) {
        let index = factories::add_non_center_factory_index(factory, &mut self.factory_counts);
        self.layer.set_input(index);
    }

    pub fn remove_factory(&mut self, factory: &[u8; 5]) {
        let index = factories::remove_non_center_factory_index(factory, &mut self.factory_counts);
        self.layer.unset_input(index);
    }

    pub fn set_center_factory(&mut self, num_tiles: usize, tile_color: usize, update: bool) {
        let new_indices = factories::set_center_factory_index(
            num_tiles,
            tile_color,
            &mut self.center_factory_counts,
        );
        if let Some((set, unset)) = new_indices {
            self.layer.set_input(set);
            if update {
                self.layer.unset_input(unset);
            }
        }
    }

    pub fn update_upper_pattern_lines(
        &mut self,
        player: usize,
        pattern_lines: [u8; 5],
        colors: [Option<TileColor>; 5],
        update: bool,
    ) {
        let index = pattern_lines::calculate_upper_index(pattern_lines, colors)
            + PLAYER_ENCODING_OFFSET[player];
        if update {
            self.layer
                .unset_input(self.player_states[player].upper_pattern_lines);
        }
        self.layer.set_input(index);
        self.player_states[player].upper_pattern_lines = index;
    }

    pub fn update_lower_pattern_lines(
        &mut self,
        player: usize,
        pattern_lines: [u8; 5],
        colors: [Option<TileColor>; 5],
        update: bool,
    ) {
        let index = pattern_lines::calculate_lower_index(pattern_lines, colors)
            + UPPER_PATTERN_LINES_ENCODING_SIZE
            + PLAYER_ENCODING_OFFSET[player];
        if update {
            self.layer
                .unset_input(self.player_states[player].lower_pattern_lines);
        }
        self.layer.set_input(index);
        self.player_states[player].lower_pattern_lines = index;
    }

    pub fn update_wall(&mut self, player: usize, wall: u32, update: bool) {
        let upper_index =
            wall::calculate_upper_index(wall) + WALL_OFFSET + PLAYER_ENCODING_OFFSET[player];
        let middle_index = wall::calculate_middle_index(wall)
            + WALL_OFFSET
            + 1024
            + PLAYER_ENCODING_OFFSET[player];
        let lower_index =
            wall::calculate_lower_index(wall) + WALL_OFFSET + 2048 + PLAYER_ENCODING_OFFSET[player];

        if update {
            self.layer
                .unset_input(self.player_states[player].wall_indices[0]);
            self.layer
                .unset_input(self.player_states[player].wall_indices[1]);
            self.layer
                .unset_input(self.player_states[player].wall_indices[2]);
        }

        self.layer.set_input(upper_index);
        self.layer.set_input(middle_index);
        self.layer.set_input(lower_index);

        self.player_states[player].wall_indices = [upper_index, middle_index, lower_index];
    }

    pub fn update_score(
        &mut self,
        player: usize,
        score: i16,
        floor_line_progress: u8,
        update: bool,
    ) {
        let (score_index, floor_line_index) =
            score::calculate_score_index(score, floor_line_progress);
        let score_index = score_index + SCORE_ENCODING_OFFSET + PLAYER_ENCODING_OFFSET[player];
        let floor_line_index =
            floor_line_index + SCORE_ENCODING_OFFSET + PLAYER_ENCODING_OFFSET[player];
        let current_score_index = self.player_states[player].score_indices[0];
        let current_floor_line_index = self.player_states[player].score_indices[1];

        if score_index != current_score_index || !update {
            if update {
                self.layer.unset_input(current_score_index);
            }
            self.layer.set_input(score_index);
            self.player_states[player].score_indices[0] = score_index;
        }

        if floor_line_index != current_floor_line_index || !update {
            if update {
                self.layer.unset_input(current_floor_line_index);
            }
            self.layer.set_input(floor_line_index);
            self.player_states[player].score_indices[1] = floor_line_index;
        }
    }

    pub fn set_game_state(&mut self, game_state: &GameState) {
        self.layer.reset();

        self.factory_counts = [0; NUM_POSSIBLE_FACTORY_PERMUTATIONS];
        self.center_factory_counts = [0; NUM_TILE_COLORS];
        // // Set all factories to empty
        // for _ in 0..NUM_NON_CENTER_FACTORIES {
        //     self.add_factory(&[0; 5]);
        // }

        println!("Adding factories");
        for factory in game_state.factories.iter().take(NUM_NON_CENTER_FACTORIES) {
            self.add_factory(factory); // TODO: Factories that are not entirely filled will be encoded incorrectly
        }
        for (tile_color, num_tiles) in game_state.factories[CENTER_FACTORY_INDEX]
            .iter()
            .enumerate()
        {
            self.set_center_factory(*num_tiles as usize, tile_color, false);
        }

        for player in 0..NUM_PLAYERS {
            println!("Adding upper pattern lines for {player}");
            self.update_upper_pattern_lines(
                player,
                game_state.pattern_lines_occupancy[player],
                game_state.pattern_lines_colors[player],
                false,
            );
            println!("Adding lower pattern lines for {player}");
            self.update_lower_pattern_lines(
                player,
                game_state.pattern_lines_occupancy[player],
                game_state.pattern_lines_colors[player],
                false,
            );
            println!("Adding wall for {player}");
            self.update_wall(player, game_state.walls[player], false);
            println!("Adding score for {player}");
            self.update_score(player, game_state.scores[player], game_state.floor_line_progress[player], false);
        }
    }

    pub fn output(&self) -> &[f32] {
        self.layer.output()
    }

    pub fn layer(&self) -> &L {
        &self.layer
    }

    pub fn mut_layer(&mut self) -> &mut L {
        &mut self.layer
    }
}
*/
#[cfg(test)]
mod tests {
    use super::*;
    use game::Factories;
    use rand::{rngs::SmallRng, SeedableRng as _};

    #[derive(Default)]
    pub struct MockLayer {
        pub input: Vec<usize>,
    }

    impl InputLayer for MockLayer {
        fn set_input(&mut self, index: usize) {
            println!("Setting input: {}", index);
            for value in self.input.iter() {
                assert_ne!(value, &index, "Index {} already set", index);
            }
            self.input.push(index);
        }

        fn unset_input(&mut self, index: usize) {
            println!("Unsetting input: {}", index);
            let length_before = self.input.len();
            self.input.retain(|&x| x != index);
            let length_after = self.input.len();
            assert_eq!(length_before as i64 - 1, length_after as i64);
        }

        fn reset(&mut self) {
            self.input.clear();
        }

        fn output(&self) -> &[f32] {
            let mut output = Vec::new();
            for value in self.input.iter() {
                output.push(*value as f32);
            }

            let output = output.into_boxed_slice();
            let output = Box::leak(output);
            output
        }
    }

    impl MockLayer {
        pub fn input(&self) -> &[usize] {
            &self.input
        }
    }
}
/*

    // cargo test --features three_players --package player --lib -- mcts::neural_network::encoding_v2::tests::test_factory_encoding --exact --show-output
    #[test]
    fn test_factory_encoding() {
        let layer = MockLayer { input: vec![] };
        let mut accumulator = Accumulator::new(layer);

        let game_state = GameState::empty();
        accumulator.set_game_state(&game_state);
        println!("{game_state}");

        // All factories should have been set to empty
        let output = accumulator
            .output()
            .iter()
            .map(|x| *x as usize)
            .collect::<Vec<usize>>();
        let required_indices: &[usize] = match NUM_PLAYERS {
            2 => &[70, 141, 212, 283, 354],
            3 => &[70, 141, 212, 283, 354, 425, 496],
            _ => &[70, 141, 212, 283, 354, 425, 496, 567],
        };
        for index in required_indices.iter() {
            assert!(output.contains(index));
        }

        println!("{:?}", output);

        println!("#################");
        let mut game_state = game_state;
        // game_state.scores[0] = 1;
        game_state.floor_line_progress[0] = 1;
        accumulator.set_game_state(&game_state);
        println!("{game_state}");
        let new_output = accumulator
            .output()
            .iter()
            .map(|x| *x as usize)
            .collect::<Vec<usize>>();
        println!("{:?}", new_output);

        // Print the difference
        for index in output.iter() {
            if !new_output.contains(index) {
                println!("Removed: {}", index);
            }
        }
        for index in new_output.iter() {
            if !output.contains(index) {
                println!("Added: {}", index);
            }
        }

    }

    fn reconstruct_factories(mut indices: Vec<usize>) -> Factories {
        indices.retain(|x| x < &FACTORY_ENCODING_SIZE);

        for index in indices {
            let factory_index = index % NUM_POSSIBLE_FACTORY_PERMUTATIONS;
            let factory_count = index / NUM_POSSIBLE_FACTORY_PERMUTATIONS + 1;
            println!(
                "Factory index: {}, Factory count: {}",
                factory_index, factory_count
            );
        }

        Factories::empty()
    }

    #[test]
    fn test_factory_updates() {
        let layer = MockLayer { input: vec![] };
        let mut accumulator = Accumulator::new(layer);

        // let mut rng = SmallRng::from_seed([4; 32]);
        // let game_state: GameState = GameState::new(&mut rng);
        let game_state = GameState::from_fen("2_1_0_73317486350_0_274-274-4369-0-69649-65792_65537000_0_0-0_512-0_1099511563519-1099511627775_0").unwrap();
        println!("{game_state}");
        accumulator.set_game_state(&game_state);

        let indices = accumulator
            .output()
            .iter()
            .map(|x| *x as usize)
            .collect::<Vec<usize>>();
        let factories = reconstruct_factories(indices);
        println!("{:?}", factories);
    }
}
*/
