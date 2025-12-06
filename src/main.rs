mod bitboard;
mod lookups;
mod board;
mod engine;
mod interface;

use std::time::Instant;
use crate::{board::Board, interface::{user_input_move, print_board}};


fn main() {
    inter();
    // perft(8);
}

fn inter() {
    let mut board = Board::default();
    while board.status > 2 {
        print_board(&board);
        let legals = board.generate_legal_moves();
        println!("{}", format!("{:0128b}", legals));
        let mov = user_input_move(legals);
        board.make_move(mov);
    }
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