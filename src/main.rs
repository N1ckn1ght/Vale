use std::time::Instant;

use frame::field::Field;

mod frame;

fn main() {
    let mut fd = Field::default();

    let timer = Instant::now();
    let mut mvc: u64 = 0;
    for i in 1..=9 {
        let cmvc = fd.perft(i);
        println!("depth = {}, legal moves counted = {}", i, cmvc);
        mvc += cmvc;
    }
    let time = timer.elapsed().as_millis() as u64;
    println!("moves = {}, time = {} ms", mvc, time);
    println!("speed = {}.{}kps", mvc / time / 1000, mvc / time % 10);
    // 214.4 kps, FAIL
    // println!("depth = {}, legal moves counted = {}", 100, fd.perft(100));
}