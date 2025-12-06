use std::{cmp::{max, min}, time::Instant};
use once_cell::sync::Lazy;
use crate::{bitboard::{GetBit, PopBit, SetBit}, board::Board, lookups::{SUB_LOOKUP, WIN_LOOKUP}};

// search aux
const PLY_LIMIT: usize = 96;   // 81
const INF: i16 = 16384;

// eval weights
static LEVAL_WEIGHTS: Lazy<[i8; 262144]> = Lazy::new(gen_leval_weights); 
static ANCHOR_WEIGHTS: [i16; 9] = [3, 2, 3, 2, 4, 2, 3, 2, 3];
const FREE_MOVE_FACT: i16 = 9;

// eval aux
static LEVAL_XPOS: Lazy<[bool; 262144]> = Lazy::new(gen_leval_xpos);
static LEVAL_OPOS: Lazy<[bool; 262144]> = Lazy::new(gen_leval_opos);


pub struct Vale {
    board:    Board,

    /* Search trackers */
    ts:       Instant,                       // timer start
    tl:       u128,                          // time limit in ms
    abort:    bool,                          // stop search signal
    nodes:    u64,                           // nodes searched
    ply:      usize,                         // current distance to the search root
    
    tpv:      [[i8; PLY_LIMIT]; PLY_LIMIT],  // triangular table of a principal variation
    tpv_len:  [usize; PLY_LIMIT],            // current length of tpv
    tpv_flag: bool,                          // is this variation the principle one
    cur_ply:  i8,                            // current depth

}

impl Vale {
    pub fn init() -> Self {
        let board = Board::default();
        
        Self {
            board,
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

    pub fn think(&mut self, aspiration_window: i32, time_limit_ms: u128, depth_limit: i8) {
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
        let legals = self.board.generate_legal_moves();
        
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

    pub fn search(&mut self) {

    }

    // Before calling this function, search MUST determine if the game already ended!
    // legals - legal moves, eval takes in account (heuristically) number of moves available, and returns better score in case it's more than threshold
    pub fn eval(&mut self, legals: u128) -> i16 {
        let mut score = 0;

        // getting LOCAL_EVALS related + other stuff that needs loop over locals
        let mut xpos: u16 = 0;
        let mut opos: u16 = 0;
        let mut scores = [0; 9];
        for i in 0u8..9 {
            let xs = (self.board.locals[0] & SUB_LOOKUP[i as usize]) >> i * 9;
            let os = (self.board.locals[1] & SUB_LOOKUP[i as usize]) >> i * 9;
            let lbs = xs as usize | (os << 9) as usize;
            if LEVAL_XPOS[lbs] {
                xpos.set_bit(i);
            }
            if LEVAL_OPOS[lbs] {
                opos.set_bit(i);
            }
            scores[i as usize] = LEVAL_WEIGHTS[lbs];
            // applying KEY/ANCHOR CELLS (pre-sort optimization mostly) scores
            if self.board.global[i as usize] > 1 {
                if xs.get_bit(i) != 0 {
                    score += ANCHOR_WEIGHTS[i as usize];
                } else if os.get_bit(i) != 0 {
                    score -= ANCHOR_WEIGHTS[i as usize];
                }
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
        score += x1 * 10 + x2 - o1 * 10 - o2;

        // applying move count heuristic
        if legals.count_ones() > 9 {
            if self.board.turn {
                score -= FREE_MOVE_FACT;
            } else{
                score += FREE_MOVE_FACT;
            }
        }

        score
    }
}


// the point of this function is to get and save relative score (+ for X, - for O) on a local board for further calculations
// it does not account if it's possible to improve (from 0) score at all, so we have to use the following functions as well
// note: the initial idea was to have x_score and o_score split, so this is an experiment
fn gen_leval_weights() -> [i8; 262144] {
    const ERR: i8 = 0;
    const MX: i8 = 30;
    let mut results: [i8; 262144] = [0; 262144];
    for index in 0..262144 {
        let bbx = (index & 0b111111111) as u16;
        let bbo = ((index >> 9) & 0b111111111) as u16;
        // if permutation is not possible
        if bbx & bbo != 0 {
            results[index] = ERR;
            continue;
        }
        // if it's local win or not possible (aka double win)
        let (mut xw, mut ow) = (false, false);
        for lookup in WIN_LOOKUP.iter() {
            if lookup & bbx == *lookup {
                xw = true;
                continue;
            }
            if lookup & bbo == *lookup {
                ow = true;
            }
        }
        if xw {
            if ow {
                results[index] = ERR;
            } else {
                results[index] = MX;
            }
            continue;
        } else if ow {
            results[index] = -MX;
            continue;
        }
        // if permutation is possible and it's noone's win
        // (guaranteed local draw will return 0 as well)
        let (mut x1, mut x2, mut o1, mut o2) = (0, 0, 0, 0);
        for lookup in WIN_LOOKUP.iter() {
            let maskx = bbx & lookup;
            let masko = bbo & lookup;
            if maskx != 0 {
                if masko != 0 {
                    continue;
                }
                let cnt = maskx.count_ones();
                if cnt > x1 {
                    x2 = x1;
                    x1 = cnt;
                } else if cnt > x2 {
                    x2 = cnt;
                }
            } else if masko != 0 {
                let cnt = masko.count_ones();
                if cnt > o1 {
                    o2 = o1;
                    o1 = cnt;
                } else if cnt > o2 {
                    o2 = cnt;
                }
            }
        }
        results[index] = x1 as i8 * 10 + x2 as i8 - o1 as i8 * 10 - o2 as i8;
    }
    results
}

// this returns true for cases when we can improve our position on a local board for X
// if it's false, then X cannot ever win this local board
fn gen_leval_xpos() -> [bool; 262144] {
    let mut results: [bool; 262144] = [false; 262144];
    for index in 0..262144 {
        let bbx = (index & 0b111111111) as u16;
        let bbo = ((index >> 9) & 0b111111111) as u16;
        // if permutation is not possible
        if bbx & bbo != 0 {
            continue;
        }
        // just check if any possible to win, nothing more
        for lookup in WIN_LOOKUP.iter() {
            let masko = bbo & lookup;
            if masko == 0 {
                results[index] = true;
                break;
            }
        }
    }
    results
}

// this returns true for cases when we can improve our position on a local board for O
// if it's false, then O cannot ever win this local board
fn gen_leval_opos() -> [bool; 262144] {
    let mut results: [bool; 262144] = [false; 262144];
    for index in 0..262144 {
        let bbx = (index & 0b111111111) as u16;
        let bbo = ((index >> 9) & 0b111111111) as u16;
        // if permutation is not possible
        if bbx & bbo != 0 {
            continue;
        }
        // just check if any possible to win, nothing more
        for lookup in WIN_LOOKUP.iter() {
            let maskx = bbx & lookup;
            if maskx == 0 {
                results[index] = true;
                break;
            }
        }
    }
    results
}