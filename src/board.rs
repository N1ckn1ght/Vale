use super::{bitboard::*, lookups::*};


pub struct Board {
    pub global:   [u16; 3],   // 0b111111111 - board of finished boards, 3rd board means it's draw (9 bits used)
    pub locals:   [u128; 2],  // sub-boards, little-endian, 0 for X, 1 for O (81 bits used)
    pub status:   u8,         // 0 - X won, 1 - O won, 2 - Draw, 3 - Game still on (could be enum, but "status = turn as usize" is used)\
    pub turn:     bool,       // is current move for O?
    pub history:  Vec<u128>,  // board backups: locals[previous_turn] | (mov << 96)
                              //     null moves are not included, use null_move() method to change the turn
}

impl Default for Board {
    // generate empty and ready to play board
    fn default() -> Self {
        Self {
            global: [0; 3],
            locals: [0; 2],
            status: 3,
            turn: false,
            history: Vec::default(),
        }
    }
}

impl Board {
    // todo: make option to specify the next turn due to local board win transformation
    pub fn import(&mut self, ken: &str) {
        self.global = [0; 3];
        self.locals = [0; 2];
        self.status = 3;
        self.turn = false;
        self.history = Vec::default();

        let parts = ken.split('-');
        let mut bit = 81;
        for part in parts.into_iter() {
            for char in part.chars() {
                bit -= 1;
                match char {
                    'x' => {
                        self.locals[0].set_bit(bit);
                        self.turn = !self.turn;
                    },
                    'o' => {
                        self.locals[1].set_bit(bit);
                        self.turn = !self.turn;
                    },
                    _ => {
                        let skip = char.to_digit(10).unwrap() as u8;
                        bit -= skip - 1;
                    }
                }
            }
        }

        for i in 0..9 {
            let mut win_occured = false;

            let localx = SUB_LOOKUP[i as usize] & self.locals[0];
            let subx = ((localx >> (i * 9)) & LS) as u16;
            for mask in WIN_LOOKUP.iter() {
                if subx & mask == *mask {
                    self.global[0].set_bit(i);
                    win_occured = true;
                    break;
                }
            }

            if win_occured {
                continue;
            }

            let localo = SUB_LOOKUP[i as usize] & self.locals[1];
            let subo = ((localo >> (i * 9)) & LS) as u16;
            for mask in WIN_LOOKUP.iter() {
                if subo & mask == *mask {
                    self.global[1].set_bit(i);
                    win_occured = true;
                    break;
                }
            }

            if win_occured {
                continue;
            }

            if subx | subo == SF {
                self.global[2].set_bit(i);
            }
        }

        for i in 0..1 {
            for mask in WIN_LOOKUP.iter() {
                if self.global[i as usize] & mask == *mask {
                    self.status = i;
                    break;
                }
            }
        }

        if self.status > 1 && self.global[0] | self.global[1] | self.global[2] == SF {
            self.status = 2;
        }
    }

    pub fn init(ken: &str) -> Self {
        let mut board = Board::default();
        board.import(ken);
        board
    }

    pub fn generate_legal_moves(&self) -> u128 {
        if self.status < 3 {
            return 0;
        }
    
        let free = !self.locals[0] & !self.locals[1] & LF;

        if self.history.is_empty() {
            return free;
        }
    
        let last_mov = (self.history.last().unwrap() >> 96) as usize;
        let overlap = free & SUB_LOOKUP[MOD_LOOKUP[last_mov] as usize];
    
        if overlap == 0 {
            return free;
        }

        overlap
    }

    pub fn make_move(&mut self, mov: u8) {
        let my_turn = self.turn as usize;

        self.history.push(self.locals[my_turn] | ((mov as u128) << 96));

        self.locals[my_turn].set_bit(mov);

        let gbit = DIV_LOOKUP[mov as usize];
        let my_overlap = self.locals[my_turn] & SUB_LOOKUP[gbit as usize];
        let my_sub = ((my_overlap >> (gbit * 9)) & LS) as u16;

        let mut win_occured = false;
        let cbit = MOD_LOOKUP[mov as usize] as usize;

        if my_sub.count_ones() > 2 {
            for mask in WIN_LOOKUP_INDEXED.iter().skip(WIN_LOOKUP_INDICES[cbit][0]).take(WIN_LOOKUP_INDICES[cbit][1]) {
                if my_sub & mask == *mask {
                    self.global[my_turn].set_bit(gbit);
                    win_occured = true;              
                    break;
                }
            }
        }

        if win_occured {
            let mut global_win_occured = false;

            // I guess if you wanna REAL values of local boards, keep them on the interface level?
            self.locals[my_turn] |= SUB_LOOKUP[gbit as usize];
            
            if self.global[my_turn].count_ones() > 2 {
                for mask in WIN_LOOKUP_INDEXED.iter().skip(WIN_LOOKUP_INDICES[gbit as usize][0]).take(WIN_LOOKUP_INDICES[gbit as usize][1]) {
                    if self.global[my_turn] & mask == *mask {
                        self.status = my_turn as u8;
                        global_win_occured = true;         
                        break;
                    }
                }
            }

            if !global_win_occured {
                if self.global[0] | self.global[1] | self.global[2] == SF {
                    self.status = 2;
                }
            }
        } else {
            let op_turn = !self.turn as usize;
            let op_overlap = self.locals[op_turn] & SUB_LOOKUP[gbit as usize];
            if (my_overlap | op_overlap) & SUB_LOOKUP[gbit as usize] == SUB_LOOKUP[gbit as usize] {
                self.global[2].set_bit(gbit);
            }
        }

        self.turn = !self.turn;
    }

    pub fn null_move(&mut self) {
        self.turn = !self.turn;
    }

    pub fn undo_move(&mut self) {
        self.turn = !self.turn;
        self.locals[self.turn as usize] = self.history.last().unwrap() & LF;
        let mov = (self.history.pop().unwrap() >> 96) as usize;
        self.global[self.turn as usize].del_bit(DIV_LOOKUP[mov]);
        self.global[2].del_bit(DIV_LOOKUP[mov]);
        self.status = 3;
    }

    /* Debug and benchmarking */

    pub fn perft(&mut self, depth: u8) -> u64 {
        let mut free = self.generate_legal_moves();

        if depth == 1 {
            return free.count_ones() as u64;
        }

        let mut cnt = 0;
        while free != 0 {
            self.make_move(free.pop_bit());
            cnt += self.perft(depth - 1);
            self.undo_move();
        }

        cnt
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_make_move() {
        let mut board1 = Board::default();
        let board2 = Board::default();

        board1.make_move(0);
        board1.undo_move();
        board1.make_move(80);
        board1.undo_move();

        assert_eq!(board1.locals[0], board2.locals[0]);
        assert_eq!(board1.locals[1], board2.locals[1]);

        board1.make_move(0);
        board1.make_move(1);
        board1.undo_move();
        board1.undo_move();

        assert_eq!(board1.locals[0], board2.locals[0]);
        assert_eq!(board1.locals[1], board2.locals[1]);
    }

    #[test]
    fn board_move_generation() {
        let mut board = Board::default();

        board.make_move(0);
        assert_eq!(board.locals[0], 1);
        assert_eq!(board.locals[1], 0);
        assert_eq!(board.generate_legal_moves(), 0b111111110);

        board.make_move(1);
        assert_eq!(board.locals[0], 1);
        assert_eq!(board.locals[1], 0b10);
        assert_eq!(board.generate_legal_moves(), 0b111111111000000000);
        board.undo_move();
        board.undo_move();

        board.make_move(40);
        board.make_move(42);
        board.make_move(58);
        board.make_move(43);
        board.make_move(67);
        board.make_move(44);
        board.make_move(76);
        assert_eq!(board.generate_legal_moves(), 0b111101111111101111111101111111111111000000000111111111111111111111111111111111111);

        board.undo_move();
        board.undo_move();
        assert_eq!(board.generate_legal_moves(), 0b100101111000000000000000000000000000000000000);
    }

    #[test]
    fn board_move_generation_perft_pre_local_wins() {
        let mut board = Board::default();
        assert_eq!(board.perft(1), 81);
        assert_eq!(board.perft(2), 720);
        assert_eq!(board.perft(3), 6336);
        assert_eq!(board.perft(4), 55080);
        assert_eq!(board.perft(5), 473256);
    }

    /* NOT VERIFIED ! */
    #[test]
    fn board_move_generation_perft_past_local_wins() {
        let mut board = Board::default();
        assert_eq!(board.perft(6), 4020960);
        assert_eq!(board.perft(7), 33782544);
        // assert_eq!(board.perft(8), 281067408);
    }

    #[test]
    fn board_import() {
        let mut board1 = Board::default();
        board1.make_move(0);
        board1.make_move(1);
        let mut board2 = Board::default();
        board2.import("9-9-9-9-9-9-9-9-7ox");
        // assert_eq!(board1, board2);

        let board = Board::init("9-9-9-9-o3x4-9-9-9-9");
        assert_eq!(board.locals[0], 0b000000000000000000000000000000000000000010000000000000000000000000000000000000000);
        assert_eq!(board.locals[1], 0b000000000000000000000000000000000000100000000000000000000000000000000000000000000);
        assert_eq!(board.global, [0, 0, 0]);
        assert_eq!(board.status, 3);
        assert_eq!(board.turn, false);

        let board = Board::init("2x2x2x-o3o3o-9-9-4ox3-9-9-9-xxoooxxox");
        assert_eq!(board.locals[0], 0b001001001000000000000000000000000000000001000000000000000000000000000000110001101);
        assert_eq!(board.locals[1], 0b000000000100010001000000000000000000000010000000000000000000000000000000001110010);
        assert_eq!(board.global, [0b100000000, 0b010000000, 0b000000001]);
        assert_eq!(board.status, 3);
        assert_eq!(board.turn, true);

        let board = Board::init("2x2x2x-o3o3o-3ooo3-1x2x2x1-x3x3x-ooo6-6ooo-x2x2x2-xoxooxxxo");
        assert_eq!(board.global, [0b100110010, 0b011001100, 0b000000001]);
        assert_eq!(board.status, 2);
        assert_eq!(board.turn, true);
    }
}