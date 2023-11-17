#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    IllegalMove,
    PlayerCountMismatch,
    InvalidGameState,
}
