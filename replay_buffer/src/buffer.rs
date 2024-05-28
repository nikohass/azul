use async_mutex::Mutex;
use game::{GameState, Move, NUM_PLAYERS};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReplayEntry {
    pub game_state: GameState,
    pub value: [f32; NUM_PLAYERS],
    pub iterations: u64,
    pub action_value_pairs: Vec<(Move, [f32; NUM_PLAYERS])>,
}

pub struct Buffer {
    entries: Vec<ReplayEntry>,
    max_size: usize,
    evicted_entries: Vec<ReplayEntry>,
}

impl Buffer {
    pub fn with_size(max_size: usize) -> Arc<Mutex<Self>> {
        let buffer = Arc::new(Mutex::new(Self {
            entries: Vec::new(),
            max_size,
            evicted_entries: Vec::new(),
        }));

        let buffer_clone = buffer.clone();
        tokio::spawn(async move {
            let mut swap_vec = Vec::new();

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                {
                    let mut buffer_lock = buffer_clone.lock().await;
                    std::mem::swap(&mut swap_vec, &mut buffer_lock.evicted_entries);
                }

                // TODO: Save the entries in swap_vec to disk
                println!("Would save {} entries to disk", swap_vec.len());
                swap_vec.clear();
            }
        });

        buffer
    }

    pub fn add_entries(&mut self, entries: Vec<ReplayEntry>) {
        let num_entries = entries.len();
        if self.entries.len() + num_entries > self.max_size {
            let num_to_remove = self.entries.len() + num_entries - self.max_size;
            let removed = self.entries.drain(0..num_to_remove);
            self.evicted_entries.extend(removed);
        }
        self.entries.extend(entries);
    }

    pub fn sample_n_entries(&self, n: usize, rng: &mut impl rand::Rng) -> Vec<ReplayEntry> {
        self.entries.choose_multiple(rng, n).cloned().collect()
    }

    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
    }
}
