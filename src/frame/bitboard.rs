const RESERVED: u32 = 0b00000111111111111111111111111111;

#[derive(Clone, Copy, Debug)]
pub struct Bitboard {
    pub rows: [u32; 3]  /* 27-bit * 3 = 81 */
}

impl Bitboard {
    fn default() -> Bitboard {
        Self {
            rows: [0, 0, 0]
        }
    }
    
    #[inline]
    pub fn clear(&mut self) {
        self.rows = [0, 0, 0];
    }

    // pub fn get_bit(&self, bit: usize) -> Self {
        
    // }

    #[inline]
    pub fn get_row(&self, index: usize) -> u32 {
        self.rows[index]
    }

    #[inline]
    pub fn get_rows(&self) -> [u32; 3] {
        self.rows.clone()
    }

    pub fn init(values: &[u32; 3]) -> Self {
        Self {
            rows: [values[0], values[1], values[2]]
        }
    }

    pub fn left_shift(&mut self, bits: usize) {
        if bits > 26 {
            if bits > 53 {
                self.rows[2] = self.rows[0] << (bits - 54);
                self.rows[1] = 0;
            } else {
                self.rows[2] = (self.rows[1] << (bits - 27)) | ((self.rows[0] & RESERVED) >> (54 - bits));
                self.rows[1] = self.rows[0] << (bits - 27);
            }
            self.rows[0] = 0;
        } else {
            self.rows[2] = (self.rows[2] << bits) | ((self.rows[1] & RESERVED) >> (27 - bits));
            self.rows[1] = (self.rows[1] << bits) | ((self.rows[0] & RESERVED) >> (27 - bits));
            self.rows[0] <<= bits;
        }
    }

    pub fn pop_bit(&mut self) -> usize {
        let mut bit: usize = 81;
        if self.rows[0] & RESERVED != 0 {
            self.rows[0] &= self.rows[0] - 1;
            bit = u32::trailing_zeros(self.rows[0]) as usize;
        }
        else if self.rows[1] & RESERVED != 0 {
            self.rows[1] &= self.rows[1] - 1;
            bit = u32::trailing_zeros(self.rows[1]) as usize + 27;
        }
        else if self.rows[2] & RESERVED != 0 {
            self.rows[2] &= self.rows[2] - 1;
            bit = u32::trailing_zeros(self.rows[2]) as usize + 54;
        }
        return bit;
    }

    pub fn right_shift(&mut self, bits: usize) {
        if bits > 26 {
            if bits > 53 {
                self.rows[0] = (self.rows[2] & RESERVED) >> (bits - 54);
                self.rows[1] = 0;
            } else {
                self.rows[0] = ((self.rows[1] & RESERVED) >> (bits - 27)) | (self.rows[2] >> (54 - bits));
                self.rows[1] = (self.rows[2] & RESERVED) >> (bits - 27);
            }
            self.rows[2] = 0;
        } else {
            self.rows[0] = ((self.rows[0] & RESERVED) >> bits) | (self.rows[1] >> (27 - bits));
            self.rows[1] = ((self.rows[1] & RESERVED) >> bits) | (self.rows[2] >> (27 - bits));
            self.rows[2] = (self.rows[2] & RESERVED) >> bits;
        }
    }

    #[inline]
    pub fn set_row(&mut self, index: usize, value: u32) {
        self.rows[index] = value;
    }

    #[inline]
    pub fn set_rows(&mut self, values: &[u32; 3]) {
        self.rows = values.clone();
    }
}

/* Trait implementation */

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        Self::init(&[self.rows[0] & other.rows[0], self.rows[1] & other.rows[1], self.rows[2] & other.rows[2]])
    }
}

impl std::ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, other: Self) {
        self.rows[0] &= other.rows[0];
        self.rows[1] &= other.rows[1];
        self.rows[2] &= other.rows[2];
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self::init(&[self.rows[0] | other.rows[0], self.rows[1] | other.rows[1], self.rows[2] | other.rows[2]])
    }
}

impl std::ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Self) {
        self.rows[0] |= other.rows[0];
        self.rows[1] |= other.rows[1];
        self.rows[2] |= other.rows[2];
    }
}

impl std::ops::BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        Self::init(&[self.rows[0] ^ other.rows[0], self.rows[1] ^ other.rows[1], self.rows[2] ^ other.rows[2]])
    }
}

impl std::ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, other: Self) {
        self.rows[0] ^= other.rows[0];
        self.rows[1] ^= other.rows[1];
        self.rows[2] ^= other.rows[2];
    }
}

impl Eq for Bitboard {}

impl PartialEq for Bitboard {
    fn eq(&self, other: &Self) -> bool {
        (self.rows[0] & RESERVED) == (other.rows[0] & RESERVED) && 
        (self.rows[1] & RESERVED) == (other.rows[1] & RESERVED) && 
        (self.rows[2] & RESERVED) == (other.rows[2] & RESERVED)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitboard_eq() {
        let mut bb1 = Bitboard::default();
        bb1.set_rows(&[0b1111, 0b0111, 0b0011]);
        let mut bb2 = Bitboard::default();
        bb2.set_rows(&[0b1111, 0b0111, 0b0011]);
        let mut bb3 = Bitboard::default();
        bb3.set_rows(&[0b1111, 0b0011, 0b0011]);
        let mut bb4 = Bitboard::default();
        bb4.set_rows(&[0b0111, 0b0111, 0b1011]);
        let mut bb5 = Bitboard::default();
        bb5.set_rows(&[0b0111, 0b0111, 0b1011]);
        let mut bb6 = Bitboard::default();
        bb6.set_rows(&[0b1111 | (RESERVED + 1), 0b0111, 0b0011]);

        assert_eq!(bb1, bb2);
        assert_ne!(bb1, bb3);
        assert_ne!(bb1, bb4);
        assert_ne!(bb1, bb5);
        assert_eq!(bb1, bb6);

        let rows = bb1.get_rows();
        let bb7 = Bitboard::init(&rows);
        let mut bb8 = Bitboard::default();
        bb8.set_rows(&rows);

        assert_eq!(bb1, bb7);
        assert_eq!(bb7, bb8);
    }
}