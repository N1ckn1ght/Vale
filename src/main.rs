mod bitboard;
mod lookups;
mod board;
mod engine;
mod interface;

use core::time;
use std::{thread::sleep, time::Instant};
use crate::{board::Board, interface::user_box};


fn main() {
    println!("Hello, world!");
    let _ = &*engine::LEVAL_WEIGHTS;
    let _ = &*engine::LEVAL_XPOS;
    let _ = &*engine::LEVAL_OPOS;
    println!("Force init completed.");

    perft(8);
    // println!("(results are a bit skewed because of user_box right after perft idk)\n");
    // user_box();
}

fn perft(x: u8) {
    let mut board = Board::default();

    let timer = Instant::now();
    let mut mvc: u64 = 0;
    for i in 1..=x {
        let cmvc = board.perft(i);
        println!("depth = {}, legal moves counted = {}", i, cmvc);
        mvc += cmvc;
    }
    let time = timer.elapsed().as_millis() as u64;
    println!("moves = {}, time = {} ms", mvc, time);
    println!("speed = {}.{}mps", mvc / time / 1000, mvc / time % 1000);
}