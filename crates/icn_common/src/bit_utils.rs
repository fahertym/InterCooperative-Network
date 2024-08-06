// File: crates/icn_common/src/bit_utils.rs

/// A custom bit vector implementation.
#[derive(Clone, Debug)]
pub struct BitVec {
    storage: Vec<u64>,
    len: usize,
}

impl BitVec {
    /// Creates a new `BitVec` with the specified length, initialized to all zeros.
    pub fn new(len: usize) -> Self {
        let storage_len = (len + 63) / 64;
        BitVec {
            storage: vec![0; storage_len],
            len,
        }
    }

    /// Returns the length of the bit vector.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the bit vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Sets the bit at the specified index.
    pub fn set(&mut self, index: usize) {
        assert!(index < self.len, "Index out of bounds");
        let (word_index, bit_index) = (index / 64, index % 64);
        self.storage[word_index] |= 1 << bit_index;
    }

    /// Clears the bit at the specified index.
    pub fn clear(&mut self, index: usize) {
        assert!(index < self.len, "Index out of bounds");
        let (word_index, bit_index) = (index / 64, index % 64);
        self.storage[word_index] &= !(1 << bit_index);
    }

    /// Toggles the bit at the specified index.
    pub fn toggle(&mut self, index: usize) {
        assert!(index < self.len, "Index out of bounds");
        let (word_index, bit_index) = (index / 64, index % 64);
        self.storage[word_index] ^= 1 << bit_index;
    }

    /// Returns the value of the bit at the specified index.
    pub fn get(&self, index: usize) -> bool {
        assert!(index < self.len, "Index out of bounds");
        let (word_index, bit_index) = (index / 64, index % 64);
        (self.storage[word_index] & (1 << bit_index)) != 0
    }

    /// Sets a range of bits to 1.
    pub fn set_range(&mut self, start: usize, end: usize) {
        assert!(start <= end && end < self.len, "Index out of bounds");
        for index in start..=end {
            self.set(index);
        }
    }

    /// Clears a range of bits.
    pub fn clear_range(&mut self, start: usize, end: usize) {
        assert!(start <= end && end < self.len, "Index out of bounds");
        for index in start..=end {
            self.clear(index);
        }
    }

    /// Counts the number of set bits in the bit vector.
    pub fn count_ones(&self) -> u32 {
        self.storage.iter().map(|&x| x.count_ones()).sum()
    }
}

/// Sets the nth bit of a u64 value.
pub fn set_bit(value: u64, n: u8) -> u64 {
    debug_assert!(n < 64, "Bit index out of bounds");
    value | (1 << n)
}

/// Clears the nth bit of a u64 value.
pub fn clear_bit(value: u64, n: u8) -> u64 {
    debug_assert!(n < 64, "Bit index out of bounds");
    value & !(1 << n)
}

/// Toggles the nth bit of a u64 value.
pub fn toggle_bit(value: u64, n: u8) -> u64 {
    debug_assert!(n < 64, "Bit index out of bounds");
    value ^ (1 << n)
}

/// Rotates the bits of a u64 value left by a specified amount.
pub fn rotate_left(value: u64, n: u8) -> u64 {
    value.rotate_left(n as u32)
}

/// Rotates the bits of a u64 value right by a specified amount.
pub fn rotate_right(value: u64, n: u8) -> u64 {
    value.rotate_right(n as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_vec_operations() {
        let mut bv = BitVec::new(100);
        assert_eq!(bv.len(), 100);
        assert!(!bv.is_empty());

        bv.set(5);
        bv.set(50);
        assert!(bv.get(5));
        assert!(bv.get(50));
        assert!(!bv.get(0));

        bv.toggle(5);
        assert!(!bv.get(5));

        bv.clear(50);
        assert!(!bv.get(50));

        bv.set_range(10, 20);
        for i in 10..=20 {
            assert!(bv.get(i));
        }

        bv.clear_range(15, 18);
        for i in 15..=18 {
            assert!(!bv.get(i));
        }

        assert_eq!(bv.count_ones(), 10);

        bv.set(10);
        bv.set(20);
        bv.set(30);
        assert_eq!(bv.count_ones(), 12);
    }

    #[test]
    fn test_bit_operations() {
        assert_eq!(set_bit(0, 3), 8);
        assert_eq!(clear_bit(15, 2), 11);
        assert_eq!(toggle_bit(5, 1), 7);
        assert_eq!(rotate_left(0b1101, 2), 0b110100);
        assert_eq!(rotate_right(0b110100, 2), 0b1101);
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_bit_vec_out_of_bounds() {
        let mut bv = BitVec::new(10);
        bv.set(10);
    }
}
