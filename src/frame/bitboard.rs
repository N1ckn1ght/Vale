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
                self.rows[0] = self.rows[2] << (bits - 54);
                self.rows[1] = 0;
            } else {
                self.rows[0] = (self.rows[1] << (bits - 27)) | ((self.rows[2] & RESERVED) >> (54 - bits));
                self.rows[1] = self.rows[2] << (bits - 27);
            }
            self.rows[2] = 0;
        } else {
            self.rows[0] = (self.rows[0] << bits) | ((self.rows[1] & RESERVED) >> (27 - bits));
            self.rows[1] = (self.rows[1] << bits) | ((self.rows[2] & RESERVED) >> (27 - bits));
            self.rows[2] <<= bits;
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
                self.rows[2] = (self.rows[0] & RESERVED) >> (bits - 54);
                self.rows[1] = 0;
            } else {
                self.rows[2] = ((self.rows[1] & RESERVED) >> (bits - 27)) | (self.rows[0] << (54 - bits));
                self.rows[1] = (self.rows[0] & RESERVED) >> (bits - 27);
            }
            self.rows[0] = 0;
        } else {
            self.rows[2] = ((self.rows[2] & RESERVED) >> bits) | (self.rows[1] << (27 - bits));
            self.rows[1] = ((self.rows[1] & RESERVED) >> bits) | (self.rows[0] << (27 - bits));
            self.rows[0] = (self.rows[0] & RESERVED) >> bits;
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

impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::init(&[!self.rows[0], !self.rows[1], !self.rows[2]])
    }
}

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

    #[test]
    fn bitboard_bits() {
        let mut bb1 = Bitboard::init(&[0b1111, 0b0111, 0b0011]);
        let mut bb2 = Bitboard::init(&[0b0001, 0b1010, 0b1100]);

        let bb3 = bb1 & bb2;
        let bb4 = bb1 | bb2;
        let bb5 = bb1 ^ bb2;
        
        assert_ne!(bb1, bb3);
        assert_eq!(bb3, Bitboard::init(&[0b0001, 0b0010, 0b0000]));
        assert_eq!(bb4, Bitboard::init(&[0b1111, 0b1111, 0b1111]));
        assert_eq!(bb5, Bitboard::init(&[0b1110, 0b1101, 0b1111]));

        bb1 ^= bb2;
        assert_eq!(bb1, bb5);
        bb2 &= bb1;
        assert_eq!(bb2, Bitboard::init(&[0b0000, 0b1000, 0b1100]));
        bb1 |= bb2;
        assert_eq!(bb1, Bitboard::init(&[0b1110, 0b1101, 0b1111]));
        
        let bb6 = !bb1;
        assert_eq!(bb6.get_rows(), [!0b1110, !0b1101, !0b1111]);
    }

    #[test]
    fn bitboard_shift() {
        let bb1 = Bitboard::init(&[0b0110, 0b0001, 0b0100]);
        let bb2 = Bitboard::init(&[0b1100, 0b0010, 0b1000]);
        let bb3 = Bitboard::init(&[0b0001, 0b0100, 0b0000]);
        let bb4 = Bitboard::init(&[0b0100, 0b0000, 0b0000]);
        let bb5 = Bitboard::init(&[0b0000, 0b0000, 0b0000]);
        let bb6 = Bitboard::init(&[0b0000, 0b0110, 0b0001]);
        let bb7 = Bitboard::init(&[0b0000, 0b0000, 0b0011]);
        let bb8 = Bitboard::init(&[0b0000, 0b0000, 0b1100]);

        let mut bb9 = bb1.clone();
        bb9.left_shift(1);
        assert_eq!(bb2, bb9);

        let mut bb9_2 = bb1.clone();
        bb9_2.left_shift(2);
        bb9_2.right_shift(1);
        assert_eq!(bb2, bb9_2);

        let mut bb10 = bb1.clone();
        bb10.left_shift(27);
        assert_eq!(bb3, bb10);

        let mut bb10_2 = bb1.clone();
        bb10_2.left_shift(1);
        bb10_2.left_shift(26);
        assert_eq!(bb3, bb10_2);

        let mut bb11 = bb1.clone();
        bb11.left_shift(54);
        assert_eq!(bb4, bb11);

        let mut bb12 = bb1.clone();
        bb12.left_shift(80);
        assert_eq!(bb5, bb12);

        let mut bb13 = bb1.clone();
        bb13.right_shift(80);
        assert_eq!(bb5, bb13);

        let mut bb14 = bb1.clone();
        bb14.right_shift(27);
        assert_eq!(bb6, bb14);

        let mut bb15 = bb1.clone();
        bb15.right_shift(55);
        assert_eq!(bb7, bb15);

        let mut bb16 = bb1.clone();
        bb16.right_shift(53);
        assert_eq!(bb8, bb16);

        let mut bb17 = bb6.clone();
        let mut bb18 = bb6.clone();
        bb17.left_shift(26);
        bb18.left_shift(40);
        for _ in 0..14 {
            bb18.right_shift(1);
        }
        assert_eq!(bb17, bb18);
    }
}