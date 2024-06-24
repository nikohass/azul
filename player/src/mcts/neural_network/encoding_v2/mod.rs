use super::layers::InputLayer;
use bag::BagEncoding;
use factories::{CenterFactoryEncoding, NonCenterFactoryEncoding};
use game::{GameState, CENTER_FACTORY_INDEX, NUM_PLAYERS};
use pattern_lines::{LowerPatternLinesEncoding, UpperPatternLinesEncoding};
use score::ScoreEncoding;
use wall::WallEncoding;

pub mod bag;
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
    bag_encoding: BagEncoding,

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
        let bag_encoding = BagEncoding::initialize(&mut layer);
        Self {
            center_factory_encoding,
            factory_encoding,
            upper_pattern_lines_encoding,
            lower_pattern_lines_encoding,
            score_encoding,
            wall_encoding,
            bag_encoding,
            layer,
        }
    }

    pub fn set_game_state(&mut self, game_state: &GameState) {
        self.layer.reset();

        self.center_factory_encoding = CenterFactoryEncoding::initialize(&mut self.layer);
        self.center_factory_encoding
            .set(&game_state.factories[CENTER_FACTORY_INDEX], &mut self.layer);

        self.factory_encoding = NonCenterFactoryEncoding::initialize(&mut self.layer);
        self.factory_encoding
            .set_factories(&game_state.factories, &mut self.layer);

        self.upper_pattern_lines_encoding = UpperPatternLinesEncoding::initialize(&mut self.layer);
        self.lower_pattern_lines_encoding = LowerPatternLinesEncoding::initialize(&mut self.layer);
        self.score_encoding = ScoreEncoding::initialize(&mut self.layer);
        self.wall_encoding = WallEncoding::initialize(&mut self.layer);
        self.bag_encoding = BagEncoding::initialize(&mut self.layer);

        for player in 0..NUM_PLAYERS {
            self.upper_pattern_lines_encoding.set(
                game_state.pattern_lines_occupancy[player],
                game_state.pattern_lines_colors[player],
                player,
                &mut self.layer,
            );
            self.lower_pattern_lines_encoding.set(
                game_state.pattern_lines_occupancy[player],
                game_state.pattern_lines_colors[player],
                player,
                &mut self.layer,
            );
            self.score_encoding.set(
                game_state.scores[player],
                game_state.floor_line_progress[player],
                player,
                &mut self.layer,
            );
            self.wall_encoding
                .set(game_state.walls[player], player, &mut self.layer);
        }
        self.bag_encoding.set_bag(&game_state.bag, &mut self.layer);
    }
}

#[cfg(test)]
pub mod tests {
    use super::factories::NonCenterFactoryEncoding;
    use super::pattern_lines::UpperPatternLinesEncoding;
    use super::*;
    use game::{MoveGenerationResult, MoveList};
    use pattern_lines::LowerPatternLinesEncoding;
    use rand::rngs::SmallRng;
    use rand::{Rng as _, SeedableRng as _};
    use wall::WallEncoding;

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
                if game_state.get_possible_moves(&mut move_list, &mut rng)
                    == MoveGenerationResult::GameOver
                {
                    break;
                }
                println!("{}", game_state);
                accumulator.set_game_state(&game_state);

                let move_index = rng.gen_range(0..move_list.len());
                let mov = move_list[move_index];
                game_state.do_move(mov);
            }
        }
    }
}
