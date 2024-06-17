pub use value_inner::Value;

// Value for more than 2 players
#[cfg(any(feature = "three_players", feature = "four_players"))]
mod value_inner {
    use game::NUM_PLAYERS;

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Value([f64; NUM_PLAYERS]);

    impl Value {
        pub fn from_game_scores(game_scores: [i16; NUM_PLAYERS]) -> Self {
            let max_score = game_scores.iter().cloned().fold(i16::MIN, i16::max);
            let min_score = game_scores.iter().cloned().fold(i16::MAX, i16::min);

            let score_range = max_score - min_score;
            if score_range == 0 {
                // If all scores are the same, return 1 / NUM_PLAYERS for each player
                // e.g. if there are 2 players, return [0.5, 0.5] for each player
                return Self([1.0 / NUM_PLAYERS as f64; NUM_PLAYERS]);
            }

            let mut value = [0.0; NUM_PLAYERS];
            let score_range = score_range as f64;
            for (i, &score) in game_scores.iter().enumerate() {
                let normalized_score = (score - min_score) as f64 / score_range;
                // Add very small bonus for higher scores.
                // Otherwise the program will do random moves as soon as its victory is inevitable which is perceived as arrogant by other players.
                value[i] = normalized_score + score as f64 * 0.00001;
            }

            // Divide by the sum of all values to normalize them
            let sum: f64 = value.iter().sum();
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

    impl std::ops::Add for Value {
        type Output = Self;

        fn add(mut self, rhs: Self) -> Self::Output {
            self += rhs;
            self
        }
    }

    impl std::ops::DivAssign<f64> for Value {
        fn div_assign(&mut self, rhs: f64) {
            for value in self.0.iter_mut() {
                *value /= rhs;
            }
        }
    }

    impl std::ops::Div<f64> for Value {
        type Output = Self;

        fn div(mut self, rhs: f64) -> Self::Output {
            self /= rhs;
            self
        }
    }

    impl std::ops::MulAssign<f64> for Value {
        fn mul_assign(&mut self, rhs: f64) {
            for value in self.0.iter_mut() {
                *value *= rhs;
            }
        }
    }

    impl std::ops::Mul<f64> for Value {
        type Output = Self;

        fn mul(mut self, rhs: f64) -> Self::Output {
            self *= rhs;
            self
        }
    }

    impl std::ops::Index<usize> for Value {
        type Output = f64;

        fn index(&self, index: usize) -> &Self::Output {
            &self.0[index]
        }
    }

    impl std::convert::From<Value> for [f64; NUM_PLAYERS] {
        fn from(value: Value) -> Self {
            value.0
        }
    }

    impl std::convert::From<[f64; NUM_PLAYERS]> for Value {
        fn from(value: [f64; NUM_PLAYERS]) -> Self {
            Self(value)
        }
    }

    impl std::iter::Sum for Value {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Value::default(), std::ops::Add::add)
        }
    }
}

// Value for 2 players
#[cfg(not(any(feature = "three_players", feature = "four_players")))]
mod value_inner {
    use game::NUM_PLAYERS;

    #[derive(Debug, Clone, Copy, Default)]
    pub struct Value(f64);

    impl Value {
        pub fn from_game_scores(game_scores: [i16; NUM_PLAYERS]) -> Self {
            let score_diff = (game_scores[0] - game_scores[1]).abs() as f64;
            let max_diff = 100.0; 
            let bonus = score_diff / max_diff * 0.01; 

            if game_scores[0] == game_scores[1] {
                return Self(0.5);
            }

            if game_scores[0] > game_scores[1] {
                Self(1.0 + bonus.min(0.01)) 
            } else {
                Self(-bonus.min(0.01)) 
            }
        }
    }

    impl std::fmt::Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:.2} {:.2}", self.0, 1.0 - self.0)
        }
    }

    impl std::ops::AddAssign for Value {
        fn add_assign(&mut self, rhs: Self) {
            self.0 += rhs.0;
        }
    }

    impl std::ops::Add for Value {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(self.0 + rhs.0)
        }
    }

    impl std::ops::DivAssign<f64> for Value {
        fn div_assign(&mut self, rhs: f64) {
            self.0 /= rhs;
        }
    }

    impl std::ops::Div<f64> for Value {
        type Output = Self;

        fn div(self, rhs: f64) -> Self::Output {
            Self(self.0 / rhs)
        }
    }

    impl std::ops::MulAssign<f64> for Value {
        fn mul_assign(&mut self, rhs: f64) {
            self.0 *= rhs;
        }
    }

    impl std::ops::Mul<f64> for Value {
        type Output = Self;

        fn mul(self, rhs: f64) -> Self::Output {
            Self(self.0 * rhs)
        }
    }

    impl std::ops::Index<usize> for Value {
        type Output = f64;

        fn index(&self, index: usize) -> &f64 {
            match index {
                0 => &self.0,
                1 => {
                    let complement = 1.0 - self.0;
                    Box::leak(Box::new(complement))
                }
                _ => panic!("Index out of bounds"),
            }
        }
    }

    impl std::convert::From<Value> for [f64; 2] {
        fn from(value: Value) -> Self {
            [value.0, 1.0 - value.0]
        }
    }

    impl std::convert::From<[f64; NUM_PLAYERS]> for Value {
        fn from(value: [f64; NUM_PLAYERS]) -> Self {
            Self(value[0])
        }
    }

    impl std::iter::Sum for Value {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Value::default(), std::ops::Add::add)
        }
    }

    impl std::ops::Sub for Value {
        type Output = f64;

        fn sub(self, rhs: Self) -> Self::Output {
            self.0 - rhs.0
        }
    }
}
