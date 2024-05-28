use crate::NUM_FACTORIES;

use super::Move;

const MAX_MOVES: usize = ((NUM_FACTORIES - 2) * 4 + 5) * 6;

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

    pub fn swap(&mut self, index1: usize, index2: usize) {
        #[cfg(debug_assertions)]
        {
            if index1 >= self.len || index2 >= self.len {
                panic!("Index out of bounds");
            }
        }
        self.moves.swap(index1, index2);
    }

    pub fn remove(&mut self, index: usize) {
        #[cfg(debug_assertions)]
        {
            if index >= self.len {
                panic!("Index out of bounds");
            }
        }
        self.swap(index, self.len - 1);
        self.len -= 1;
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        #[cfg(debug_assertions)]
        {
            if index >= self.len {
                panic!("Index out of bounds");
            }
        }
        &self.moves[index]
    }
}

impl std::ops::IndexMut<usize> for MoveList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        #[cfg(debug_assertions)]
        {
            if index >= self.len {
                panic!("Index out of bounds");
            }
        }
        &mut self.moves[index]
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

impl std::fmt::Debug for MoveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl std::fmt::Display for MoveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, move_) in self.into_iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", move_)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let move_list = MoveList::new();
        assert_eq!(move_list.len(), 0);
        assert!(move_list.is_empty());
    }

    #[test]
    fn test_push_and_len() {
        let mut move_list = MoveList::new();
        let move_ = Move::DUMMY;
        move_list.push(move_);
        assert_eq!(move_list.len(), 1);
        assert!(!move_list.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut move_list = MoveList::new();
        move_list.push(Move::DUMMY);
        move_list.clear();
        assert_eq!(move_list.len(), 0);
        assert!(move_list.is_empty());
    }

    #[test]
    fn test_contains() {
        let mut move_list = MoveList::new();
        let move_ = Move::DUMMY;
        assert!(!move_list.contains(move_));
        move_list.push(move_);
        assert!(move_list.contains(move_));
    }

    #[test]
    fn test_index() {
        let mut move_list = MoveList::new();
        let move_ = Move::DUMMY;
        move_list.push(move_);
        assert_eq!(&move_list[0], &move_);
    }

    #[test]
    fn test_iter() {
        let mut move_list = MoveList::new();
        move_list.push(Move::DUMMY);
        move_list.push(Move::DUMMY);
        let mut iter = move_list.into_iter();
        assert_eq!(iter.next(), Some(&Move::DUMMY));
        assert_eq!(iter.next(), Some(&Move::DUMMY));
        assert_eq!(iter.next(), None);
    }
}
