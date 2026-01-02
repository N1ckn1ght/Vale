use std::{cmp::{Reverse, min}, io, time::Instant};


macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn main() {
    let mut xs = vec![0u8; 262144];
    let mut os = vec![0u8; 262144];
    gen_local_scores(&mut xs, &mut os);

    let mut board = Board::default();
    let mut engine = Engine::default();

    loop {
        /* input */
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let opponent_row = parse_input!(inputs[0], i32);
        let opponent_col = parse_input!(inputs[1], i32);

        /* useless chunk of code */
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let valid_action_count = parse_input!(input_line, i32);
        for i in 0..valid_action_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let row = parse_input!(inputs[0], i32);
            let col = parse_input!(inputs[1], i32);
        }
        /* end of useless part */

        if opponent_row != -1 {
            let mov = transform_move(opponent_row, opponent_col);
            board.make_move(mov);

            if board.status < 3 {
                break;
            }
        }

        let (mov, _) = engine.search(&mut board, Some(95), None, &xs, &os);
        board.make_move(mov);
        let (r, c) = transform_move_back(mov);
        println!("{} {}", r, c);

        if board.status < 3 {
            break;
        }
    }
}


/* Engine */

// comms
const NODES_BETWEEN_UPDATES: u64 = 2048;

// search aux
const PLY_LIMIT: usize = 81;
const INF: i32 = 1_073_741_824;
pub const LARGE: i32 = 536_870_912;
pub const LARGM: i32 = LARGE - 512;

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
    pub fn search(&mut self, board: &mut Board, time_limit_ms: Option<u128>, depth_limit: Option<usize>, XS: &[u8], OS: &[u8]) -> (u8, i32) {
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
            let temp = self.negamax(board, alpha, beta, self.td, XS, OS);
            if !self.abort {
                score = temp;
                if score > LARGM || score < -LARGM {
                    if !self.mate {
                        self.mate = true;
                    } else {
                        self.abort = true;
                    }
                } else {
                    self.mate = false;
                }
                if board.turn {
                    score = -score;  // maybe should be not there?
                }
            } else {
                break;
            }
            self.post(score);

            self.td += 2;
            if self.td > dl || self.ts.elapsed().as_millis() > self.tl {
                break;
            }
        }

        (self.tpv[0][0], score)
    }

    pub fn negamax(&mut self, board: &mut Board, mut alpha: i32, beta: i32, depth: i8, XS: &[u8], OS: &[u8]) -> i32 {
        if self.nodes & NODES_BETWEEN_UPDATES == 0 {
            self.update();
        }

        self.nodes += 1;
        self.tpv_len[self.ply] = self.ply;

        match board.status {
            3 => {},
            2 => { return 0; },
            _ => { return -LARGE + self.ply as i32 }
        }

        let mut legals = board.generate_legal_moves();

        if depth == 0 || self.ply > PLY_LIMIT {
            if board.turn {
                return -eval(board, XS, OS);
            }
            return eval(board, XS, OS);
        }

        // pre-sort on eval when it makes sense, so if depth > 1
        if depth > 1 {
            let total_moves = legals.count_ones();
            let mut deep_moves = 2;
            let mut zugs = 0;
            let mut presort: Vec<(u8, i32)> = Vec::with_capacity(total_moves as usize);
            while legals != 0 {
                let bit = legals.pop_bit();
                board.make_move(bit);
                let mut bscore = eval(board, XS, OS);
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
                    -self.negamax(board, -beta, -alpha, depth - 2, XS, OS)
                } else {
                    -self.negamax(board, -beta, -alpha, depth - 2 - (depth > 3) as i8 - ((depth > 4) && (i > 18)) as i8, XS, OS)
                };

                if score > alpha {
                    score = -self.negamax(board, -alpha - 1, -alpha, depth - 1, XS, OS);
                    if score > alpha && score < beta {
                        score = -self.negamax(board, -beta, -alpha, depth - 1, XS, OS);
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

                let score = -self.negamax(board, -beta, -alpha, depth - 1, XS, OS);

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

    pub fn post(&self, score: i32) {
        // if self.post {
        //     if self.evm {
        //         print!("depth {} / {} / ms {} / nodes {} / pv:", self.td, score, self.ts.elapsed().as_millis(), self.nodes);
        //     } else {
        //         print!("depth {} / {} / ms {} / nodes {} / pv:", self.td, &format_eval(score), self.ts.elapsed().as_millis(), self.nodes);
        //     }
        //     for (_, mov) in self.tpv[0].iter().enumerate().take(max(self.tpv_len[0], 1)) {
        //         print!(" {}", transform_move_back(*mov));
        //     }
        //     println!();
        // }
    }
}

// Before calling this function, search MUST determine if the game already ended!
pub fn eval(board: &Board, XS: &[u8], OS: &[u8]) -> i32 {
    let mut score = 0;

    // scores on the local boards, separated
    let mut xscores = [0; 9];
    let mut oscores = [0; 9];

    // get local scores
    for i in 0u8..9 {
        let xs = (board.locals[0] & SUB_LOOKUP[i as usize]) >> (i * 9);
        let os = (board.locals[1] & SUB_LOOKUP[i as usize]) >> (i * 9);
        let lbs = xs as usize | (os << 9) as usize;
        xscores[i as usize] = XS[lbs];
        oscores[i as usize] = OS[lbs];
    }

    // line scores
    let mut xlines = [0; 8];
    let mut olines = [0; 8];

    // convert local scores to line scores
    for (i, lookup) in WIN_LOOKUP.iter().enumerate() {
        let mut xcnt: i32 = 1;
        let mut ocnt: i32 = 1;
        let mut bits = *lookup;
        while bits != 0 {
            let bit = bits.pop_bit();
            xcnt *= xscores[bit as usize] as i32;
            ocnt *= oscores[bit as usize] as i32;
        }
        xlines[i] = xcnt;
        olines[i] = ocnt;
    }

    xlines.sort();
    xlines.reverse();
    olines.sort();
    olines.reverse();

    /* Eval function itself */
    score += xlines[0] + xlines[1] / 4 + xlines[2] / 16 + xlines[3] / 64 + xlines[4] / 256 + xlines[5] / 1024;
    score -= olines[0] + olines[1] / 4 + olines[2] / 16 + olines[3] / 64 + olines[4] / 256 + olines[5] / 1024;

    score
}


/* Weights */

static POS_SCORE: [[u8; 9]; 4] = [
    [   0, 128, 128, 128, 128, 128, 128, 128, 128],
    [   0,  32,  64,  64,  64,  64,  64,  64,  64],
    [   0,  16,  20,  24,  32,  32,  32,  32,  32],
    [   0,   1,   4,   6,   8,  10,  12,  16,  16]
];

static SC_LIMITS: [u8; 4] = [128, 64, 32, 16];

pub fn gen_local_scores(xscores: &mut [u8], oscores: &mut [u8]) {
    for permut in 0usize..262144 {
        let xbits = (permut & 0b111111111) as u16;
        let obits = ((permut >> 9) & 0b111111111) as u16;

        // impossible
        if xbits & obits != 0 {
            continue;
        }

        let (xl, ol) = gen_local_map(permut);

        if (xl >> POS_CNT[0]) & POS_MASK != 0 {
            xscores[permut] = POS_SCORE[0][1];
            continue;
        }
        if (ol >> POS_CNT[0]) & POS_MASK != 0 {
            oscores[permut] = POS_SCORE[0][1];
            continue;
        }

        if xscores[permut] == 0 {
            xscores[permut] = POS_SCORE[1][((xl >> POS_CNT[1]) & POS_MASK) as usize] + POS_SCORE[2][((xl >> POS_CNT[2]) & POS_MASK) as usize] + POS_SCORE[3][((xl >> POS_CNT[3]) & POS_MASK) as usize];
            if (xl >> POS_CNT[1]) & POS_MASK != 0 {
                xscores[permut] = min(xscores[permut], SC_LIMITS[1]);
            } else if (xl >> POS_CNT[2]) & POS_MASK != 0 {
                xscores[permut] = min(xscores[permut], SC_LIMITS[2]);
            } else {
                xscores[permut] = min(xscores[permut], SC_LIMITS[3]);
            }
        }
        if oscores[permut] == 0 {
            oscores[permut] = POS_SCORE[1][((ol >> POS_CNT[1]) & POS_MASK) as usize] + POS_SCORE[2][((ol >> POS_CNT[2]) & POS_MASK) as usize] + POS_SCORE[3][((ol >> POS_CNT[3]) & POS_MASK) as usize];
            if (ol >> POS_CNT[1]) & POS_MASK != 0 {
                oscores[permut] = min(oscores[permut], SC_LIMITS[1]);
            } else if (ol >> POS_CNT[2]) & POS_MASK != 0 {
                oscores[permut] = min(oscores[permut], SC_LIMITS[2]);
            } else {
                oscores[permut] = min(oscores[permut], SC_LIMITS[3]);
            }
        }
    }
}


/* Board impl */

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
                if self.global[0] | self.global[1] | self.global[2] == SF {
                    self.status = 2;
                }
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

pub fn transform_move(row: i32, col: i32) -> u8 {
    let block_row = row / 3;
    let block_col = col / 3;
    let block = block_row * 3 + block_col;

    let inner_row = row % 3;
    let inner_col = col % 3;
    let inner = inner_row * 3 + inner_col;

    (block * 9 + inner) as u8
}

pub fn transform_move_back(index: u8) -> (i32, i32) {
    let block = index as i32 / 9;
    let inner = index as i32 % 9;

    let block_row = block / 3;
    let block_col = block % 3;

    let inner_row = inner / 3;
    let inner_col = inner % 3;

    let row = block_row * 3 + inner_row;
    let col = block_col * 3 + inner_col;

    (row, col)
}

#[inline]
pub fn grb(rank: u8, file: u8) -> u8 {
    (rank / 3) * 27 + (rank % 3) * 3 + (file / 3) * 9 + (file % 3)
}


/* Lookups */

pub const DIV_LOOKUP: [u8; 81] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8];
pub const MOD_LOOKUP: [u8; 81] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8];
pub const SUB_LOOKUP: [u128; 9] = [0b000000000000000000000000000000000000000000000000000000000000000000000000111111111, 0b000000000000000000000000000000000000000000000000000000000000000111111111000000000, 0b000000000000000000000000000000000000000000000000000000111111111000000000000000000, 0b000000000000000000000000000000000000000000000111111111000000000000000000000000000, 0b000000000000000000000000000000000000111111111000000000000000000000000000000000000, 0b000000000000000000000000000111111111000000000000000000000000000000000000000000000, 0b000000000000000000111111111000000000000000000000000000000000000000000000000000000, 0b000000000111111111000000000000000000000000000000000000000000000000000000000000000, 0b111111111000000000000000000000000000000000000000000000000000000000000000000000000];
pub const WIN_LOOKUP: [u16; 8] = [0b000000111, 0b000111000, 0b001001001, 0b001010100, 0b010010010, 0b100010001, 0b100100100, 0b111000000];
pub const WIN_LOOKUP_INDEXED: [u16; 24] = [0b000000111, 0b001001001, 0b100010001, 0b000000111, 0b010010010, 0b000000111, 0b001010100, 0b100100100, 0b000111000, 0b001001001, 0b000111000, 0b001010100, 0b010010010, 0b100010001, 0b000111000, 0b100100100, 0b001001001, 0b001010100, 0b111000000, 0b010010010, 0b111000000, 0b100010001, 0b100100100, 0b111000000];
pub const WIN_LOOKUP_INDICES: [[usize; 2]; 9] = [[0, 3], [3, 2], [5, 3], [8, 2], [10, 4], [14, 2], [16, 3], [19, 2], [21, 3]];

pub const POS_CNT: [u8; 4] = [12, 8, 4, 0];
pub const POS_MASK: u16 = 0b1111;

pub fn gen_local_map(permut: usize) -> (u16, u16) {
    let xbits = (permut & 0b111111111) as u16;
    let obits = ((permut >> 9) & 0b111111111) as u16;

    // impossible
    if xbits & obits != 0 {
        return (0, 0);
    }

    let mut x_left = [0; 4];
    let mut o_left = [0; 4];

    for lookup in WIN_LOOKUP {
        let maskx = xbits & lookup;
        let masko = obits & lookup;
        
        if maskx != 0 && masko != 0 {
            continue;
        }
        if maskx == 0 && masko == 0 {
            x_left[3] += 1;
            o_left[3] += 1;
            continue
        }
        if maskx != 0 {
            match maskx.count_ones() {
                1 => { x_left[2] += 1; },
                2 => { x_left[1] += 1; },
                3 => {
                    x_left[0] = 1;
                    break;
                },
                _ => {}
            }
        } else {
            match masko.count_ones() {
                1 => { o_left[2] += 1; },
                2 => { o_left[1] += 1; },
                3 => {
                    o_left[0] = 1;
                    break;
                },
                _ => {}
            }
        }
    }

    if x_left[0] != 0 {
        return (1 << POS_CNT[0], 0);
    }
    if o_left[0] != 0 {
        return (0, 1 << POS_CNT[0]);
    }

    let xpts = (x_left[1] << POS_CNT[1]) | (x_left[2] << POS_CNT[2]) | (x_left[3] << POS_CNT[3]);
    let opts = (o_left[1] << POS_CNT[1]) | (o_left[2] << POS_CNT[2]) | (o_left[3] << POS_CNT[3]);
    (xpts, opts)
}


/* Implements traits for u16 (3x3 boards) and u128 (9x9 board) */

pub const LF: u128 = 0b111111111111111111111111111111111111111111111111111111111111111111111111111111111;
pub const LS: u128 = 0b111111111;
pub const SF: u16  = 0b111111111;

pub trait DelBit<T> {
    fn del_bit(&mut self, bit: u8);
}

pub trait GetBit<T> {
    fn get_bit(&self, bit: u8) -> Self;
}

pub trait SetBit<T> {
    fn set_bit(&mut self, bit: u8);
}

pub trait PopBit<T> {
    fn pop_bit(&mut self) -> u8;
}

pub trait SwapBits<T> {
    fn swap_bits(&mut self, first: u8, second: u8);
}


impl DelBit<&u8> for u16 {
    #[inline]
    fn del_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }
}

impl DelBit<&u8> for u128 {
    #[inline]
    fn del_bit(&mut self, bit: u8) {
        *self &= !(1 << bit);
    }
}

impl GetBit<&u8> for u16 {
    #[inline]
    fn get_bit(&self, bit: u8) -> Self {
        *self & (1 << bit)
    }
}

impl GetBit<&u8> for u128 {
    #[inline]
    fn get_bit(&self, bit: u8) -> Self {
        *self & (1 << bit)
    }
}

impl SetBit<&u8> for u16 {
    #[inline]
    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }
}

impl SetBit<&u8> for u128 {
    #[inline]
    fn set_bit(&mut self, bit: u8) {
        *self |= 1 << bit;
    }
}

impl PopBit<&u8> for u16 {
    #[inline]
    fn pop_bit(&mut self) -> u8 {
        let tz = self.trailing_zeros() as u8;
        *self &= *self - 1;
        tz
    }
}

impl PopBit<&u8> for u128 {
    #[inline]
    fn pop_bit(&mut self) -> u8 {
        let tz = self.trailing_zeros() as u8;
        *self &= *self - 1;
        tz
    }
}

impl SwapBits<&u8> for u16 {
    fn swap_bits(&mut self, first: u8, second: u8) {
        let fb = self.get_bit(first);
        let sb = self.get_bit(second);
        *self &= !(fb | sb);
        if fb != 0 {
            self.set_bit(second);
        }
        if sb != 0 {
            self.set_bit(first);
        }
    }
}

impl SwapBits<&u8> for u128 {
    fn swap_bits(&mut self, first: u8, second: u8) {
        let fb = self.get_bit(first);
        let sb = self.get_bit(second);
        *self &= !(fb | sb);
        if fb != 0 {
            self.set_bit(second);
        }
        if sb != 0 {
            self.set_bit(first);
        }
    }
}
