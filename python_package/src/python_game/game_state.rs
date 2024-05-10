use super::{move_::Move, player_marker::PlayerMarker, tile_color::TileColor};
use game::{Factories, Factory, MoveList, NUM_FACTORIES, NUM_PLAYERS, NUM_TILE_COLORS};
use pyo3::{basic::CompareOp, exceptions::PyValueError, pyclass, pymethods, PyResult};
use rand::{rngs::SmallRng, SeedableRng as _};

#[pyclass]
#[derive(Clone)]
pub struct MoveGenerationResult(pub game::MoveGenerationResult);

#[pymethods]
impl MoveGenerationResult {
    fn is_game_over(&self) -> bool {
        matches!(self.0, game::MoveGenerationResult::GameOver)
    }

    fn is_round_over(&self) -> bool {
        matches!(self.0, game::MoveGenerationResult::RoundOver)
    }

    fn is_continue(&self) -> bool {
        matches!(self.0, game::MoveGenerationResult::Continue)
    }

    fn __richcmp__(&self, other: Self, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.0 == other.0,
            CompareOp::Ne => self.0 != other.0,
            _ => false,
        }
    }

    fn __str__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyclass]
pub struct GameState(pub game::GameState);

#[pymethods]
impl GameState {
    #[new]
    fn new(fen: Option<&str>) -> PyResult<Self> {
        Ok(Self(match fen {
            None => {
                let mut rng = SmallRng::from_entropy();
                game::GameState::new(&mut rng)
            }
            Some(fen) => game::GameState::deserialize_string(fen)
                .map_err(|e| PyValueError::new_err(format!("Invalid FEN: {}", e)))?,
        }))
    }

    fn __str__(&self) -> String {
        format!("{}", self.0)
    }

    fn __repr__(&self) -> String {
        format!("{}", self.0)
    }

    #[getter]
    fn current_player(&self) -> PlayerMarker {
        PlayerMarker(self.0.get_current_player())
    }

    #[getter]
    fn next_round_starting_player(&self) -> PlayerMarker {
        PlayerMarker(self.0.get_next_round_starting_player())
    }

    #[getter]
    fn scores(&self) -> Vec<i16> {
        self.0.get_scores().to_vec()
    }

    #[getter]
    fn bag(&self) -> Vec<u8> {
        self.0.get_bag().to_vec()
    }

    fn set_bag(&mut self, bag: Vec<u8>) -> PyResult<()> {
        let bag: [u8; NUM_TILE_COLORS] = bag
            .try_into()
            .map_err(|_| PyValueError::new_err("Bag must have 5 elements"))?;
        self.0.set_bag(bag);
        Ok(())
    }

    #[getter]
    fn out_of_bag(&self) -> Vec<u8> {
        self.0.get_out_of_bag().to_vec()
    }

    fn set_out_of_bag(&mut self, out_of_bag: Vec<u8>) -> PyResult<()> {
        let out_of_bag: [u8; NUM_TILE_COLORS] = out_of_bag.try_into().map_err(|_| {
            PyValueError::new_err(format!("Out of bag must have {} elements", NUM_TILE_COLORS))
        })?;
        self.0.set_out_of_bag(out_of_bag);
        Ok(())
    }

    #[getter]
    fn factories(&self) -> Vec<Vec<u8>> {
        self.0.get_factories().iter().map(|f| f.to_vec()).collect()
    }

    fn set_factories(&mut self, factories: [Factory; NUM_FACTORIES]) -> PyResult<()> {
        self.0.set_factories(Factories::from(factories));
        Ok(())
    }

    #[getter]
    fn floor_line_progress(&self) -> Vec<u8> {
        self.0.get_floor_line_progress().to_vec()
    }

    #[getter]
    fn walls(&self) -> [[u32; NUM_TILE_COLORS]; NUM_PLAYERS] {
        *self.0.get_walls()
    }

    fn set_walls(&mut self, walls: [[u32; NUM_TILE_COLORS]; NUM_PLAYERS]) {
        self.0.set_walls(walls);
    }

    #[getter]
    fn get_wall_occupancy(&self) -> [u32; NUM_PLAYERS] {
        self.0.get_wall_occupancy()
    }

    #[getter]
    fn pattern_line_occupancy(&self) -> [[u8; 5]; NUM_PLAYERS] {
        *self.0.get_pattern_lines_occupancy()
    }

    fn set_pattern_line_occupancy(&mut self, occupancy: [[u8; 5]; NUM_PLAYERS]) {
        self.0.set_pattern_lines_occupancy(occupancy);
    }

    #[getter]
    fn pattern_line_colors(&self) -> [[Option<TileColor>; 5]; NUM_PLAYERS] {
        let mut result = [[None; 5]; NUM_PLAYERS];
        for (player_index, pattern_colors) in self.0.get_pattern_line_colors().iter().enumerate() {
            for (pattern_index, &color) in pattern_colors.iter().enumerate() {
                result[player_index][pattern_index] = color.map(TileColor);
            }
        }
        result
    }

    fn set_pattern_line_colors(&mut self, colors: [[Option<TileColor>; 5]; NUM_PLAYERS]) {
        let mut result = [[None; 5]; NUM_PLAYERS];
        for (i, pattern_colors) in colors.iter().enumerate() {
            for (j, color) in pattern_colors.iter().enumerate() {
                result[i][j] = color.map(|c| c.0);
            }
        }
        self.0.set_pattern_line_colors(result);
    }

    #[getter]
    fn tile_taken_from_center(&self) -> bool {
        self.0.get_tile_taken_from_center()
    }

    #[getter]
    fn fen(&self) -> String {
        self.0.serialize_string()
    }

    fn do_move(&mut self, move_: Move) {
        self.0.do_move(move_.0);
    }

    fn get_possible_moves(&mut self) -> (Vec<Move>, MoveGenerationResult) {
        let mut move_list = MoveList::default();
        let mut rng = SmallRng::from_entropy();
        let result = self.0.get_possible_moves(&mut move_list, &mut rng);
        let moves = move_list.into_iter().map(|m| Move(*m)).collect::<Vec<_>>();
        (moves, MoveGenerationResult(result))
    }

    fn check_integrity(&self) -> PyResult<()> {
        self.0
            .check_integrity()
            .map_err(|_| PyValueError::new_err("Integrity check failed.".to_string()))
    }
}
