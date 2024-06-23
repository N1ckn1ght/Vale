use super::bitboard::{Bitboard, PopBit, SwapBits};

pub struct Lookups {
    pub diag_lines:    [u16; 2],             // diagonal lines, x00-0x0-00x and 00x-0x0-x00
    pub side_lines:    [u16; 4],             // side lines, such as xxx-000-000, etc.
    pub cent_lines:    [u16; 2],             // center horizontal and vertical lines
    pub corners:       u16,                  // corners of small board, x0x-000-x0x
    pub sides:         u16,                  // sides of small board, 0x0-x0x-0x0
    pub center:        u16,                  // simply center bit, which is 000-0x0-000
    pub large_mirrors: [u16; 512],           // only 9-bit rows (9 bits from 27/32), otherwise it's too large
    pub large_rotates: [[Bitboard; 512]; 9], // only by 9-bit rows (9 bits from 9x9), otherwise it's too large
                                             // these are CLOCKWISE flips! Do 3 of them to get COUNTER-CLOCKWISE. 4 is 0
    pub small_mirrors: [u16; 512],           // mirror full (!) board (3x3)
    pub small_rotates: [u16; 512],           // flip full (!) board (3x3)
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

        let mut large_rotates = [[Bitboard::default(); 512]; 9];
        for i in 0..9 {
            for j in 0..512 {
                let mut bits = j;
                let mut shifted = Bitboard::default();
                while bits != 0 {
                    let bit = bits.pop_bit();
                    shifted.set_bit((bit * 9 + 8 - i) as usize);
                }
                large_rotates[i as usize][j as usize] |= shifted;
            }
        }

        let mut small_mirrors: [u16; 512] = [0; 512];
        for i in 0..512 {
            small_mirrors[i] = i as u16;
            small_mirrors[i].swap_bits(0, 2);
            small_mirrors[i].swap_bits(3, 5);
            small_mirrors[i].swap_bits(6, 8);
        }

        let mut small_rotates: [u16; 512] = [0; 512];
        for i in 0..512 {
            small_rotates[i] = i as u16;
            small_rotates[i].swap_bits(2, 8);
            small_rotates[i].swap_bits(0, 2);
            small_rotates[i].swap_bits(0, 6);
            small_rotates[i].swap_bits(5, 7);
            small_rotates[i].swap_bits(1, 5);
            small_rotates[i].swap_bits(1, 3);
        }

        Self {
            diag_lines: [0b001010100, 0b100010001],
            side_lines: [0b000000111, 0b001001001, 0b100100100, 0b111000000],
            cent_lines: [0b000111000, 0b010010010],
            corners: 0b101000101,
            sides: 0b010101010,
            center: 0b000010000,
            large_mirrors,
            large_rotates,
            small_mirrors,
            small_rotates
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookups_large_mirrors() {
        let initial = Bitboard::init(&[0b101100010000111000110101011, 0b110011001100110011001100110, 0b000000000011111111110000000]);
        let rotated = Bitboard::default();
        for i in 0..9 {
            
        }
    }

    #[test]
    fn lookups_large_rotates() {
        
    }

    #[test]
    fn lookups_small_mirrors() {
        let lookups = Lookups::default();

        let board: u16 = 0b110101001;
        assert_eq!(lookups.small_mirrors[board as usize], 0b011101100);
        assert_eq!(lookups.small_mirrors[lookups.small_mirrors[board as usize] as usize], board);
        let board: u16 = 0;
        assert_eq!(lookups.small_mirrors[board as usize], board);
        let board: u16 = 0b101010101;
        assert_eq!(lookups.small_mirrors[board as usize], board);
    }

    #[test]
    fn lookups_small_rotates() {
        let lookups = Lookups::default();
        
        let board1: u16 = 0b110101001;
        let mut board2 = lookups.small_rotates[board1 as usize];
        assert_eq!(board2, 0b011001110);
        board2 = lookups.small_rotates[board2 as usize];
        assert_eq!(board2, 0b100101011);
        board2 = lookups.small_rotates[board2 as usize];
        assert_eq!(board2, 0b011100110);
        board2 = lookups.small_rotates[board2 as usize];
        assert_eq!(board2, board1);

        let board3: u16 = 0b111111111;
        let board4: u16 = 0b000000000;
        let board5: u16 = 0b101010101;
        assert_eq!(board3, lookups.small_rotates[board3 as usize]);
        assert_eq!(board4, lookups.small_rotates[board4 as usize]);
        assert_eq!(board5, lookups.small_rotates[board5 as usize]);
    }
}