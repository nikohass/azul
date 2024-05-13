use game::*;

#[derive(Debug, Clone)]
pub struct ProbabilisticOutcome {
    pub factories: Factories,
    pub bag: Bag,
    pub out_of_bag: Bag,
}

impl ProbabilisticOutcome {
    pub fn apply_to_game_state(&self, game_state: &mut GameState) {
        #[cfg(debug_assertions)]
        {
            let factories = game_state.get_factories();
            assert!(factories.is_empty());
        }
        #[cfg(debug_assertions)]
        let original_game_state = game_state.clone();

        game_state.evaluate_round(); // This will move the tiles from the factories to the pattern lines
        game_state.set_factories(self.factories.clone()); // Overwrite the factories with the outcome of the probabilistic event

        // The number of tiles in and out of bag also changes when the factories are refilled, so overwrite those as well
        game_state.set_out_of_bag(self.out_of_bag);
        game_state.set_bag(self.bag);

        #[cfg(debug_assertions)]
        {
            let err: bool = game_state.check_integrity().is_err();
            if err {
                println!("Original game state:\n{}", original_game_state);
                println!("After applying event:\n{}", game_state);
                println!("Edge:\n{:#?}", self);
                panic!();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Edge {
    Deterministic(Move),
    Probabilistic(ProbabilisticOutcome),
}

impl Edge {
    pub fn apply_to_game_state(&self, game_state: &mut GameState) {
        match self {
            Edge::Deterministic(move_) => game_state.do_move(*move_),
            Edge::Probabilistic(outcome) => {
                #[cfg(debug_assertions)]
                {
                    let factories = game_state.get_factories();
                    assert!(factories.is_empty());
                }
                outcome.apply_to_game_state(game_state)
            }
        }
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Edge::Deterministic(move_) => write!(f, "{}", move_),
            Edge::Probabilistic(outcome) => {
                let string = outcome
                    .factories
                    .iter()
                    .take(NUM_FACTORIES - 1)
                    .map(|factory| {
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
