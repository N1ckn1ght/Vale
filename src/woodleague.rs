use std::io;


macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn main() {
    let mut board = SmallBoard::default();
    let mut engine = SmallEngine::default();

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
            let mov = SmallBoard::transform_move(opponent_row, opponent_col);
            board.make_move(mov);

            // board.print();
            if board.status < 3 {
                break;
            }
        }

        let (score, mov) = engine.search(&mut board);
        board.make_move(mov);
        let (r, c) = SmallBoard::transform_move_back(mov);
        println!("{} {}", r, c);

        // println!("score: {}", score);
        // board.print();
        if board.status < 3 {
            break;
        }
    }
}


const PLY_LIMIT: usize = 10;
const INF: i16 = 16384;
pub const LARGE: i16 = 8192;
pub const LARGM: i16 = LARGE - 16;

pub struct SmallEngine {
    ply:      usize,                          // current distance to the search root
    tpv:      [[u8; PLY_LIMIT]; PLY_LIMIT],  // triangular table of a principal variation
    tpv_len:  [usize; PLY_LIMIT],            // current length of tpv
}

impl Default for SmallEngine {
    fn default() -> Self {
        Self {
            ply: 0,
            tpv: [[0; PLY_LIMIT]; PLY_LIMIT],
            tpv_len: [0; PLY_LIMIT],
        }
    }
}

impl SmallEngine {
    pub fn search(&mut self, board: &mut SmallBoard) -> (i16, u8) {
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
        let score = self.negamax(board, alpha, beta);
        (score, self.tpv[0][0])
    }

    pub fn negamax(&mut self, board: &mut SmallBoard, mut alpha: i16, beta: i16) -> i16 {
        self.tpv_len[self.ply] = self.ply;

        match board.status {
            3 => {},
            2 => { return eval(board); },
            _ => { return -LARGE + self.ply as i16 }
        }

        let mut legals = board.generate_legal_moves();

        while legals != 0 {
            let bit = legals.pop_bit();

            self.ply += 1;
            board.make_move(bit);

            let score = -self.negamax(board, -beta, -alpha);

            board.undo_move();
            self.ply -= 1;

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

        alpha
    }
}

pub static TRAP: [i16; 9] = [5, 0, 3, 0, 2, 0, 3, 0, 5];

pub fn eval(board: &SmallBoard) -> i16 {
    let mut cnt = 0;
    let mut l0 = board.bbs[board.turn as usize];
    let mut l1 = board.bbs[!board.turn as usize];
    while l0 != 0 {
        cnt += TRAP[l0.pop_bit() as usize];
    }
    while l1 != 0 {
        cnt -= TRAP[l1.pop_bit() as usize];
    }
    cnt
}

pub struct SmallBoard {
    pub bbs: [u16; 2],
    pub status: u8,
    pub turn: bool,
    pub bkps: Vec<u16>
}

impl Default for SmallBoard {
    fn default() -> Self {
        Self {
            bbs: [0; 2],
            status: 3,
            turn: false,
            bkps: Vec::with_capacity(10)
        }
    }
}

impl SmallBoard {
    pub fn generate_legal_moves(&self) -> u16 {
        if self.status < 3 {
            return 0;
        }

        !(self.bbs[0] | self.bbs[1]) & SF
    }

    pub fn make_move(&mut self, mov: u8) {
        let my_turn = self.turn as usize;

        self.bkps.push(self.bbs[my_turn]);
        self.bbs[my_turn].set_bit(mov);

        for mask in WIN_LOOKUP_INDEXED.iter().skip(WIN_LOOKUP_INDICES[mov as usize][0]).take(WIN_LOOKUP_INDICES[mov as usize][1]) {
            if self.bbs[my_turn] & mask == *mask {
                self.status = my_turn as u8;    
                break;
            }
        }

        if self.status > 2 && (self.bbs[0] | self.bbs[1]) == SF {
            self.status = 2;
        }

        self.turn = !self.turn;
    }

    pub fn undo_move(&mut self) {
        self.turn = !self.turn;
        let my_turn = self.turn as usize;

        self.bbs[my_turn] = self.bkps.pop().unwrap();
        self.status = 3;
    }

    pub fn transform_move(row: i32, col: i32) -> u8 {
        (row * 3 + col) as u8
    }

    pub fn transform_move_back(mov: u8) -> (i32, i32) {
        ((mov / 3) as i32, (mov % 3) as i32)
    }

    pub fn print(&self) {
        for i in 0..9 {
            if self.bbs[0].get_bit(i) != 0 {
                print!("x ");
            } else if self.bbs[1].get_bit(i) != 0 {
                print!("o ");
            } else {
                print!(". ");
            }
            if i % 3 == 2 {
                println!();
            }
        }
    }
}


/* Lookups */

pub const DIV_LOOKUP: [u8; 81] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8];
pub const MOD_LOOKUP: [u8; 81] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8];
pub const SUB_LOOKUP: [u128; 9] = [0b000000000000000000000000000000000000000000000000000000000000000000000000111111111, 0b000000000000000000000000000000000000000000000000000000000000000111111111000000000, 0b000000000000000000000000000000000000000000000000000000111111111000000000000000000, 0b000000000000000000000000000000000000000000000111111111000000000000000000000000000, 0b000000000000000000000000000000000000111111111000000000000000000000000000000000000, 0b000000000000000000000000000111111111000000000000000000000000000000000000000000000, 0b000000000000000000111111111000000000000000000000000000000000000000000000000000000, 0b000000000111111111000000000000000000000000000000000000000000000000000000000000000, 0b111111111000000000000000000000000000000000000000000000000000000000000000000000000];
pub const WIN_LOOKUP: [u16; 8] = [0b000000111, 0b000111000, 0b001001001, 0b001010100, 0b010010010, 0b100010001, 0b100100100, 0b111000000];
pub const WIN_LOOKUP_INDEXED: [u16; 24] = [0b000000111, 0b001001001, 0b100010001, 0b000000111, 0b010010010, 0b000000111, 0b001010100, 0b100100100, 0b000111000, 0b001001001, 0b000111000, 0b001010100, 0b010010010, 0b100010001, 0b000111000, 0b100100100, 0b001001001, 0b001010100, 0b111000000, 0b010010010, 0b111000000, 0b100010001, 0b100100100, 0b111000000];
pub const WIN_LOOKUP_INDICES: [[usize; 2]; 9] = [[0, 3], [3, 2], [5, 3], [8, 2], [10, 4], [14, 2], [16, 3], [19, 2], [21, 3]];


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
