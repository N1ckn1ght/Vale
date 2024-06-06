pub struct Field {
    bitboards:  [u64; 2],   // bitboards for X and O respectively
    turn:       bool,       // is current move for O?

    history:    Vec<u16>
}

impl Field {

    pub fn make_null_move(&mut self) {
        self.turn = !self.turn;
    }
}