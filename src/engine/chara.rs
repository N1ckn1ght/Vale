use crate::frame::field::Field;

use std::time::Instant;

pub struct Vale {
    field: Field,

    /* Accessible constants */

    /* Search trackers */
    ts:    Instant,          // timer start
    tl:    u128,             // time limit in ms
    abord: bool,             // stop search signal
    nodes: u64,              // nodes searched
    ply:   usize,            // current distance to the search root
    
    tpv:   [[u32; 32]; 32],  // 
}

impl Vale {
    pub fn think(&mut self) {
        
    }

    pub fn search(&mut self) {
        
    }

    pub fn eval(&mut self) {
        
    }
}   