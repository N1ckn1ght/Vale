use super::bitboard::{Bitboard, DelBit};

pub struct Field {
    bitboards:  [Bitboard; 2], // bitboards for X and O respectively
    secured:    [u16; 3],      // 0b0000000AAAAAAAAA - board of finished boards, 3rd board means it's draw
    turn:       bool,          // is current move for O?
    history:    Vec<u8>        // history of made moves (null moves are not included as they are unmakeable by themselves)
}

impl Field {
    pub fn default() -> Self {
        Self {
            bitboards: [Bitboard::default(); 2],
            secured: [0; 3],
            turn: false,
            history: Vec::default()
        }
    }

    pub fn make_move(&mut self, mov: u8) {
        self.bitboards[self.turn as usize].set_bit(mov as usize);
        self.turn = !self.turn;
        self.history.push(mov);
        // if ...
        // self.secured[self.turn as usize].set_bit(mov / 9);
    }

    pub fn null_move(&mut self) {
        self.turn = !self.turn;
    }

    pub fn undo_move(&mut self) {
        let mov = self.history.pop().unwrap();
        self.bitboards[self.turn as usize].del_bit(mov as usize);
        self.turn = !self.turn;
        
        let supercell = mov / 9;
        self.secured[self.turn as usize].del_bit(supercell);
        self.secured[2].del_bit(supercell);
    }
}