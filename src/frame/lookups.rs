use super::bitboard::{Bitboard, PopBit};

pub struct Lookups {
    diag_lines:    [u16; 2],             // diagonal lines, x00-0x0-00x and 00x-0x0-x00
    side_lines:    [u16; 4],             // side lines, such as xxx-000-000, etc.
    cent_lines:    [u16; 2],             // center horizontal and vertical lines
    corners:       u16,                  // corners of small board, x0x-000-x0x
    sides:         u16,                  // sides of small board, 0x0-x0x-0x0
    center:        u16,                  // simply center bit, which is 000-0x0-000
    large_mirrors: [u16; 512],           // only 9-bit rows (9 bits from 27/32), otherwise it's too large
    large_flips:   [[Bitboard; 512]; 9], // only by 9-bit rows (9 bits from 9x9), otherwise it's too large
                                         // these are CLOCKWISE flips! Do 3 of them to get COUNTER-CLOCKWISE. 4 is 0
    small_mirrors: [u16; 512],           // mirror full (!) board (3x3)
    small_flips:   [u16; 512],           // flip full (!) board (3x3)
                                         // these are CLOCKWISE flips! Do 3 of them to get COUNTER-CLOCKWISE. 4 is 0
}

impl Lookups {
    pub fn default() -> Self {
        let mut large_mirrors = [0; 512];
        for i in 0..16 {
            let mut bits = i;
            let mut shifted = 0;
            while bits != 0 {
                let bit = bits.pop_bit();
                shifted |= 1 << (8 - bit);
            }
            large_mirrors[i as usize] = shifted;
            large_mirrors[(i | 16) as usize] = shifted | 16;
            large_mirrors[(shifted | 16) as usize] = i | 16;
            large_mirrors[shifted as usize] = i;
        }

        let mut large_flips = [[Bitboard::default(); 512]; 9];
        for i in 0..9 {
            for j in 0..512 {
                let mut bits = j;
                let mut shifted = Bitboard::default();
                while bits != 0 {
                    let bit = bits.pop_bit();
                    shifted.set_bit(1 << (bit * 9 + 8 - i));
                }
                large_flips[i as usize][j as usize] |= shifted;
            }
        }

        let mut small_mirrors = [0; 512];
        for i in 0..512 {

        }

        let mut small_flips = [0; 512];
        for i in 0..512 {

        }

        Self {
            diag_lines: [0b001010100, 0b100010001],
            side_lines: [0b000000111, 0b001001001, 0b100100100, 0b111000000],
            cent_lines: [0b000111000, 0b010010010],
            corners: 0b101000101,
            sides: 0b010101010,
            center: 0b000010000,
            large_mirrors,
            large_flips,
            small_mirrors,
            small_flips
        }
    }
}