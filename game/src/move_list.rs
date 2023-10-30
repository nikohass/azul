use super::Move;

const MAX_MOVES: usize = 1024;

pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    pub fn new() -> Self {
        #[allow(clippy::uninit_assumed_init, invalid_value)]
        // Safety: We only read from the array after it has been initialized.
        let moves = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        Self { moves, len: 0 }
    }

    pub fn push(&mut self, move_: Move) {
        self.moves[self.len] = move_;
        self.len += 1;
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn contains(&self, move_: Move) -> bool {
        for i in 0..self.len {
            if self.moves[i] == move_ {
                return true;
            }
        }
        false
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("Index out of bounds");
        } else {
            &self.moves[index]
        }
    }
}

impl std::ops::IndexMut<usize> for MoveList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("Index out of bounds");
        } else {
            &mut self.moves[index]
        }
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MoveList {
    fn drop(&mut self) {
        // Safety: Ensure we only drop the initialized elements.
        let ptr = self.moves.as_mut_ptr() as *mut [Move; MAX_MOVES];
        for i in 0..self.len {
            unsafe { std::ptr::drop_in_place(&mut (*ptr)[i]) };
        }
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = std::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: We use self.len to ensure we only access initialized memory.
        let slice: &'a [Move] =
            unsafe { std::slice::from_raw_parts(self.moves.as_ptr() as *const _, self.len) };
        slice.iter()
    }
}

impl<'a> IntoIterator for &'a mut MoveList {
    type Item = &'a mut Move;
    type IntoIter = std::slice::IterMut<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        // Safety: We use self.len to ensure we only access initialized memory.
        let slice: &'a mut [Move] =
            unsafe { std::slice::from_raw_parts_mut(self.moves.as_mut_ptr() as *mut _, self.len) };
        slice.iter_mut()
    }
}
