use std::{cmp::{max, Reverse}, time::Instant};
use once_cell::sync::Lazy;
use crate::{bitboard::{GetBit, PopBit}, board::{Board, transform_move_back}, interface::format_eval, lookups::{DIV_LOOKUP, MOD_LOOKUP, SUB_LOOKUP, WIN_LOOKUP}, weights::gen_local_scores};


// comms
const NODES_BETWEEN_UPDATES: u64 = 2048;

// search aux
const PLY_LIMIT: usize = 81;
const INF: i16 = 24576;
pub const LARGE: i16 = 16384;
pub const LARGM: i16 = LARGE - 82;

pub static LOCAL_SCORES: Lazy<(Box<[i8]>, Box<[i8]>)> = Lazy::new(|| {
    let mut x = vec![0i8; 262144];
    let mut o = vec![0i8; 262144];
    gen_local_scores(&mut x, &mut o);
    (x.into_boxed_slice(), o.into_boxed_slice())
});


pub struct Engine {
    /* Search trackers */
    ts:       Instant,                       // timer start
    tl:       u128,                          // time limit in ms
    abort:    bool,                          // stop search signal
    nodes:    u64,                           // nodes searched
    ply:      usize,                         // current distance to the search root
    
    tpv:      [[u8; PLY_LIMIT]; PLY_LIMIT],  // triangular table of a principal variation
    tpv_len:  [usize; PLY_LIMIT],            // current length of tpv
    td:       i8,                            // current target depth
    mate:     bool,                          // is mate detected

    /* Interface related */
    pub post: bool,
    pub evm:  bool
}

impl Default for Engine {
    // generate empty and ready to play board
    fn default() -> Self {
        Self {
            ts: Instant::now(),
            tl: 0,
            abort: false,
            nodes: 0,
            ply: 0,
            tpv: [[0; PLY_LIMIT]; PLY_LIMIT],
            tpv_len: [0; PLY_LIMIT],
            td: 0,
            mate: false,
            post: true,
            evm: false
        }
    }
}

impl Engine {
    pub fn update(&mut self) {
        if self.ts.elapsed().as_millis() > self.tl {
            self.abort = true;
        }

        // add comms here
    }

    // pass board clone
    pub fn search(&mut self, board: &mut Board, time_limit_ms: Option<u128>, depth_limit: Option<usize>) -> (u8, i16) {
        self.ts = Instant::now();
        self.tl = time_limit_ms.unwrap_or(31_536_000_000);
        let dl = (depth_limit.unwrap_or(PLY_LIMIT) & !1).clamp(1, PLY_LIMIT) as i8;  // must be an even number
        self.abort = false;
        self.nodes = 0;
        self.ply = 0;

        for line in self.tpv.iter_mut() {
            for node in line.iter_mut() {
                *node = 0;
            }
        }
        for len in self.tpv_len.iter_mut() {
            *len = 0;
        }

        let alpha = -INF;
        let beta  =  INF;
        let mut score =  0;
        self.td = 2;

        loop {
            let temp = self.negamax(board, alpha, beta, self.td);
            if !self.abort {
                score = temp;
                if score > LARGM {
                    if !self.mate {
                        println!("#DEBUG\tMate detected.");
                        self.mate = true;
                    } else {
                        self.abort = true;
                    }
                }
                if board.turn {
                    score = -score;  // maybe should be not there?
                }
            } else {
                println!("#DEBUG\tAbort signal reached.");
                break;
            }
            self.post(score);

            self.td += 2;
            if self.td > dl || self.ts.elapsed().as_millis() > self.tl {
                break;
            }
        }

        println!("#DEBUG  Time spent: {} ms", self.ts.elapsed().as_millis());
        (self.tpv[0][0], score)
    }

    pub fn negamax(&mut self, board: &mut Board, mut alpha: i16, beta: i16, depth: i8) -> i16 {
        if self.nodes & NODES_BETWEEN_UPDATES == 0 {
            self.update();
        }

        self.nodes += 1;
        self.tpv_len[self.ply] = self.ply;

        match board.status {
            3 => {},
            2 => { return 0; },
            _ => { return -LARGE + self.ply as i16 }
        }

        let mut legals = board.generate_legal_moves();

        if depth == 0 || self.ply > PLY_LIMIT {
            if board.turn {
                return -eval(board);
            }
            return eval(board);
        }

        // pre-sort on eval when it makes sense, so if depth > 1
        if depth > 1 {
            let total_moves = legals.count_ones();
            let mut deep_moves = 2;
            let mut zugs = 0;
            let mut presort: Vec<(u8, i16)> = Vec::with_capacity(total_moves as usize);
            while legals != 0 {
                let bit = legals.pop_bit();
                board.make_move(bit);
                let mut bscore = eval(board);
                board.undo_move();
                
                if bit == self.tpv[0][self.ply] {
                    // principal variation goes first
                    bscore += LARGE;
                    deep_moves += 1;
                } else if DIV_LOOKUP[bit as usize] == MOD_LOOKUP[bit as usize] {
                    // "anchor" move should be looked into as well (the bit is not guaranteed to be empty)
                    bscore += LARGE >> 2;
                    deep_moves += 1;
                } else if !board.moves.is_empty() {  // && DIV_LOOKUP[*board.moves.last().unwrap() as usize] == MOD_LOOKUP[bit as usize] {
                    let gbit = DIV_LOOKUP[*board.moves.last().unwrap() as usize];
                    if gbit == MOD_LOOKUP[bit as usize] && board.global[0].get_bit(gbit) == 0 && board.global[1].get_bit(gbit) == 0 && board.global[2].get_bit(gbit) == 0 {
                        // move that sends opponent into Zugswang should be looked into as well (the bit is not guaranteed to be emtpy)
                        bscore += LARGE >> 1;
                        deep_moves += 1;
                        zugs += 1;
                    }
                }
                presort.push((bit, bscore));
            }
            presort.sort_by_key(|&(_, score)| Reverse(score));

            if zugs > 2 {
                deep_moves -= 1;
            }

            for (i, (mov, _)) in presort.iter().enumerate() {
                self.ply += 1;
                board.make_move(*mov);

                let mut score = if depth < 3 || i < deep_moves {
                    alpha + 1  // force recheck as if LMR failed
                } else if i < (deep_moves + 2) {
                    -self.negamax(board, -beta, -alpha, depth - 2)
                } else {
                    -self.negamax(board, -beta, -alpha, depth - 2 - (depth > 3) as i8 - ((depth > 4) && (i > 18)) as i8)
                };

                if score > alpha {
                    score = -self.negamax(board, -alpha - 1, -alpha, depth - 1);
                    if score > alpha && score < beta {
                        score = -self.negamax(board, -beta, -alpha, depth - 1);
                    }
                }

                board.undo_move();
                self.ply -= 1;

                if self.abort {
                    return 0;  // time limit exceed
                }

                if score > alpha {
                    alpha = score;

                    self.tpv[self.ply][self.ply] = *mov;
                    let mut next = self.ply + 1;
                    while next < self.tpv_len[self.ply + 1] {
                        self.tpv[self.ply][next] = self.tpv[self.ply + 1][next];
                        next += 1;
                    }
                    self.tpv_len[self.ply] = self.tpv_len[self.ply + 1];

                    if alpha >= beta {
                        return beta;  // fail high, opponent will not choose the branch led to this move
                    }
                }
            }
        } else {
            while legals != 0 {
                let bit = legals.pop_bit();

                self.ply += 1;
                board.make_move(bit);

                let score = -self.negamax(board, -beta, -alpha, depth - 1);

                board.undo_move();
                self.ply -= 1;

                if self.abort {
                    return 0;  // time limit exceed
                }

                if score > alpha {
                    alpha = score;

                    self.tpv[self.ply][self.ply] = bit;
                    let mut next = self.ply + 1;
                    while next < self.tpv_len[self.ply + 1] {
                        self.tpv[self.ply][next] = self.tpv[self.ply + 1][next];
                        next += 1;
                    }
                    self.tpv_len[self.ply] = self.tpv_len[self.ply + 1];

                    if alpha >= beta {
                        return beta;  // fail high, opponent will not choose the branch led to this move
                    }
                }
            }
        }

        alpha  // fail low, we won't choose the branch led to this move
    }

    pub fn post(&self, score: i16) {
        if self.post {
            if self.evm {
                print!("depth {} / {} / ms {} / nodes {} / pv:", self.td, score, self.ts.elapsed().as_millis(), self.nodes);
            } else {
                print!("depth {} / {} / ms {} / nodes {} / pv:", self.td, &format_eval(score), self.ts.elapsed().as_millis(), self.nodes);
            }
            for (_, mov) in self.tpv[0].iter().enumerate().take(max(self.tpv_len[0], 1)) {
                print!(" {}", transform_move_back(*mov));
            }
            println!();
        }
    }
}

// Before calling this function, search MUST determine if the game already ended!
pub fn eval(board: &Board) -> i16 {
    let mut score = 0;

    // scores on the local boards, separated
    let mut xscores = [0; 9];
    let mut oscores = [0; 9];

    // get local scores
    for i in 0u8..9 {
        let xs = (board.locals[0] & SUB_LOOKUP[i as usize]) >> (i * 9);
        let os = (board.locals[1] & SUB_LOOKUP[i as usize]) >> (i * 9);
        let lbs = xs as usize | (os << 9) as usize;
        xscores[i as usize] = LOCAL_SCORES.0[lbs];
        oscores[i as usize] = LOCAL_SCORES.1[lbs];
    }

    // line scores
    let mut xlines = [0; 8];
    let mut olines = [0; 8];

    // convert local scores to line scores
    for (i, lookup) in WIN_LOOKUP.iter().enumerate() {
        let mut xcnt: i16 = 1;
        let mut ocnt: i16 = 1;
        let mut bits = *lookup;
        while bits != 0 {
            let bit = bits.pop_bit();
            xcnt *= xscores[bit as usize] as i16;
            ocnt *= oscores[bit as usize] as i16;
        }
        xlines[i] = xcnt;
        olines[i] = ocnt;
    }

    xlines.sort();
    xlines.reverse();
    olines.sort();
    olines.reverse();

    // with MAX_LOCAL_SCORE = 20 theorietically (im)possible upperbound is (20^3) * 1.9375, and 15500
    score += xlines[0] + xlines[1] / 2 + xlines[2] / 4 + xlines[3] / 8 + xlines[4] / 16;
    score -= olines[0] + olines[1] / 2 + olines[2] / 4 + olines[3] / 8 + olines[4] / 16;

    // there could be an additional weight if free move, feel lazy to implement

    score
}


#[cfg(test)]
mod tests {
    use crate::board::transform_move;
    use super::*;

    #[test]
    fn eval_common_sense() {
        // don't change this test unless you're absolutely sure you know what you're doing
        let mut board = Board::default();

        board.import_ken("xx1xox1xx-o8-9-o8-9-o8-9-o8-o8 b2");
        assert!(eval(&board) < 0);
        board.import_ken("xx1x1xoxx-o8-9-o8-9-o8-9-o8-o8 a3");
        assert!(eval(&board) < 0);
        board.import_ken("xxox1x1xx-o8-9-o8-9-o8-9-o8-o8 c1");
        assert!(eval(&board) < 0);
        board.import_ken("9-9-9-9-4x4-9-9-9-9 e5");
        assert!(eval(&board) > 0);

        board.import_history("1. e5 d6 2. a9 c7 3. i1 g3 4. a8 a5 5. c5 i5 6. g5 a6 7. a7 a1 8. b1 e1 9. e3 e9 10. d7 b2 11. d4 c3 12. i9 g8 13. c6 i8 14. g4 f6 15. g7 h8 16. e4 f1 17. h1 d1 18. e6");
        let legals = board.generate_legal_moves();
        let eval0 = eval(&board);
        assert!(eval0 > 0);

        board.make_move(transform_move("d9", legals));
        let eval1 = eval(&board);
        assert!(eval0 <= eval1);
        board.undo_move();

        board.make_move(transform_move("f9", legals));
        let eval2 = eval(&board);
        assert!(eval0 <= eval2);
        board.undo_move();

        board.make_move(transform_move("e8", legals));
        let eval3 = eval(&board);
        assert!(eval0 <= eval3);
        board.undo_move();

        board.make_move(transform_move("e7", legals));
        let eval4 = eval(&board);
        assert!(eval0 <= eval4);
        board.undo_move();

        board.make_move(transform_move("d8", legals));
        let eval5 = eval(&board);
        assert!(eval5 <= eval1 && eval5 <= eval2 && eval5 <= eval3 && eval5 <= eval4);
    }

    #[test]
    fn eval_basic() {
        // may depend on how you implement weights
        let mut board = Board::default();

        board.import_ken("x8-9-9-9-9-9-9-9-9 a1");
        let eval10 = eval(&board);
        board.import_ken("xx7-o8-9-9-9-9-9-9-9 a1");
        assert!(eval(&board) > eval10);

        board.import_ken("o8-9-9-9-9-9-9-9-9 a1");
        let eval11 = eval(&board);
        board.import_ken("oo7-x8-9-9-9-9-9-9-9 a1");
        let eval12 = eval(&board);
        assert!(eval12 < eval11);

        board.import_ken("oo1oo4-9-9-9-9-9-9-9-9");
        let eval13 = eval(&board);
        assert!(eval13 > eval12);  // not a mistake 
        board.import_ken("oo7-9-9-9-9-9-9-9-9");
        let eval14 = eval(&board);
        assert!(eval13 > eval14);  // not a mistake 
    }

    #[test]
    fn eval_limits() {
        // if this test fails, it's bad for the search function
        let mut board = Board::default();

        board.import_ken("xx1xxx1xx-1xxx1xxx1-xx1x1x1xx-1xxx1xxx1-xx1x1x1xx-1xxx1xxx1-xx1x1x1xx-1xxx1xxx1-xx1xxx1xx -");
        let ev1 = eval(&board);
        assert!(ev1 > 0);
        assert!(ev1 < LARGE);

        board.import_ken("xx1xxx1xx-1xxxxxxx1-xx1x1x1xx-1xxxxxxx1-xx1x1x1xx-1xxxxxxx1-xx1x1x1xx-1xxxxxxx1-xx1xxx1xx -");
        assert!(eval(&board) > 0);
        assert!(eval(&board) < LARGE);
        assert!(eval(&board) > ev1);

        board.import_ken("xxx-xxx-9-xxx-9-xxx-9-xxx-xxx -");
        let ev1 = eval(&board);
        assert!(ev1 > 0);
        assert!(ev1 < LARGE);

        let mut board = Board::default();
        board.import_ken("oo1o1o1oo-1ooo1ooo1-oo1o1o1oo-1ooo1ooo1-oo1ooo1oo-1ooo1ooo1-oo1o1o1oo-1ooo1ooo1-oo1o1o1oo -");
        assert!(eval(&board) < 0);
        assert!(eval(&board) > -LARGE);
    }
}