use super::{bitboard::*, lookups::*};

#[derive(Debug, Clone)]
pub struct Field {
    pub global:   [u16; 3],   // 0b111111111 - board of finished boards, 3rd board means it's draw (9 bits used)
    pub locals:   [u128; 2],  // sub-boards, little-endian, 0 for X, 1 for O (81 bits used)
    pub status:   u8,         // 0 - X won, 1 - O won, 2 - Draw, 3 - Game still on (could be enum, but "status = turn as usize" is used)\
    pub turn:     bool,       // is current move for O?
    pub history:  Vec<u128>,  // field backups: locals[previous_turn] | (mov << 96)
                              //     null moves are not included, use null_move() method to change the turn
}

impl Default for Field {
    // generate an emtpy and ready to play field
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

impl Field {
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
        let mut fd = Field::default();
        fd.import(ken);
        fd
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

impl PartialEq for Field {
    // fast and not precise for debug
    fn eq(&self, other: &Self) -> bool {
        self.locals[0] == other.locals[0] &&
        self.locals[1] == other.locals[1]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_make_move() {
        let mut field1 = Field::default();
        let field2 = Field::default();

        field1.make_move(0);
        field1.undo_move();
        field1.make_move(80);
        field1.undo_move();

        assert_eq!(field1.locals[0], field2.locals[0]);
        assert_eq!(field1.locals[1], field2.locals[1]);

        field1.make_move(0);
        field1.make_move(1);
        field1.undo_move();
        field1.undo_move();

        assert_eq!(field1.locals[0], field2.locals[0]);
        assert_eq!(field1.locals[1], field2.locals[1]);
    }

    #[test]
    fn field_move_generation() {
        let mut fd = Field::default();

        fd.make_move(0);
        assert_eq!(fd.locals[0], 1);
        assert_eq!(fd.locals[1], 0);
        assert_eq!(fd.generate_legal_moves(), 0b111111110);

        fd.make_move(1);
        assert_eq!(fd.locals[0], 1);
        assert_eq!(fd.locals[1], 0b10);
        assert_eq!(fd.generate_legal_moves(), 0b111111111000000000);
        fd.undo_move();
        fd.undo_move();

        assert_eq!(fd.perft(1), 81);
        assert_eq!(fd.perft(2), 720);
        assert_eq!(fd.perft(3), 6336);
    }

    /* NOT VERIFIED ! */
    #[test]
    fn field_move_generation_perft6() {
        let mut fd = Field::default();

        assert_eq!(fd.perft(4), 55080);
        assert_eq!(fd.perft(5), 473256);
        assert_eq!(fd.perft(6), 4017888);
    }

    /* NOT VERIFIED ! */
    #[test]
    fn field_move_generation_perft7() {
        let mut fd = Field::default();

        assert_eq!(fd.perft(7), 33702480);
    }

    #[test]
    fn field_import() {
        let mut fd1 = Field::default();
        fd1.make_move(0);
        fd1.make_move(1);
        let mut fd2 = Field::default();
        fd2.import("9-9-9-9-9-9-9-9-7ox");
        assert_eq!(fd1, fd2);

        let fd = Field::init("9-9-9-9-o3x4-9-9-9-9");
        assert_eq!(fd.locals[0], 0b000000000000000000000000000000000000000010000000000000000000000000000000000000000);
        assert_eq!(fd.locals[1], 0b000000000000000000000000000000000000100000000000000000000000000000000000000000000);
        assert_eq!(fd.global, [0, 0, 0]);
        assert_eq!(fd.status, 3);
        assert_eq!(fd.turn, false);

        let fd = Field::init("2x2x2x-o3o3o-9-9-4ox3-9-9-9-xxoooxxox");
        assert_eq!(fd.locals[0], 0b001001001000000000000000000000000000000001000000000000000000000000000000110001101);
        assert_eq!(fd.locals[1], 0b000000000100010001000000000000000000000010000000000000000000000000000000001110010);
        assert_eq!(fd.global, [0b100000000, 0b010000000, 0b000000001]);
        assert_eq!(fd.status, 3);
        assert_eq!(fd.turn, true);

        let fd = Field::init("2x2x2x-o3o3o-3ooo3-1x2x2x1-x3x3x-ooo6-6ooo-x2x2x2-xoxooxxxo");
        assert_eq!(fd.global, [0b100110010, 0b011001100, 0b000000001]);
        assert_eq!(fd.status, 2);
        assert_eq!(fd.turn, true);
    }
}