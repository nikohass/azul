use game::*;

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

impl std::convert::From<Value> for [f32; NUM_PLAYERS] {
    fn from(value: Value) -> Self {
        value.0
    }
}

impl std::convert::From<[f32; NUM_PLAYERS]> for Value {
    fn from(value: [f32; NUM_PLAYERS]) -> Self {
        Self(value)
    }
}
