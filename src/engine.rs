use std::{cmp::{Reverse, max, min}, time::Instant};
use once_cell::sync::Lazy;
use crate::{bitboard::{GetBit, PopBit, SetBit}, board::Board, lookups::{SUB_LOOKUP, WIN_LOOKUP, gen_local_maps}};

// search aux
const PLY_LIMIT: usize = 81;
const INF: i16 = 16384;
const LARGE: i16 = 8192;  // careful, it's used as |= MASK in search() for tpv

// pub static LOCAL_MAPS: Lazy<(Box<[u16]>, Box<[u16]>)> = Lazy::new(|| {
//     let mut x = vec![0u16; 262144];
//     let mut o = vec![0u16; 262144];
//     gen_local_maps(&mut x, &mut o);
//     (x.into_boxed_slice(), o.into_boxed_slice())
// });
// eval weights
const FREE_MOVE_FACT: i16 = 9;
static ANCHOR_WEIGHTS: [i16; 9] = [3, 2, 3, 2, 4, 2, 3, 2, 3];


pub struct Engine {
    /* Search trackers */
    ts:       Instant,                       // timer start
    tl:       u128,                          // time limit in ms
    abort:    bool,                          // stop search signal
    nodes:    u64,                           // nodes searched
    ply:      usize,                         // current distance to the search root
    
    tpv:      [[u8; PLY_LIMIT]; PLY_LIMIT],  // triangular table of a principal variation
    tpv_len:  [usize; PLY_LIMIT],            // current length of tpv
    tpv_flag: bool,                          // is this variation the principle one
    cur_ply:  i8,                            // current depth
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
            tpv_flag: false,
            cur_ply: 0
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
    pub fn think(
        &mut self,
        board: &mut Board,
        aspiration_window: i32,
        time_limit_ms: u128,
        depth_limit: i8
    ) {
        self.ts = Instant::now();
        self.tl = time_limit_ms;
        self.abort = false;
        
        for line in self.tpv.iter_mut() {
            for node in line.iter_mut() {
                *node = 0;
            }
        }
        for len in self.tpv_len.iter_mut() {
            *len = 0;
        }
        
        let mut alpha = -INF;
        let mut beta  =  INF;
        let mut score =  0;
        let mut delta =  1;
        self.cur_ply = 1;
        let legals = board.generate_legal_moves();
        
        loop {
            self.tpv_flag = true;
            // let temp = self.search(alpha, beta, self.cur_depth);
            if !self.abort {
                // score = temp;
            } else {
                println!()
            }
            

            break;
        }
    }

    pub fn search(
        &mut self,
        board: &mut Board,
        mut alpha: i16,
        beta: i16,
        mut depth: i16
    ) -> i16 {
        self.nodes += 1;
        self.tpv_len[self.ply] = self.ply;

        match board.status {
            3 => {},
            2 => { return 0; },
            _ => { return -LARGE + self.ply as i16 }
        }

        let mut legals = board.generate_legal_moves();

        if depth == 0 || self.ply > PLY_LIMIT {
            return eval(&board, &legals);
        }

        if depth > 1 {
            let mut presort: Vec<(u8, i16)> = Vec::with_capacity(legals.count_ones() as usize);
            let mut lcopy = legals;
            while legals != 0 {
                let bit = legals.pop_bit();
                board.make_move(bit);
                let mut score = eval(board, &board.generate_legal_moves());
                board.undo_move();
                if board.turn {
                    score = -score;
                }
                // principal variation goes first
                if bit == self.tpv[0][self.ply] {
                    score |= LARGE;
                }
                presort.push((bit, score));
            }
            presort.sort_by_key(|&(_, score)| Reverse(score));

            for (i, (mov, _)) in presort.iter().enumerate() {
                
            }
        } else {
            while legals != 0 {
                let bit = legals.pop_bit();
                board.make_move(bit);
                
                board.undo_move();
            }
        }
        // pre-sort?
        

        0
    }
}

/* Before calling this function, search MUST determine if the game already ended!
   legals - legal moves, eval takes in account (heuristically) number of moves available, and returns better score in case it's more than threshold
   it's made as a passable argument to avoid duplicate calculations */
/* This eval is temporary.
   I think I should apply weights depending on the possibilities of win (e.g. 1 open lane vs 3/4) */
pub fn eval(board: &Board, legals: &u128) -> i16 {
    return 0;
    
    let mut score = 0;

    // getting LOCAL_EVALS related + other stuff that needs loop over locals
    let mut xpos: u16 = 0;
    let mut opos: u16 = 0;
    let mut scores = [0; 9];
    for i in 0u8..9 {
        // it duplicates some lazy operations but it actually helps the bottleneck
        if board.global[0].get_bit(i) != 0 {
            xpos.set_bit(i);
            scores[i as usize] = 30;  // LEVAL_WEIGHTS[lbs];
            continue;
        }
        if board.global[1].get_bit(i) != 0 {
            opos.set_bit(i);
            scores[i as usize] = -30;  // LEVAL_WEIGHTS[lbs];
            continue;
        }
        // don't check for global[2] (draw), because the real draw happens when there's no xpos and no opos
        let xs = (board.locals[0] & SUB_LOOKUP[i as usize]) >> (i * 9);
        let os = (board.locals[1] & SUB_LOOKUP[i as usize]) >> (i * 9);
        let lbs = xs as usize | (os << 9) as usize;
        let mut draw_flg = true;
        // if LEVAL_XPOS[lbs] {
        //     xpos.set_bit(i);
        //     draw_flg = false;
        // }
        // if LEVAL_OPOS[lbs] {
        //     opos.set_bit(i);
        //     draw_flg = false;
        // }
        // real draw check (may be avoided because LEVAL_WEIGHTS[lbs] returns 0, BUT needed if there are also other checks later)
        if draw_flg {
            continue;
        }
        // scores[i as usize] = LEVAL_WEIGHTS[lbs];
        // applying KEY/ANCHOR CELLS (pre-sort optimization mostly) scores
        // note that local board is overridden if it's won
        if xs.get_bit(i) != 0 {  // && board.global[0].get_bit(i) == 0 {
            score += ANCHOR_WEIGHTS[i as usize];
        } else if os.get_bit(i) != 0 {  // && board.global[1].get_bit(i) == 0 {
            score -= ANCHOR_WEIGHTS[i as usize];
        }
    }

    // applying LOCAL_EVALS scores
    let (mut x1, mut x2, mut o1, mut o2) = (0, 0, 0, 0);
    for lookup in WIN_LOOKUP.iter() {
        if xpos & lookup == *lookup {
            let mut cnt: i16 = 0;
            let mut bits = *lookup;
            while bits != 0 {
                let bit = bits.pop_bit();
                cnt += max(0, scores[bit as usize] as i16);
            }
            if cnt > x1 {
                x2 = x1;
                x1 = cnt;
            } else if cnt > x2 {
                x2 = cnt;
            }
        }
        if opos & lookup == *lookup {
            let mut cnt: i16 = 0;
            let mut bits = *lookup;
            while bits != 0 {
                let bit = bits.pop_bit();
                cnt += min(0, scores[bit as usize] as i16);
            }
            if cnt < o1 {
                o2 = o1;
                o1 = cnt;
            } else if cnt < o2 {
                o2 = cnt;
            }
        }
    }
    score += x1 * 10 + x2 + o1 * 10 + o2;

    // applying move count heuristic
    if legals.count_ones() > 9 {
        if board.turn {
            score -= FREE_MOVE_FACT;
        } else{
            score += FREE_MOVE_FACT;
        }
    }

    score
}

fn gen_local_chances(xchc: &mut [u16], ochc: &mut[u16]) {
    let mut xlocal = [0; 262144];
    let mut olocal = [0; 262144];
    gen_local_maps(&mut xlocal, &mut olocal);

    for permut in 0usize..262144 {
        let xbits = (permut & 0b111111111) as u16;
        let obits = ((permut >> 9) & 0b111111111) as u16;

        // impossible
        if xbits & obits != 0 {
            continue;
        }

        // check for finished boards
        let mut finish_flg = permut.count_ones() == 9;
        for lookup in WIN_LOOKUP {
            let maskx = xbits & lookup;
            let masko = obits & lookup;

            if maskx == lookup {
                xchc[permut] = 1000;
                finish_flg = true;
                break;
            }
            if masko == lookup {
                ochc[permut] = 1000;
                finish_flg = true;
                break;
            }
        }
        if finish_flg {
            continue;
        }

        // count probabilities
        let mut x_win_cnt = 0;
        let mut o_win_cnt = 0;
        let mut draw_cnt = 0;

        // do dfs? start it earlier?
    }
}


#[cfg(test)]
mod tests {
    use crate::board::transform_move;
    use super::*;

    #[test]
    fn eval_common_sense() {
        // don't change this test unless you're 100% know what you're doing
        let mut board = Board::default();
        board.import_ken("xx1xox1xx-o8-9-o8-9-o8-9-o8-o8 b2");
        assert!(eval(&board, &board.generate_legal_moves()) < 0);
        board.import_ken("xx1x1xoxx-o8-9-o8-9-o8-9-o8-o8 a3");
        assert!(eval(&board, &board.generate_legal_moves()) < 0);
        board.import_ken("xxox1x1xx-o8-9-o8-9-o8-9-o8-o8 c1");
        assert!(eval(&board, &board.generate_legal_moves()) < 0);
        board.import_ken("9-9-9-9-4x4-9-9-9-9 e5");
        assert!(eval(&board, &board.generate_legal_moves()) > 0);

        board.import_history("1. e5 d6 2. a9 c7 3. i1 g3 4. a8 a5 5. c5 i5 6. g5 a6 7. a7 a1 8. b1 e1 9. e3 e9 10. d7 b2 11. d4 c3 12. i9 g8 13. c6 i8 14. g4 f6 15. g7 h8 16. e4 f1 17. h1 d1 18. e6");
        let legals = board.generate_legal_moves();
        let eval0 = eval(&board, &legals);
        assert!(eval0 > 0);

        board.make_move(transform_move("d9", legals));
        let eval1 = eval(&board, &legals);
        assert!(eval0 <= eval1);
        board.undo_move();

        board.make_move(transform_move("f9", legals));
        let eval2 = eval(&board, &legals);
        assert!(eval0 <= eval2);
        board.undo_move();

        board.make_move(transform_move("e8", legals));
        let eval3 = eval(&board, &legals);
        assert!(eval0 <= eval3);
        board.undo_move();

        board.make_move(transform_move("e7", legals));
        let eval4 = eval(&board, &legals);
        assert!(eval0 <= eval4);
        board.undo_move();

        board.make_move(transform_move("d8", legals));
        let eval5 = eval(&board, &legals);
        assert!(eval5 <= eval1 && eval5 <= eval2 && eval5 <= eval3 && eval5 <= eval4);
    }

    #[test]
    fn eval_limits() {
        let mut board = Board::default();
        board.import_ken("xx1xxx1xx-1xxx1xxx1-xx1x1x1xx-1xxx1xxx1-xx1x1x1xx-1xxx1xxx1-xx1x1x1xx-1xxx1xxx1-xx1xxx1xx -");
        let ev1 = eval(&board, &board.generate_legal_moves());
        assert!(ev1 > 0);
        assert!(ev1 < LARGE);
        board.import_ken("xx1xxx1xx-1xxxxxxx1-xx1x1x1xx-1xxxxxxx1-xx1x1x1xx-1xxxxxxx1-xx1x1x1xx-1xxxxxxx1-xx1xxx1xx -");
        assert!(eval(&board, &board.generate_legal_moves()) > 0);
        assert!(eval(&board, &board.generate_legal_moves()) < LARGE);
        assert!(eval(&board, &board.generate_legal_moves()) > ev1);

        let mut board = Board::default();
        board.import_ken("oo1o1o1oo-1ooo1ooo1-oo1o1o1oo-1ooo1ooo1-oo1ooo1oo-1ooo1ooo1-oo1o1o1oo-1ooo1ooo1-oo1o1o1oo -");
        assert!(eval(&board, &board.generate_legal_moves()) < 0);
        assert!(eval(&board, &board.generate_legal_moves()) > -LARGE);
    }

    #[test]
    fn eval_specific() {
        // change this test with every change of eval()

    }
}