use super::{lookups::Lookups};

pub struct Field {
    board:    [u16; 3],   // 0b111111111 - board of finished boards, 3rd board means it's draw
    subs:     [u16; 18],  // Subboards, right-to-left, down-to-up, evens for X, odds for O
    turn:     bool,       // is current move for O?
    history:  Vec<u8>,    // history of made moves, 0xBBBBAAAA - A is the sub index, B is the cell
                          //     null moves are not included, use null_move() method to change the turn
    lookups:  Lookups     // 
}

impl Default for Field {
    fn default() -> Field {
        Field::init("9-9-9-9-9-9-9-9-9")
    }
}

impl Field {
    // pub fn default() -> Field {
        
    //     // Self {
    //     //     bitboards: [Bitboard::default(); 2],
    //     //     secured: [0; 3],
    //     //     turn: false,
    //     //     history: Vec::default(),
    //     //     lookups: Lookups::default()
    //     // }
    // }

    pub fn init(ken: &str) -> Self {
        let mut board = [0; 3];
        let mut subs = [0; 18];
        let mut turn = true;

        let mut parts = ken.split('-');
        for (i, part) in parts.into_iter().enumerate() {
            let mut j = 8;
            for char in part.chars() {
                match char {
                    'x' => {

                    },
                    'o' => {
                        
                    },
                    'X' => {
                        
                        break;
                    },
                    'O' => {

                        break;
                    },
                    '/' => {
                        
                        break;
                    },
                    _ => {
                       j -= char.to_digit(10).unwrap() - 1; 
                    }
                }
                j -= 1;
            }
        }

        Self {
            board,
            subs,
            turn,
            history: Vec::new(),
            lookups: Lookups::default()
        }
    }



//     pub fn make_move(&mut self, mov: u8) {
//         self.bitboards[self.turn as usize].set_bit(mov as usize);
//         self.turn = !self.turn;
//         self.history.push(mov);
//         // if ...
//         // self.secured[self.turn as usize].set_bit(mov / 9);
//     }

//     pub fn null_move(&mut self) {
//         self.turn = !self.turn;
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