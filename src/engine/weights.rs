pub struct Weights {
    pub pre_mult_limit: i32,
    pub local_threat_mult: i32,
    pub local_win_mult: i32,
    pub core_board_value: i32,
    pub global_threat_value: i32
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            pre_mult_limit: 4,
            local_threat_mult: 5,
            local_win_mult: 9,
            core_board_value: 9,
            global_threat_value: 364
        }
    }
}