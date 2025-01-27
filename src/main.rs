mod frame;
mod engine;

use std::time::Instant;
use frame::field::Field;

fn main() {
    perft(8);
}

fn perft(x: u8) {
    let mut fd = Field::default();

    let timer = Instant::now();
    let mut mvc: u64 = 0;
    for i in 1..=x {
        let cmvc = fd.perft(i);
        println!("depth = {}, legal moves counted = {}", i, cmvc);
        mvc += cmvc;
    }
    let time = timer.elapsed().as_millis() as u64;
    println!("moves = {}, time = {} ms", mvc, time);
    println!("speed = {}.{}mps", mvc / time / 1000, mvc / time % 1000);
}