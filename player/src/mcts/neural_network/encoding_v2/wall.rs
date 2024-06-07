use game::wall;

pub fn calculate_upper_index(wall: u32) -> usize {
    // Row 1 and 2 (2 ** 10 = 1024)
    let mut index = wall & wall::get_row_mask(0);
    index <<= 5;
    index |= (wall & wall::get_row_mask(1)) >> 6;
    index as usize
}

pub fn calculate_middle_index(wall: u32) -> usize {
    // Row 3 and 4 (2 ** 10 = 1024)
    let mut index = (wall & wall::get_row_mask(2)) >> 12;
    index <<= 5;
    index |= (wall & wall::get_row_mask(3)) >> 18;
    index as usize
}

pub fn calculate_lower_index(wall: u32) -> usize {
    // Row 5 (2 ** 5 = 32)
    ((wall & wall::get_row_mask(4)) >> 24) as usize
}

#[cfg(test)]
mod tests {
    use wall::print_bitboard;

    use super::*;

    #[test]
    fn test_calculate_upper_index_all_combinations() {
        let mut indices_set = std::collections::HashSet::new();
        for row1 in 0..=31 {
            for row2 in 0..=31 {
                let wall = (row1 << 6) | row2;
                // print_bitboard(wall);
                let index = calculate_upper_index(wall);
                indices_set.insert(index);
            }
        }
        assert_eq!(indices_set.len(), 1024);
        let max = indices_set.iter().max().unwrap();
        assert_eq!(*max, 1023);
    }

    #[test]
    fn test_calculate_middle_index_all_combinations() {
        let mut indices_set = std::collections::HashSet::new();
        for row3 in 0..=31 {
            for row4 in 0..=31 {
                let wall = (row3 << 12) | (row4 << 18);
                // print_bitboard(wall);
                let index = calculate_middle_index(wall);
                indices_set.insert(index);
            }
        }
        assert_eq!(indices_set.len(), 1024);
        let max = indices_set.iter().max().unwrap();
        assert_eq!(*max, 1023);
    }

    #[test]
    fn test_calculate_lower_index_all_combinations() {
        let mut indices_set = std::collections::HashSet::new();
        for row5 in 0..=31 {
            let wall = row5 << 24;
            print_bitboard(wall);
            let index = calculate_lower_index(wall);
            indices_set.insert(index);
        }
        assert_eq!(indices_set.len(), 32);
        let max = indices_set.iter().max().unwrap();
        assert_eq!(*max, 31);
    }
}
