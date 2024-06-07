mod frame;

use frame::field::Field;

fn main() {
    let mut field = Field::default();

    field.null_move();
}