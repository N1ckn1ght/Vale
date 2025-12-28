use std::cmp::min;
use crate::lookups::{POS_CNT, POS_MASK, gen_local_map};


/* WEIGHTS PER WIN / THREAT / ATTACK / POSSIBILITY */

static POS_SCORE: [[u8; 9]; 4] = [
    [   0, 128, 128, 128, 128, 128, 128, 128, 128],
    [   0,  32,  64,  64,  64,  64,  64,  64,  64],
    [   0,  16,  20,  24,  32,  32,  32,  32,  32],
    [   0,   1,   4,   6,   8,  10,  12,  16,  16]
];

static SC_LIMITS: [u8; 4] = [128, 64, 32, 16];


/* WEIGHTS TO SCORE TRANSFORM */

pub fn gen_local_scores(xscores: &mut [u8], oscores: &mut [u8]) {
    for permut in 0usize..262144 {
        let xbits = (permut & 0b111111111) as u16;
        let obits = ((permut >> 9) & 0b111111111) as u16;

        // impossible
        if xbits & obits != 0 {
            continue;
        }

        let (xl, ol) = gen_local_map(permut);

        if (xl >> POS_CNT[0]) & POS_MASK != 0 {
            xscores[permut] = POS_SCORE[0][1];
            continue;
        }
        if (ol >> POS_CNT[0]) & POS_MASK != 0 {
            oscores[permut] = POS_SCORE[0][1];
            continue;
        }

        if xscores[permut] == 0 {
            xscores[permut] = POS_SCORE[1][((xl >> POS_CNT[1]) & POS_MASK) as usize] + POS_SCORE[2][((xl >> POS_CNT[2]) & POS_MASK) as usize] + POS_SCORE[3][((xl >> POS_CNT[3]) & POS_MASK) as usize];
            if (xl >> POS_CNT[1]) & POS_MASK != 0 {
                xscores[permut] = min(xscores[permut], SC_LIMITS[1]);
            } else if (xl >> POS_CNT[2]) & POS_MASK != 0 {
                xscores[permut] = min(xscores[permut], SC_LIMITS[2]);
            } else {
                xscores[permut] = min(xscores[permut], SC_LIMITS[3]);
            }
        }
        if oscores[permut] == 0 {
            oscores[permut] = POS_SCORE[1][((ol >> POS_CNT[1]) & POS_MASK) as usize] + POS_SCORE[2][((ol >> POS_CNT[2]) & POS_MASK) as usize] + POS_SCORE[3][((ol >> POS_CNT[3]) & POS_MASK) as usize];
            if (ol >> POS_CNT[1]) & POS_MASK != 0 {
                oscores[permut] = min(oscores[permut], SC_LIMITS[1]);
            } else if (ol >> POS_CNT[2]) & POS_MASK != 0 {
                oscores[permut] = min(oscores[permut], SC_LIMITS[2]);
            } else {
                oscores[permut] = min(oscores[permut], SC_LIMITS[3]);
            }
        }
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
        assert_eq!(xs[t1], POS_SCORE[3][8]);  // xl[t1], 8 << POS_CNT[3]
        assert_eq!(os[t1], POS_SCORE[3][8]);  // ol[t1], 8 << POS_CNT[3]

        let t2 = 0b_000000101_000010000;
        assert_eq!(xs[t2], POS_SCORE[3][1] + POS_SCORE[2][2]);  // xl[t2], 1 << POS_CNT[3] | 2 << POS_CNT[2]
        assert_eq!(os[t2], POS_SCORE[3][1] + POS_SCORE[2][2] + POS_SCORE[1][1]);  // ol[t2], 1 << POS_CNT[3] | 2 << POS_CNT[2] | 1 << POS_CNT[1].

        let t3 = 0b_011100101_100011010;
        assert_eq!(xs[t3], 0);  // xl[t3], 0
        assert_eq!(os[t3], 0);  // ol[t3], 0

        let t4 = 0b_111000000_000010100;
        assert_eq!(xs[t4], 0);
        assert_eq!(os[t4], SC_LIMITS[0]);  // ol[t4].get_bit(POS_CNT[0]) != 0

        let t5 = 0b_000000000_110101011;
        assert_eq!(xs[t5], POS_SCORE[3][1]);
        assert_eq!(os[t5], POS_SCORE[3][1]);

        let t6 = 0b_000000000_101000100;
        assert!(xs[t6] == SC_LIMITS[1]);
        assert_eq!(os[t6], POS_SCORE[3][2]);
    }
}