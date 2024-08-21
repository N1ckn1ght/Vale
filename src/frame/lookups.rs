use super::bitboard::SwapBits;

pub const DIV_LOOKUP: [u8; 81] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 
                                  1, 1, 1, 1, 1, 1, 1, 1, 1,
                                  2, 2, 2, 2, 2, 2, 2, 2, 2,
                                  3, 3, 3, 3, 3, 3, 3, 3, 3,
                                  4, 4, 4, 4, 4, 4, 4, 4, 4,
                                  5, 5, 5, 5, 5, 5, 5, 5, 5,
                                  6, 6, 6, 6, 6, 6, 6, 6, 6,
                                  7, 7, 7, 7, 7, 7, 7, 7, 7,
                                  8, 8, 8, 8, 8, 8, 8, 8, 8,
                                 ];

pub const SUB_LOOKUP: [u128; 9] = [0b111111111000000000000000000000000000000000000000000000000000000000000000000000000,
                                   0b000000000111111111000000000000000000000000000000000000000000000000000000000000000,
                                   0b000000000000000000111111111000000000000000000000000000000000000000000000000000000,
                                   0b000000000000000000000000000111111111000000000000000000000000000000000000000000000,
                                   0b000000000000000000000000000000000000111111111000000000000000000000000000000000000,
                                   0b000000000000000000000000000000000000000000000111111111000000000000000000000000000,
                                   0b000000000000000000000000000000000000000000000000000000111111111000000000000000000,
                                   0b000000000000000000000000000000000000000000000000000000000000000111111111000000000,
                                   0b000000000000000000000000000000000000000000000000000000000000000000000000111111111
                                  ];

pub struct GenLookups {
    diag_lines: [u16; 2],    // diagonal lines, x00-0x0-00x and 00x-0x0-x00
    side_lines: [u16; 4],    // side lines, such as xxx-000-000, etc.
    cent_lines: [u16; 2],    // center horizontal and vertical lines
    all_lines:  [u16; 8],    // a bit naive in terms of memory, but to make things simplier later     
    corners:    u16,         // corners of small board, x0x-000-x0x
    sides:      u16,         // sides of small board, 0x0-x0x-0x0
    center:     u16,         // simply center bit, which is 000-0x0-000
    mirrors:    [u16; 512],  // mirror of a sub board (3x3)
    rotates:    [u16; 512],  // rotate of a sub board (3x3, clockwise)
}

impl GenLookups {
    pub fn default() -> Self {
        let diag_lines: [u16; 2] = [0b001010100, 0b100010001];
        let side_lines: [u16; 4] = [0b000000111, 0b001001001, 0b100100100, 0b111000000];
        let cent_lines: [u16; 2] = [0b000111000, 0b010010010];
        let all_lines:  [u16; 8] = [0b001010100, 0b100010001, 0b000000111, 0b001001001, 0b100100100, 0b111000000, 0b000111000, 0b010010010];
        let corners: u16 = 0b101000101;
        let sides:   u16 = 0b010101010;
        let center:  u16 = 0b000010000;

        let mut mirrors: [u16; 512] = [0; 512];
        for (i, bb) in mirrors.iter_mut().enumerate() {
            *bb = i as u16;
            bb.swap_bits(0, 2);
            bb.swap_bits(3, 5);
            bb.swap_bits(6, 8);
        }
        
        let mut rotates: [u16; 512] = [0; 512];
        for (i, bb) in rotates.iter_mut().enumerate() {
            *bb = i as u16;
            bb.swap_bits(2, 8);
            bb.swap_bits(0, 2);
            bb.swap_bits(0, 6);
            bb.swap_bits(5, 7);
            bb.swap_bits(1, 5);
            bb.swap_bits(1, 3);
        }

        Self {
            diag_lines,
            side_lines,
            cent_lines,
            all_lines,
            corners,
            sides,
            center,
            mirrors,
            rotates
        }
    }

    #[inline]
    pub fn get_diag_lines(&self) -> &[u16; 2] {
        &self.diag_lines
    }

    #[inline]
    pub fn get_side_lines(&self) -> &[u16; 4] {
        &self.side_lines
    }

    #[inline]
    pub fn get_cent_lines(&self) -> &[u16; 2] {
        &self.cent_lines
    }
    
    #[inline]
    pub fn get_all_lines(&self) -> &[u16; 8] {
        &self.all_lines
    }
    
    #[inline]
    pub fn get_corners(&self) -> u16 {
        self.corners
    }

    #[inline]
    pub fn get_sides(&self) -> u16 {
        self.sides
    }

    #[inline]
    pub fn get_center(&self) -> u16 {
        self.center
    }

    #[inline]
    pub fn get_mirrors(&self) -> &[u16; 512] {
        &self.mirrors
    }

    #[inline]
    pub fn get_rotates(&self) -> &[u16; 512] {
        &self.rotates
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookups_mirrors() {
        let lookups = GenLookups::default();

        let board: u16 = 0b110101001;
        assert_eq!(lookups.mirrors[board as usize], 0b011101100);
        assert_eq!(lookups.mirrors[lookups.mirrors[board as usize] as usize], board);
        let board: u16 = 0;
        assert_eq!(lookups.mirrors[board as usize], board);
        let board: u16 = 0b101010101;
        assert_eq!(lookups.mirrors[board as usize], board);
    }

    #[test]
    fn lookups_rotates() {
        let lookups = GenLookups::default();
        
        let board1: u16 = 0b110101001;
        let mut board2 = lookups.rotates[board1 as usize];
        assert_eq!(board2, 0b011001110);
        board2 = lookups.rotates[board2 as usize];
        assert_eq!(board2, 0b100101011);
        board2 = lookups.rotates[board2 as usize];
        assert_eq!(board2, 0b011100110);
        board2 = lookups.rotates[board2 as usize];
        assert_eq!(board2, board1);

        let board3: u16 = 0b111111111;
        let board4: u16 = 0b000000000;
        let board5: u16 = 0b101010101;
        assert_eq!(board3, lookups.rotates[board3 as usize]);
        assert_eq!(board4, lookups.rotates[board4 as usize]);
        assert_eq!(board5, lookups.rotates[board5 as usize]);
    }
}