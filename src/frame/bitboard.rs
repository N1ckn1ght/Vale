// Implements traits for u16 (3x3 boards) and u128 (9x9 board)

pub const LF: u128 = 0b111111111111111111111111111111111111111111111111111111111111111111111111111111111;
pub const LS: u128 = 0b111111111;
pub const SF: u16  = 0b111111111;

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

pub trait SwapBits<T> {
    fn swap_bits(&mut self, first: u8, second: u8);
}


impl DelBit<&u8> for u16 {
    #[inline]
    fn del_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }
}

impl DelBit<&u8> for u128 {
    #[inline]
    fn del_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }
}

impl GetBit<&u8> for u16 {
    #[inline]
    fn get_bit(&self, bit: u8) -> Self {
        *self & (1 << bit)
    }
}

impl GetBit<&u8> for u128 {
    #[inline]
    fn get_bit(&self, bit: u8) -> Self {
        *self & (1 << bit)
    }
}

impl SetBit<&u8> for u16 {
    #[inline]
    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }
}

impl SetBit<&u8> for u128 {
    #[inline]
    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }
}

impl PopBit<&u8> for u16 {
    #[inline]
    fn pop_bit(&mut self) -> u8 {
        let tz = self.trailing_zeros() as u8;
        *self &= *self - 1;
        tz
    }
}

impl PopBit<&u8> for u128 {
    #[inline]
    fn pop_bit(&mut self) -> u8 {
        let tz = self.trailing_zeros() as u8;
        *self &= *self - 1;
        tz
    }
}

impl SwapBits<&u8> for u16 {
    fn swap_bits(&mut self, first: u8, second: u8) {
        let fb = self.get_bit(first);
        let sb = self.get_bit(second);
        *self &= !(fb | sb);
        if fb != 0 {
            self.set_bit(second);
        }
        if sb != 0 {
            self.set_bit(first);
        }
    }
}

impl SwapBits<&u8> for u128 {
    fn swap_bits(&mut self, first: u8, second: u8) {
        let fb = self.get_bit(first);
        let sb = self.get_bit(second);
        *self &= !(fb | sb);
        if fb != 0 {
            self.set_bit(second);
        }
        if sb != 0 {
            self.set_bit(first);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_utils() {
        
        // u16 
        
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

        let ins: [u16; 2] = [0b110001000, 0b111010100];
        let res: [u16; 2] = [0b011100000, 0b111010001];
        for (i, mut val) in ins.into_iter().enumerate() {
            val.swap_bits(0, 2);
            val.swap_bits(3, 5);
            val.swap_bits(6, 8);
            assert_eq!(val, res[i])
        }

        // u128

        let mut val: u128 = 0;
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

        let ins: [u128; 2] = [0b110001000, 0b111010100];
        let res: [u128; 2] = [0b011100000, 0b111010001];
        for (i, mut val) in ins.into_iter().enumerate() {
            val.swap_bits(0, 2);
            val.swap_bits(3, 5);
            val.swap_bits(6, 8);
            assert_eq!(val, res[i])
        }
    }
}