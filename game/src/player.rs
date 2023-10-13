use super::NUM_PLAYERS;

pub struct Player(u8);

impl Player {
    pub fn new(id: u8) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u8 {
        self.0
    }

    pub fn next(&self) -> Self {
        Self((self.0 + 1) % (NUM_PLAYERS as u8))
    }
}
