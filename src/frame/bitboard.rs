const RESERVED: u32 = 0b00000111111111111111111111111111;

// Used for 9x9 board. It's preferrable to use simple u16 for 3x3 boards.
#[derive(Clone, Copy, Debug)]
pub struct Bitboard {
    pub rows: [u32; 3]  /* 27-bit * 3 = 81 */
}

impl Bitboard {
    pub fn bit() -> Bitboard {
        Self {
            rows: [0, 0, 1]
        }
    }
    
    #[inline]
    pub fn clear(&mut self) {
        self.rows = [0, 0, 0];
    }

    pub fn default() -> Bitboard {
        Self {
            rows: [0, 0, 0]
        }
    }

    pub fn del_bit(&mut self, bit: usize) {
        *self &= !(Self::bit() << bit);
    }

    pub fn get_bit(&self, bit: usize) -> Self {
        *self & (Self::bit() << bit)
    }

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

    pub fn pop_bit(&mut self) -> usize {
        let mut bit: usize = 81;
        if self.rows[2] & RESERVED != 0 {
            bit = u32::trailing_zeros(self.rows[2]) as usize;
            self.rows[2] &= self.rows[2] - 1;
        }
        else if self.rows[1] & RESERVED != 0 {
            bit = u32::trailing_zeros(self.rows[1]) as usize + 27;
            self.rows[1] &= self.rows[1] - 1;
        }
        else if self.rows[0] & RESERVED != 0 {
            bit = u32::trailing_zeros(self.rows[0]) as usize + 54;
            self.rows[0] &= self.rows[0] - 1;
        }
        return bit;
    }

    pub fn set_bit(&mut self, bit: usize) {
        *self |= Self::bit() << bit;
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

/* Trait implementations for bitboards */

impl Eq for Bitboard {}

impl PartialEq for Bitboard {
    fn eq(&self, other: &Self) -> bool {
        (self.rows[0] & RESERVED) == (other.rows[0] & RESERVED) && 
        (self.rows[1] & RESERVED) == (other.rows[1] & RESERVED) && 
        (self.rows[2] & RESERVED) == (other.rows[2] & RESERVED)
    }
}

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

impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::init(&[!self.rows[0], !self.rows[1], !self.rows[2]])
    }
}

impl std::ops::Shl<usize> for Bitboard {
    type Output = Self;

    fn shl(self, shift: usize) -> Self::Output {
        let mut bb = self.clone();
        bb <<= shift;
        bb
    }
}

impl std::ops::ShlAssign<usize> for Bitboard {
    fn shl_assign(&mut self, shift: usize) {
        if shift > 26 {
            if shift > 53 {
                self.rows[0] = self.rows[2] << (shift - 54);
                self.rows[1] = 0;
            } else {
                self.rows[0] = (self.rows[1] << (shift - 27)) | ((self.rows[2] & RESERVED) >> (54 - shift));
                self.rows[1] = self.rows[2] << (shift - 27);
            }
            self.rows[2] = 0;
        } else {
            self.rows[0] = (self.rows[0] << shift) | ((self.rows[1] & RESERVED) >> (27 - shift));
            self.rows[1] = (self.rows[1] << shift) | ((self.rows[2] & RESERVED) >> (27 - shift));
            self.rows[2] <<= shift;
        }
    }
}

impl std::ops::Shr<usize> for Bitboard {
    type Output = Self;

    fn shr(self, shift: usize) -> Self::Output {
        let mut bb = self.clone();
        bb >>= shift;
        bb
    }
}

impl std::ops::ShrAssign<usize> for Bitboard {
    fn shr_assign(&mut self, shift: usize) {
        if shift > 26 {
            if shift > 53 {
                self.rows[2] = (self.rows[0] & RESERVED) >> (shift - 54);
                self.rows[1] = 0;
            } else {
                self.rows[2] = ((self.rows[1] & RESERVED) >> (shift - 27)) | (self.rows[0] << (54 - shift));
                self.rows[1] = (self.rows[0] & RESERVED) >> (shift - 27);
            }
            self.rows[0] = 0;
        } else {
            self.rows[2] = ((self.rows[2] & RESERVED) >> shift) | (self.rows[1] << (27 - shift));
            self.rows[1] = ((self.rows[1] & RESERVED) >> shift) | (self.rows[0] << (27 - shift));
            self.rows[0] = (self.rows[0] & RESERVED) >> shift;
        }
    }
}

/* Trait implementations for primivites */

pub trait DelBit<T> {
    fn del_bit(&mut self, bit: u8);
}

pub trait GetBit<T> {
    fn get_bit(&self, bit: u8) -> Self;
}

pub trait SetBit<T> {
    fn set_bit(&mut self, bit: u8);
}

pub trait PopBit<T> {
    fn pop_bit(&mut self) -> u8;
}

impl DelBit<&u8> for u16 {
    fn del_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }
}

impl GetBit<&u8> for u16 {
    fn get_bit(&self, bit: u8) -> Self {
        *self & (1 << bit)
    }
}

impl SetBit<&u8> for u16 {
    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }
}

impl PopBit<&u8> for u16 {
    fn pop_bit(&mut self) -> u8 {
        let tz = self.trailing_zeros() as u8;
        *self &= *self - 1;
        tz
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
        bb9 <<= 1;
        assert_eq!(bb2, bb9);

        let mut bb9_2 = bb1.clone();
        bb9_2 <<= 2;
        bb9_2 >>= 1;
        assert_eq!(bb2, bb9_2);

        let mut bb10 = bb1.clone();
        bb10 <<= 27;
        assert_eq!(bb3, bb10);

        let mut bb10_2 = bb1.clone();
        bb10_2 <<= 1;
        bb10_2 <<= 26;
        assert_eq!(bb3, bb10_2);

        let mut bb11 = bb1.clone();
        bb11 <<= 54;
        assert_eq!(bb4, bb11);

        let mut bb12 = bb1.clone();
        bb12 <<= 80;
        assert_eq!(bb5, bb12);

        let mut bb13 = bb1.clone();
        bb13 >>= 80;
        assert_eq!(bb5, bb13);

        let mut bb14 = bb1.clone();
        bb14 >>= 27;
        assert_eq!(bb6, bb14);

        let mut bb15 = bb1.clone();
        bb15 >>= 55;
        assert_eq!(bb7, bb15);

        let mut bb16 = bb1.clone();
        bb16 >>= 53;
        assert_eq!(bb8, bb16);

        let mut bb17 = bb6.clone();
        let mut bb18 = bb6.clone();
        bb17 <<= 26;
        bb18 <<= 40;
        for _ in 0..14 {
            bb18 >>= 1;
        }
        assert_eq!(bb17, bb18);

        let mut bb19 = bb1.clone();
        bb19 <<= 1;
        assert_eq!(bb19 >> 1, bb1);
        bb19 >>= 2;
        assert_eq!(bb19 << 1, bb1);
    }

    #[test]
    fn bitboard_util() {
        let mut bb1 = Bitboard::default();
        bb1.set_bit(1);
        bb1.set_bit(27);
        bb1.set_bit(80);
        assert_eq!(bb1, Bitboard::init(&[1 << 26, 1, 2]));
        assert_eq!(bb1.get_bit(0), Bitboard::default());
        assert_eq!(bb1.get_bit(1), Bitboard::init(&[0, 0, 2]));
        assert_eq!(bb1.get_bit(26), Bitboard::default());
        assert_eq!(bb1.get_bit(27), Bitboard::init(&[0, 1, 0]));
        assert_eq!(bb1.get_bit(28), Bitboard::default());
        assert_eq!(bb1.get_bit(80), Bitboard::init(&[1 << 26, 0, 0]));
        assert_eq!(bb1.get_bit(79), Bitboard::default());
        bb1.del_bit(27);
        assert_eq!(bb1.get_bit(27), Bitboard::default());
        bb1.set_bit(27);

        assert_eq!(bb1, Bitboard::init(&[1 << 26, 1, 2]));
        let res1 = bb1.pop_bit();
        assert_eq!(bb1, Bitboard::init(&[1 << 26, 1, 0]));
        assert_eq!(res1, 1);
        let res2 = bb1.pop_bit();
        assert_eq!(bb1, Bitboard::init(&[1 << 26, 0, 0]));
        assert_eq!(res2, 27);
        let res3 = bb1.pop_bit();
        assert_eq!(bb1, Bitboard::init(&[0, 0, 0]));
        assert_eq!(res3, 80);
    }

    #[test]
    fn primitives_util() {
        let mut val: u16 = 0;
        val.set_bit(0);
        val.set_bit(2);
        assert_eq!(val, 5);
        val.del_bit(1);
        val.del_bit(0);
        assert_eq!(val, 4);
        val.set_bit(3);
        assert_eq!(val.get_bit(0), 0);
        assert_eq!(val.get_bit(2), 4);
        assert_eq!(val.pop_bit(), 2);
        assert_eq!(val, 8);
        assert_eq!(val.pop_bit(), 3);
        assert_eq!(val, 0);
    }
}