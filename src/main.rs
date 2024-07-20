use frame::field::Field;

mod frame;

fn main() {
    let mut fd = Field::default();
    for i in 1..9 {
        println!("depth = {}, legal moves counted = {}", i, fd.perft(i));
    }
}