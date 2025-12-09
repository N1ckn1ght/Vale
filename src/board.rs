use super::{bitboard::*, lookups::*};

pub const MOV_CAP: usize = 82;
pub const ERR_MOV: u8 = 128;


pub struct Board {
    pub global:   [u16; 3],   // sub-boards completion (or The Global Board)
                              // 0 - X won, 1 - O won, 2 - Draw on board (9 bits used)
    pub locals:   [u128; 2],  // sub-boards, little-endian for subs (look at move transform methods)
                              // 0 - X, 1 - O (81 bits used)
    pub status:   u8,         // 0 - X won, 1 - O won, 2 - Draw, 3 - Game still on
    pub turn:     bool,       // 0 - X to move, 1 - O to move
    pub history:  Vec<u128>,  // board backups: locals[previous_turn]
    pub moves:    Vec<u8>,    // complete move history
    pub lwbits:   u128        // local win board bits, to apply mask to generate legal moves fast when it's free board move
                              // includes !LF mask in itself as a part of optimization
}

impl Default for Board {
    // generate empty and ready to play board
    fn default() -> Self {
        Self {
            global: [0; 3],
            locals: [0; 2],
            status: 3,
            turn: false,
            history: Vec::with_capacity(MOV_CAP),
            moves: Vec::with_capacity(MOV_CAP),
            lwbits: !LF
        }
    }
}

impl Board {
    pub fn clear(&mut self) {
        self.global = [0; 3];
        self.locals = [0; 2];
        self.status = 3;
        self.turn = false;
        self.history = Vec::with_capacity(MOV_CAP);
        self.moves = Vec::with_capacity(MOV_CAP);
        self.lwbits = !LF;
    }

    pub fn import_ken(&mut self, ken: &str) {
        self.clear();
        let mut iter = ken.split(" ");
        let sup1 = iter.next().unwrap_or("9-9-9-9-9-9-9-9-9");
        let sup2 = iter.next().unwrap_or("-");
        let parts = sup1.split('-');
        let mut bit = 0;
        for part in parts.into_iter() {
            for char in part.chars() {
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
                        bit += skip - 1;
                    }
                }
                bit += 1;
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
                self.lwbits |= SUB_LOOKUP[i as usize];
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
                self.lwbits |= SUB_LOOKUP[i as usize];
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

        if sup2 != "-" {
            let last_mov = transform_move(sup2, !0); 
            if last_mov != ERR_MOV {
                self.moves.push(last_mov);
                // unused?
                let mut prev = self.locals[!self.turn as usize];
                prev.del_bit(last_mov);
                self.history.push(prev | ((last_mov as u128) << 96));
            }
        }
    }

    pub fn import_history(&mut self, moves: &str) {
        self.clear();
        let parts = moves.split_whitespace();
        let mut hist = String::new();
        for part in parts.into_iter() {
            if part.ends_with('.') {
                continue;
            }
            let mov = transform_move(part, self.generate_legal_moves());
            if mov == ERR_MOV {
                println!("#DEBUG Illegal move: {}\n#DEBUG Applied sequence: {}", part, hist);
                break;
            }
            hist += part;
            hist += " ";
            self.make_move(mov);
        }
    }

    pub fn export_ken(&self) -> String {
        let mut ken = String::new();
        let mut empty = 0;
        for bit in 0..81 {
            let xbit = self.locals[0].get_bit(bit);
            let obit = self.locals[1].get_bit(bit);
            if xbit != 0 {
                if empty != 0 {
                    ken += &((b'1' + empty - 1) as char).to_string();
                    empty = 0;
                }
                ken += "x";
            } else if obit != 0 {
                if empty != 0 {
                    ken += &((b'1' + empty - 1) as char).to_string();
                    empty = 0;
                }
                ken += "o";
            } else {
                empty += 1;
            }
            if MOD_LOOKUP[bit as usize] > 7 {
                if empty != 0 {
                    ken += &((b'1' + empty - 1) as char).to_string();
                    empty = 0;
                }
                if bit != 80 {
                    ken += "-";
                }
            }
        }
        ken += " ";
        if self.moves.is_empty() {
            ken += "-";
        } else {
            ken += &transform_move_back(*self.moves.last().unwrap());
        }
        ken
    }

    pub fn export_history(&self, format: u8) -> String {
        /* format 0 - string of moves with no dividers
           format 1 - pgn-like format
           format 2 - pgn-like format separated by \n every full move */
        if self.moves.is_empty() {
            println!("#DEBUG No moves were made.");
            return "".to_string();
        }
        if self.history[0] != 0 {
            println!("#DEBUG Cannot export REAL move history: game was imported by ken, no initial move history available!");
        }
        let mut history = String::new();
        let mut cnt: u16 = 2;
        if (self.moves.len() as u16 + self.turn as u16).get_bit(0) != 0 {
            cnt += 1;
            if format != 0 {
                history += "1. ... ";
            }
        } 
        for mov in self.moves.iter() {
            if format != 0 && cnt.get_bit(0) == 0 {
                history += &format!("{}. ", cnt / 2);
            }
            history += &transform_move_back(*mov);
            if format == 2 && cnt.get_bit(0) != 0 {
                history += "\n";
            } else {
                history += " ";
            }
            cnt += 1;
        }
        history.pop();
        history
    }

    pub fn generate_legal_moves(&self) -> u128 {
        if self.status < 3 {
            return 0;
        }

        let free = !(self.locals[0] | self.locals[1] | self.lwbits);

        if self.moves.is_empty() {
            return free;
        }
        let overlap = free & SUB_LOOKUP[MOD_LOOKUP[*self.moves.last().unwrap() as usize] as usize];

        if overlap != 0 {
            return overlap;
        }
        free
    }

    pub fn make_move(&mut self, mov: u8) {
        let my_turn = self.turn as usize;

        self.history.push(self.locals[my_turn]);
        self.moves.push(mov);
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

            self.lwbits |= SUB_LOOKUP[gbit as usize];

            if self.global[my_turn].count_ones() > 2 {
                for mask in WIN_LOOKUP_INDEXED.iter().skip(WIN_LOOKUP_INDICES[gbit as usize][0]).take(WIN_LOOKUP_INDICES[gbit as usize][1]) {
                    if self.global[my_turn] & mask == *mask {
                        self.status = my_turn as u8;
                        global_win_occured = true;         
                        break;
                    }
                }
            }

            if !global_win_occured && self.global[0] | self.global[1] | self.global[2] == SF {
                self.status = 2;
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

    pub fn undo_move(&mut self) {
        self.turn = !self.turn;
        let mov = self.moves.pop().unwrap();
        self.locals[self.turn as usize] = self.history.pop().unwrap();
        let dl = DIV_LOOKUP[mov as usize];
        self.global[self.turn as usize].del_bit(dl);
        self.global[2].del_bit(dl);
        self.status = 3;
        self.lwbits &= !SUB_LOOKUP[dl as usize];
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

pub fn transform_move(user_mov: &str, legals: u128) -> u8 {
    let chars = user_mov.trim().chars();
    if chars.count() != 2 {
        println!("#DEBUG Wrong user input: must be from a1 to i9 (expected 2 chars)");
        return ERR_MOV;
    }
    let mut chars = user_mov.trim().chars();
    let file = chars.next().unwrap().to_ascii_lowercase();
    let rank = chars.next().unwrap();
    if !(('a'..='i').contains(&file) && ('1'..='9').contains(&rank)) {
        println!("#DEBUG Wrong user input: must be from a1 to i9");
        return ERR_MOV;
    }
    let realbit = grb((rank as u32 - '1' as u32) as u8, (file as u32 - 'a' as u32) as u8);
    if legals.get_bit(realbit) == 0 {
        println!("#DEBUG Wrong user input: illegal move");
        return ERR_MOV;
    }
    realbit
}

pub fn transform_move_back(mov: u8) -> String {
    if mov > 80 {
        return "??".to_string();
    }
    let gdv = mov / 27;
    let gmd = mov % 27;
    let ldv = gmd / 9;
    let lmd = gmd % 9;
    let rank = gdv * 3 + lmd / 3;  // 0..8 -> '1'..'9'
    let file = ldv * 3 + lmd % 3;  // 0..8 -> 'a'..'i'
    format!("{}{}", (b'a' + file) as char, (b'1' + rank) as char)
}

#[inline]
pub fn grb(rank: u8, file: u8) -> u8 {
    (rank / 3) * 27 + (rank % 3) * 3 + (file / 3) * 9 + (file % 3)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_transform_move() {
        assert_eq!(transform_move("a1", !0), 0);
        assert_eq!(transform_move("b1", !0), 1);
        assert_eq!(transform_move("a2", !0), 3);
        assert_eq!(transform_move("g1", !0), 18);
        assert_eq!(transform_move("c6", !0), 35);
        assert_eq!(transform_move("i9", 1 << 80), 80);
        assert_eq!(transform_move("i9", 0), ERR_MOV);
        assert_eq!(transform_move_back(0), "a1");
        assert_eq!(transform_move_back(1), "b1");
        assert_eq!(transform_move_back(3), "a2");
        assert_eq!(transform_move_back(18), "g1");
        assert_eq!(transform_move_back(35), "c6");
        assert_eq!(transform_move_back(80), "i9");
    }

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
    fn board_generate_moves() {
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
    fn board_perft_pre_local_wins() {
        let mut board = Board::default();
        assert_eq!(board.perft(1), 81);
        assert_eq!(board.perft(2), 720);
        assert_eq!(board.perft(3), 6336);
        assert_eq!(board.perft(4), 55080);
        assert_eq!(board.perft(5), 473256);
    }

    /* NOT VERIFIED ! */
    #[test]
    fn board_perft_past_local_wins() {
        let mut board = Board::default();
        assert_eq!(board.perft(6), 4020960);
        assert_eq!(board.perft(7), 33782544);
        // assert_eq!(board.perft(8), 281067408);
    }

    #[test]
    fn board_import_export() {
        let mut board1 = Board::default();
        board1.make_move(0);
        board1.make_move(1);
        let mut board2 = Board::default();
        board2.import_ken("xo7-9-9-9-9-9-9-9-9 b1");
        assert_eq!(board1.generate_legal_moves(), board2.generate_legal_moves());
        assert_eq!(board1.locals[0], board2.locals[0]);
        assert_eq!(board1.locals[1], board2.locals[1]);
        assert_eq!(board1.lwbits, board2.lwbits);

        board1.clear();
        board1.import_history("e5 d6 b8 e6 e8 f6 h8");
        assert_eq!(board1.generate_legal_moves(), 0b111101111111101111111101111111111111000000000111111111111111111111111111111111111);

        board1.import_history("1. e5 d6 2. b8 e6 3. e8 f6 4. h8 a9 5. a7 a1 6. a2 a6 7. b9 e7 8. e1 d3 9. c9");
        board2.import_ken("o2x5-1x4o2-9-6o2-4x1ooo-9-x3x1oxx-1o2x4-4x4 c9");
        assert_eq!(board1.generate_legal_moves(), board2.generate_legal_moves());
        assert_eq!(board1.locals[0], board2.locals[0]);
        assert_eq!(board1.locals[1], board2.locals[1]);
        assert_eq!(board1.global[0], board2.global[0]);
        assert_eq!(board1.global[1], board2.global[1]);
        assert_eq!(board1.global[2], board2.global[2]);

        board1.import_ken("9-9-9-9-4x3o-9-9-9-9");
        assert_eq!(board1.locals[0], 0b000000000000000000000000000000000000000010000000000000000000000000000000000000000);
        assert_eq!(board1.locals[1], 0b000000000000000000000000000000000000100000000000000000000000000000000000000000000);
        assert_eq!(board1.global, [0, 0, 0]);
        assert_eq!(board1.status, 3);
        assert_eq!(board1.turn, false);

        board1.import_ken("xoxxoooxx-9-9-9-3xo4-9-9-o3o3o-x2x2x2");
        assert_eq!(board1.locals[0], 0b001001001000000000000000000000000000000001000000000000000000000000000000110001101);
        assert_eq!(board1.locals[1], 0b000000000100010001000000000000000000000010000000000000000000000000000000001110010);
        assert_eq!(board1.global, [0b100000000, 0b010000000, 0b000000001]);
        assert_eq!(board1.status, 3);
        assert_eq!(board1.turn, true);

        board1.import_ken("oxxxooxox-2x2x2x-ooo6-6ooo-x3x3x-1x2x2x1-3ooo3-o3o3o-x2x2x2");
        assert_eq!(board1.global, [0b100110010, 0b011001100, 0b000000001]);
        assert_eq!(board1.status, 2);
        assert_eq!(board1.turn, true);

        board1.clear();
        for _ in 0..20 {
            let mut leg = board1.generate_legal_moves();
            board1.make_move(leg.pop_bit());
        }
        for i in 0..8 {
            let hist = board1.export_history(i);
            board2.import_history(&hist);
            assert_eq!(board1.generate_legal_moves(), board2.generate_legal_moves());
            assert_eq!(board1.locals[0], board2.locals[0]);
            assert_eq!(board1.locals[1], board2.locals[1]);
            assert_eq!(board1.global[0], board2.global[0]);
            assert_eq!(board1.global[1], board2.global[1]);
            assert_eq!(board1.global[2], board2.global[2]);
            for _ in 0..5 {
                let mut leg = board1.generate_legal_moves();
                let bit = leg.pop_bit();
                board1.make_move(bit);
                board2.make_move(bit);
                assert_eq!(board1.generate_legal_moves(), board2.generate_legal_moves());
                assert_eq!(board1.locals[0], board2.locals[0]);
                assert_eq!(board1.locals[1], board2.locals[1]);
                assert_eq!(board1.global[0], board2.global[0]);
                assert_eq!(board1.global[1], board2.global[1]);
                assert_eq!(board1.global[2], board2.global[2]);
            }
            for _ in 0..5 {
                board1.undo_move();
                board2.undo_move();
                assert_eq!(board1.generate_legal_moves(), board2.generate_legal_moves());
                assert_eq!(board1.locals[0], board2.locals[0]);
                assert_eq!(board1.locals[1], board2.locals[1]);
                assert_eq!(board1.global[0], board2.global[0]);
                assert_eq!(board1.global[1], board2.global[1]);
                assert_eq!(board1.global[2], board2.global[2]);
            }
            board2.clear();
        }
        board2.import_ken(&board1.export_ken());
        assert_eq!(board1.generate_legal_moves(), board2.generate_legal_moves());
        assert_eq!(board1.locals[0], board2.locals[0]);
        assert_eq!(board1.locals[1], board2.locals[1]);
        assert_eq!(board1.global[0], board2.global[0]);
        assert_eq!(board1.global[1], board2.global[1]);
        assert_eq!(board1.global[2], board2.global[2]);
    }
}