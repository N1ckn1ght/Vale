use super::{bitboard::*, lookups::Lookups};

pub struct Field {
    global:     [u16; 3],       // 0b111111111 - board of finished boards, 3rd board means it's draw
    locals:     [[u16; 9]; 2],  // sub-boards, right-to-left, down-to-up, 0 for X, 1 for O
    status:     u8,             // 0 - X won, 1 - O won, 2 - Draw, 3 - Game still on (could be enum, but "status = turn as usize" is used)
    cells_left: u8,             // 0 means game over for draw.
    cl_boarded: [u8; 9],        // moves left per sub board, not counting if it's won by someone
    turn:       bool,           // is current move for O?
    history:    Vec<u8>,        // history of made moves, 0xBBBBAAAA - A is the sub index, B is the cell
                                //     null moves are not included, use null_move() method to change the turn
    lookups:    Lookups         // lookup tables for speeding up calculations, including engine ones (maybe should be brought out of this class?)
}

impl Default for Field {
    // will always generate an emtpy and ready to play field
    fn default() -> Field {
        Self {
            global: [0; 3],
            locals: [[0; 9]; 2],
            status: 3,
            cells_left: 81,
            cl_boarded: [9; 9],
            turn: false,
            history: Vec::default(),
            lookups: Lookups::default()
        }
    }
}

impl Field {
    pub fn import(&mut self, ken: &str) {
        self.global = [0; 3];
        self.locals = [[0; 9]; 2];
        self.cells_left = 0;
        self.cl_boarded = [0; 9];

        let parts = ken.split('-');
        for (i, part) in parts.into_iter().enumerate() {
            let mut j = 9;
            for char in part.chars() {
                j -= 1;
                match char {
                    'x' => {
                        self.locals[0][8 - i].set_bit(j);
                    },
                    'o' => {
                        self.locals[1][8 - i].set_bit(j);
                    },
                    _ => {
                        let free = char.to_digit(10).unwrap() as u8;
                        j -= free - 1;
                        self.cells_left += free;
                        self.cl_boarded[i] += free;
                    }
                }
            }
        }

        self.status = 3;
        self.turn = self.cells_left & 0b1 == 0;

        for i in 0..9 {
            self.local_win_check(false, i);
            self.local_win_check(true, i);
        }
    }

    // will return moves in bitmask format;
    //     vec size 9 in case if move in any sub is possible (bc of start of the game, or by the rule)
    //     vec size 1 in case if move only in the (cell_index) sub is possible
    //     vec size 0 in case if no legal moves are possible (the game has ended)
    pub fn get_legal_moves_raw(&self) -> Vec<u16> {
        if self.status < 3 {
            return vec![];
        }
        if self.history.is_empty() {
            return vec![0b111111111; 9];
        }
        // the rule is to move on sub by previous move cell coordinates
        let board_index = cell_index_extract(*self.history.last().unwrap());
        let board_mask = 1 << board_index;
        if (self.global[0] | self.global[1] | self.global[2]) & board_mask == 0 {
            return vec![!(self.locals[0][board_index as usize] | self.locals[1][board_index as usize]) & 0b111111111];
        }
        vec![
            !(self.locals[0][0] | self.locals[1][0]) & 0b111111111,
            !(self.locals[0][1] | self.locals[1][1]) & 0b111111111,
            !(self.locals[0][2] | self.locals[1][2]) & 0b111111111,
            !(self.locals[0][3] | self.locals[1][3]) & 0b111111111,
            !(self.locals[0][4] | self.locals[1][4]) & 0b111111111,
            !(self.locals[0][5] | self.locals[1][5]) & 0b111111111,
            !(self.locals[0][6] | self.locals[1][6]) & 0b111111111,
            !(self.locals[0][7] | self.locals[1][7]) & 0b111111111,
            !(self.locals[0][8] | self.locals[1][8]) & 0b111111111
        ]
    }

    // will return moves transformed into 0xBBBBAAAA format, NOT RECOMMENDED
    pub fn get_legal_moves_in_format(&self) -> Vec<u8> {
        let mut moves_raw = self.get_legal_moves_raw();
        let mut moves_fmt: Vec<u8> = Vec::new();

        if moves_raw.len() == 1 {
            let sub_index = cell_index_extract(*self.history.last().unwrap());
            while moves_raw[0] != 0 {
                let bit = moves_raw[0].pop_bit();
                moves_fmt.push(pack_move(sub_index, bit));
            }
        } else if moves_raw.len() == 9 {
            for (i, mut sub) in moves_raw.into_iter().enumerate() {
                while sub != 0 {
                    let bit = sub.pop_bit();
                    moves_fmt.push(pack_move(i as u8, bit));
                }
            }
        }

        moves_fmt
    }

    pub fn make_move(&mut self, mov: u8) {
        let board_index = board_index_extract(mov);
        let cell_index = cell_index_extract(mov);

        self.cells_left -= 1;
        self.cl_boarded[board_index as usize] -= 1;
        
        self.locals[self.turn as usize][board_index as usize].set_bit(cell_index);
        self.local_win_check(self.turn, board_index);
        self.history.push(pack_move(board_index, cell_index));

        self.turn = !self.turn;
    }

    pub fn null_move(&mut self) {
        self.turn = !self.turn;
    }

    pub fn undo_move(&mut self) {
        self.turn = !self.turn;
        self.status = 3;

        let mov = self.history.pop().unwrap();
        let board_index = board_index_extract(mov);
        // manual operations instead of bitboard methods here, but for saving just a little bit of computations
        let board_mask = 1 << board_index;
        let cell_mask = 1 << cell_index_extract(mov);

        // revert move on a local board
        self.locals[self.turn as usize][board_index as usize] &= !cell_mask;
        // update independent counter
        self.cl_boarded[board_index as usize] += 1;
        // revert move on a global board, update dependent counter
        if self.global[self.turn as usize] & board_mask != 0 {
            self.global[self.turn as usize] &= !board_mask;
            self.cells_left += self.cl_boarded[board_index as usize];
            return;
        }
        self.global[2] &= !board_mask;
        self.cells_left += 1;
    }

    fn local_win_check(&mut self, turn: bool, index: u8) {
        // a little optimization
        if self.cl_boarded[index as usize] > 6 {
            return;
        }
        for mask in self.lookups.get_all_lines().iter() {
            if mask & self.locals[turn as usize][index as usize] == *mask {
                self.global[turn as usize].set_bit(index);
                self.cells_left -= 9 - (self.locals[0][index as usize] | self.locals[1][index as usize]).count_ones() as u8;
                self.global_win_check(turn);
                return;
            }
        }
        if self.locals[0][index as usize] & self.locals[1][index as usize] == 0b111111111 {
            self.global[2].set_bit(index);
        }
    }

    fn global_win_check(&mut self, turn: bool) {
        for mask in self.lookups.get_all_lines().iter() {
            if mask & self.global[turn as usize] == *mask {
                self.status = turn as u8;
                return;
            }
        }
        self.status = 2 + (self.cells_left != 0) as u8;
    }

    /* Getters */

    #[inline]
    pub fn get_global_board(&self) -> &[u16; 3] {
        &self.global
    }

    #[inline]
    pub fn get_local_boards(&self) -> &[[u16; 9]; 2] {
        &self.locals
    }

    #[inline]
    pub fn get_status(&self) -> u8 {
        self.status
    }

    #[inline]
    pub fn get_cells_left(&self) -> u8 {
        self.cells_left
    }

    #[inline]
    pub fn get_turn(&self) -> bool {
        self.turn
    }

    #[inline]
    pub fn get_cells_left_per_local_board(&self) -> &[u8; 9] {
        &self.cl_boarded
    }

    #[inline]
    pub fn get_lookups(&self) -> &Lookups {
        &self.lookups
    }

    /* Bench and debug */

    pub fn perft(&mut self, depth: u8) -> u64 {
        let mut lms = self.get_legal_moves_raw();
        if depth == 1 {
            let mut cnt = 0;
            for board in lms.iter() {
                cnt += board.count_ones();
            }
            return cnt as u64;
        }

        let mut cnt = 0;
        if lms.len() == 1 {
            let board_index = cell_index_extract(*self.history.last().unwrap());
            while lms[0] != 0 {
                let bit = lms[0].pop_bit();
                self.make_move(pack_move(board_index, bit));
                cnt += self.perft(depth - 1);
                self.undo_move();
            }
        } else if lms.len() == 9 {
            for (board_index, mut board) in lms.into_iter().enumerate() {
                while board != 0 {
                    let bit = board.pop_bit();
                    self.make_move(pack_move(board_index as u8, bit));
                    cnt += self.perft(depth - 1);
                    self.undo_move();
                }
            }
        }
        cnt
    }
    
    pub fn perft_verbosed(&mut self) {
        
    }
}

#[inline]
fn cell_index_extract(mov: u8) -> u8 {
    (mov >> 4) & 0b1111
}

#[inline]
fn board_index_extract(mov: u8) -> u8 {
    mov & 0b1111
}

#[inline]
fn pack_move(board_index: u8, cell_index: u8) -> u8 {
    (cell_index << 4) | board_index
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_import() {
        let mut fd = Field::default();
        let fd2 = Field::default();
        
        fd.import("9-9-9-9-9-9-9-9-9");
        assert_eq!(fd.get_cells_left(), fd2.get_cells_left());
        assert_eq!(fd.get_cells_left(), 81);
        assert_eq!(fd.get_status(), fd2.get_status());
        assert_eq!(fd.get_status(), 3);
        assert_eq!(fd.get_turn(), fd2.get_turn());
        assert_eq!(fd.get_turn(), false);
        assert_eq!(*fd.get_cells_left_per_local_board(), *fd2.get_cells_left_per_local_board());
        assert_eq!(*fd.get_cells_left_per_local_board(), [9; 9]);
        assert_eq!(*fd.get_global_board(), *fd2.get_global_board());
        assert_eq!(*fd.get_global_board(), [0; 3]);
        assert_eq!(*fd.get_local_boards(), *fd2.get_local_boards());
        assert_eq!(*fd.get_local_boards(), [[0; 9]; 2]);
        
        fd.import("2x2x2x-o3o3o-9-9-4ox3-9-9-9-xxoooxxox");
        assert_eq!(fd.get_cells_left(), 52);
        assert_eq!(fd.get_status(), 3);
        assert_eq!(fd.get_turn(), true);
        assert_eq!(*fd.get_cells_left_per_local_board(), [0, 9, 9, 9, 7, 9, 9, 6, 6]);
        assert_eq!(*fd.get_global_board(), [0b100000000, 0b010000000, 0b000000001]);
        assert_eq!(*fd.get_local_boards(), [[0b110001101, 0, 0, 0, 0b1000, 0, 0, 0, 0b1001001], [0b1110010, 0, 0, 0, 0b10000, 0, 0, 0b100010001, 0]]);
    }
}