use crate::frame::field::Field;

use std::time::Instant;

const PLY_LIMIT: usize = 96;   // 81
const INF: i32 = 1048576;

pub struct Vale {
    field:    Field,
    
    /* Accessible constants */
    maps:     [u16; 0b1000000000000000000],  // 0b ooooooooo xxxxxxxxx - this format will yield [ ]

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
        let field = Field::default();
        
        Self {
            field,
            maps: [0; 0b1000000000000000000],
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
        let legals = self.field.generate_legal_moves();
        
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

    pub fn eval(&mut self) {
        
    }
}