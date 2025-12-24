use std::cmp::min;
use crate::lookups::{POS_CNT, POS_MASK, gen_local_maps};


/* WEIGHTS PER WIN / THREAT / ATTACK / POSSIBILITY */

static POS_SCORE: [[i8; 9]; 4] = [
    [  0, 24, 24, 24, 24, 24, 24, 24, 24],
    [  0,  8,  9, 10, 10, 10, 10, 10, 10],
    [  0,  2,  6,  8,  9,  9,  9,  9,  9],
    [  0,  1,  2,  3,  4,  5,  6,  8,  8]
];


/* WEIGHTS TO SCORE TRANSFORM */

pub fn gen_local_scores(xscores: &mut [i16], oscores: &mut [i16]) {
    let mut xlocal = [0; 262144];
    let mut olocal = [0; 262144];
    gen_local_maps(&mut xlocal, &mut olocal);

    for permut in 0usize..262144 {
        let xbits = (permut & 0b111111111) as u16;
        let obits = ((permut >> 9) & 0b111111111) as u16;

        // impossible
        if xbits & obits != 0 {
            continue;
        }

        if (xlocal[permut] >> POS_CNT[0]) & POS_MASK != 0 {
            xscores[permut] = POS_SCORE[0][1] as i16;
            continue;
        }
        if (olocal[permut] >> POS_CNT[0]) & POS_MASK != 0 {
            oscores[permut] = POS_SCORE[0][1] as i16;
            continue;
        }

        xscores[permut] = min(
            POS_SCORE[1][((xlocal[permut] >> POS_CNT[1]) & POS_MASK) as usize] +
            POS_SCORE[2][((xlocal[permut] >> POS_CNT[2]) & POS_MASK) as usize] +
            POS_SCORE[3][((xlocal[permut] >> POS_CNT[3]) & POS_MASK) as usize],
            POS_SCORE[0][1] - 1
        ) as i16;
        oscores[permut] = min(
            POS_SCORE[1][((olocal[permut] >> POS_CNT[1]) & POS_MASK) as usize] +
            POS_SCORE[2][((olocal[permut] >> POS_CNT[2]) & POS_MASK) as usize] +
            POS_SCORE[3][((olocal[permut] >> POS_CNT[3]) & POS_MASK) as usize],
            POS_SCORE[0][1] - 1
        ) as i16;
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
        assert_eq!(os[t4], 24);  // ol[t4].get_bit(POS_CNT[0]) != 0

        let t5 = 0b_000000000_110101011;
        assert_eq!(xs[t5], 23);
        assert_eq!(os[t5], 0);
    }
}