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

impl Into<Player> for u8 {
    fn into(self) -> Player {
        Player(self)
    }
}

impl Into<Player> for usize {
    fn into(self) -> Player {
        Player(self as u8)
    }
}

impl Into<u8> for Player {
    fn into(self) -> u8 {
        self.0
    }
}

impl Into<usize> for Player {
    fn into(self) -> usize {
        self.0 as usize
    }
}
