use super::{bitboard::*, lookups::*};

pub struct Field {
    global:   [u16; 3],   // 0b111111111 - board of finished boards, 3rd board means it's draw (9 bits used)
    locals:   [u128; 2],  // sub-boards, right-to-left, down-to-up, 0 for X, 1 for O (81 bits used)
    status:   u8,         // 0 - X won, 1 - O won, 2 - Draw, 3 - Game still on (could be enum, but "status = turn as usize" is used)\
    turn:     bool,       // is current move for O?
    history:  Vec<u128>,  // field backups: locals[previous_turn] | (mov << 96)
                          //     null moves are not included, use null_move() method to change the turn
    lookups:  GenLookups  // lookup tables for speeding up calculations, including engine ones (maybe should be brought out of this class?)
}

impl Default for Field {
    // generate an emtpy and ready to play field
    fn default() -> Field {
        Self {
            global: [0; 3],
            locals: [0; 2],
            status: 3,
            turn: false,
            history: Vec::default(),
            lookups: GenLookups::default()
        }
    }
}

impl Field {
    pub fn generate_legal_moves(&self) -> u128 {
        if self.status < 3 {
            return 0;
        }
        
        let free = !self.locals[0] & !self.locals[1] & LF;

        if self.history.is_empty() {
            return free;
        }
        
        let last_mov = (self.history.last().unwrap() >> 96) as usize;
        let overlap = SUB_LOOKUP[DIV_LOOKUP[last_mov] as usize] & free;
        if overlap != 0 {
            return overlap;
        }

        free
    }

    pub fn make_move(&mut self, mov: u8) {
        let my_turn = self.turn as usize;

        self.history.push(self.locals[my_turn] | ((mov as u128) << 96));

        self.locals[my_turn].set_bit(mov);

        let gbit = DIV_LOOKUP[mov as usize];
        let my_overlap = self.locals[my_turn] & SUB_LOOKUP[gbit as usize];
        let my_sub = ((my_overlap >> (gbit * 9)) & LS) as u16;

        let mut win_occured = false;

        if my_sub.count_ones() > 2 {
            for mask in self.lookups.get_all_lines().iter() {
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
                for mask in self.lookups.get_all_lines().iter() {
                    if self.global[my_turn] & mask == *mask {
                        self.status = my_turn as u8;
                        global_win_occured = true;         
                        break;
                    }
                }
            }

            if !global_win_occured {
                let op_turn = !self.turn as usize;
                if self.global[my_turn] | self.global[op_turn] == 0b111111111 {
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

    // #[test]
    // fn field_import() {

    // }
}