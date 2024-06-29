use super::{bitboard::SetBit, lookups::Lookups};

pub struct Field {
    board:    [u16; 3],      // 0b111111111 - board of finished boards, 3rd board means it's draw
    subs:    [[u16; 9]; 2],  // Subboards, right-to-left, down-to-up, 0 for X, 1 for O
    turn:     bool,          // is current move for O?
    history:  Vec<u8>,       // history of made moves, 0xBBBBAAAA - A is the sub index, B is the cell
                             //     null moves are not included, use null_move() method to change the turn
    lookups:  Lookups        // 
}

impl Default for Field {
    fn default() -> Field {
        Field::init("9-9-9-9-9-9-9-9-9")
    }
}

impl Field {
    pub fn init(ken: &str) -> Self {
        let mut board = [0; 3];
        let mut subs = [[0; 9]; 2];
        let mut turn = false;

        let parts = ken.split('-');
        for (i, part) in parts.into_iter().enumerate() {
            let mut j = 8;
            for char in part.chars() {
                match char {
                    'x' => {
                        subs[0][8 - i].set_bit(j);
                        turn = !turn;
                    },
                    'o' => {
                        subs[1][8 - i].set_bit(j);
                        turn = !turn;
                    },
                    _ => {
                       j -= char.to_digit(10).unwrap() as u8 - 1; 
                    }
                }
                j -= 1;
            }
        }

        // todo: check for win condition

        Self {
            board,
            subs,
            turn,
            history: Vec::default(),
            lookups: Lookups::default()
        }
    }

    pub fn make_move(&mut self, mov: u8) {
        
    }

    pub fn null_move(&mut self) {
        self.turn = !self.turn;
    }

    pub fn undo_move(&mut self) {
        
    }

//     pub fn make_move(&mut self, mov: u8) {
//         self.bitboards[self.turn as usize].set_bit(mov as usize);
//         self.turn = !self.turn;
//         self.history.push(mov);
//         // if ...
//         // self.secured[self.turn as usize].set_bit(mov / 9);
//     }
//     pub fn undo_move(&mut self) {
//         let mov = self.history.pop().unwrap();
//         self.bitboards[self.turn as usize].del_bit(mov as usize);
//         self.turn = !self.turn;
//         let supercell = mov / 9;
//         self.secured[self.turn as usize].del_bit(supercell);
//         self.secured[2].del_bit(supercell);
//     }

}