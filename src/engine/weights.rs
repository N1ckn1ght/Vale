pub struct Weights {
    pub possibility_limit: i32,
    pub local_threat_mult: i32,
    pub local_win_mult: i32,
    pub global_emerging_threat_value: i32,
    pub global_threat_value: i32,
    pub soft_max: i32
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            // local_possibility_add: 1,
                // algorithm to count possibilities:
                // 1. get every line from local board (total: 8)
                // 2. if line contains current mark and is not blocked by other mark, add 1
                // 3. save as max(count, possibility_limit)
            possibility_limit: 4,
            local_threat_mult: 5,
                // will overwrite possibilities, always counts as single threat
            // global_possibility_add: 1,
                // count global possibilities by the same algorithm
                // note: no need to have a won local board; just need a possibility to win one
                // multiply by it; e.g. if 0, then it's useless board after all
            local_win_mult: 9,
                // will override anything on the board, of course
            global_emerging_threat_mult: 2,
                // this is a possibility with a local board that's already won
                // if two boards are won in the row it'd be equal to two boards being won on two different lanes
            global_threat_mult: 2,
                // additional bonus for having a real threat, two boards in the row
            soft_max: 8192,
                // victory bonus
        }
    }
}