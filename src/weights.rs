use std::cmp::min;
use crate::lookups::{POS_CNT, POS_MASK, gen_local_map};


/* WEIGHTS PER WIN / THREAT / ATTACK / POSSIBILITY */

pub const MAX_LOCAL_SCORE: i8 = 20;

static POS_SCORE: [[i8; 9]; 4] = [
    [  0, 20, 20, 20, 20, 20, 20, 20, 20],
    [  0,  8,  9, 10, 10, 10, 10, 10, 10],
    [  0,  4,  6,  8,  9,  9,  9,  9,  9],
    [  0,  1,  2,  3,  4,  5,  6,  8,  8]
];


/* WEIGHTS TO SCORE TRANSFORM */

pub fn gen_local_scores(xscores: &mut [i8], oscores: &mut [i8]) {
    for permut in 0usize..262144 {
        let xbits = (permut & 0b111111111) as u16;
        let obits = ((permut >> 9) & 0b111111111) as u16;

        // impossible
        if xbits & obits != 0 {
            continue;
        }

        let (xl, ol) = gen_local_map(permut);

        if (xl >> POS_CNT[0]) & POS_MASK != 0 {
            xscores[permut] = MAX_LOCAL_SCORE;
            continue;
        }
        if (ol >> POS_CNT[0]) & POS_MASK != 0 {
            oscores[permut] = MAX_LOCAL_SCORE;
            continue;
        }

        xscores[permut] = min(
            POS_SCORE[1][((xl >> POS_CNT[1]) & POS_MASK) as usize] +
            POS_SCORE[2][((xl >> POS_CNT[2]) & POS_MASK) as usize] +
            POS_SCORE[3][((xl >> POS_CNT[3]) & POS_MASK) as usize],
            MAX_LOCAL_SCORE - 1
        );
        oscores[permut] = min(
            POS_SCORE[1][((ol >> POS_CNT[1]) & POS_MASK) as usize] +
            POS_SCORE[2][((ol >> POS_CNT[2]) & POS_MASK) as usize] +
            POS_SCORE[3][((ol >> POS_CNT[3]) & POS_MASK) as usize],
            MAX_LOCAL_SCORE - 1
        );
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_local_scores_test() {
        let mut xs = [0; 262144];
        let mut os = [0; 262144];
        gen_local_scores(&mut xs, &mut os);

        let t1 = 0b_000000000_000000000;
        assert_eq!(xs[t1], 8);  // xl[t1], 8 << POS_CNT[3]
        assert_eq!(os[t1], 8);  // ol[t1], 8 << POS_CNT[3]

        let t2 = 0b_000000101_000010000;
        assert_eq!(xs[t2], 1 + 6);  // xl[t2], 1 << POS_CNT[3] | 2 << POS_CNT[2]
        assert_eq!(os[t2], 1 + 6 + 8);  // ol[t2], 1 << POS_CNT[3] | 2 << POS_CNT[2] | 1 << POS_CNT[1]

        let t3 = 0b_011100101_100011010;
        assert_eq!(xs[t3], 0);  // xl[t3], 0
        assert_eq!(os[t3], 0);  // ol[t3], 0

        let t4 = 0b_111000000_000010100;
        assert_eq!(xs[t4], 0);
        assert_eq!(os[t4], MAX_LOCAL_SCORE);  // ol[t4].get_bit(POS_CNT[0]) != 0

        let t5 = 0b_000000000_110101011;
        assert_eq!(xs[t5], 11);
        assert_eq!(os[t5], 1);

        let t6 = 0b_000000000_101000100;
        assert_eq!(xs[t6], MAX_LOCAL_SCORE - 1);
        assert_eq!(os[t6], 2);
    }
}