use super::NUM_PLAYERS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Player(u8);

impl Player {
    pub fn new(id: u8) -> Self {
        Self(id)
    }

    pub fn next(&self) -> Self {
        Self((self.0 + 1) % (NUM_PLAYERS as u8))
    }
}

impl From<Player> for u8 {
    fn from(val: Player) -> Self {
        val.0
    }
}

impl From<Player> for usize {
    fn from(val: Player) -> Self {
        val.0 as usize
    }
}
