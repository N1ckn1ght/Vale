const RESERVED: u32 = 0b00000111111111111111111111111111;

#[derive(Clone, Copy)]
pub struct Bitboard {
    rows: [u32; 3]  /* 27-bit * 3 = 81 */
}

impl Bitboard {
    fn default() -> Bitboard {
        Self {
            rows: [0, 0, 0]
        }
    }
    
    pub fn clear(&mut self) {
        self.rows = [0, 0, 0];
    }

    pub fn pop_bit(&mut self) -> usize {
        let bit: usize = 81;
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
}

/* Trait implementation */

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        [self.rows[0] & other.rows[0], self.rows[1] & other.rows[1], self.rows[2] & other.rows[2]]
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, other: Self) -> Self {
        *self.rows[0] &= other.rows[0];
        *self.rows[1] &= other.rows[1];
        *self.rows[2] &= other.rows[2];
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        [self.rows[0] | other.rows[0], self.rows[1] | other.rows[1], self.rows[2] | other.rows[2]]
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, other: Self) -> Self {
        *self.rows[0] |= other.rows[0];
        *self.rows[1] |= other.rows[1];
        *self.rows[2] |= other.rows[2];
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        [self.rows[0] ^ other.rows[0], self.rows[1] ^ other.rows[1], self.rows[2] ^ other.rows[2]]
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, other: Self) -> Self {
        *self.rows[0] ^= other.rows[0];
        *self.rows[1] ^= other.rows[1];
        *self.rows[2] ^= other.rows[2];
    }
}

impl Eq for Bitboard {}

impl PartialEq for Bitboard {
    fn eq(&self, other: &Self) -> Bool {
        (self.rows[0] & RESERVED) == (other.rows[0] & RESERVED) && 
        (self.rows[1] & RESERVED) == (other.rows[1] & RESERVED) && 
        (self.rows[2] & RESERVED) == (other.rows[2] & RESERVED)
    }
}