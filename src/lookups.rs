/* General lookups */

pub const DIV_LOOKUP: [u8; 81] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 
                                  1, 1, 1, 1, 1, 1, 1, 1, 1,
                                  2, 2, 2, 2, 2, 2, 2, 2, 2,
                                  3, 3, 3, 3, 3, 3, 3, 3, 3,
                                  4, 4, 4, 4, 4, 4, 4, 4, 4,
                                  5, 5, 5, 5, 5, 5, 5, 5, 5,
                                  6, 6, 6, 6, 6, 6, 6, 6, 6,
                                  7, 7, 7, 7, 7, 7, 7, 7, 7,
                                  8, 8, 8, 8, 8, 8, 8, 8, 8];

pub const MOD_LOOKUP: [u8; 81] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 
                                  0, 1, 2, 3, 4, 5, 6, 7, 8,
                                  0, 1, 2, 3, 4, 5, 6, 7, 8,
                                  0, 1, 2, 3, 4, 5, 6, 7, 8,
                                  0, 1, 2, 3, 4, 5, 6, 7, 8,
                                  0, 1, 2, 3, 4, 5, 6, 7, 8,
                                  0, 1, 2, 3, 4, 5, 6, 7, 8,
                                  0, 1, 2, 3, 4, 5, 6, 7, 8,
                                  0, 1, 2, 3, 4, 5, 6, 7, 8];

pub const SUB_LOOKUP: [u128; 9] = [0b000000000000000000000000000000000000000000000000000000000000000000000000111111111,
                                   0b000000000000000000000000000000000000000000000000000000000000000111111111000000000,
                                   0b000000000000000000000000000000000000000000000000000000111111111000000000000000000,
                                   0b000000000000000000000000000000000000000000000111111111000000000000000000000000000,
                                   0b000000000000000000000000000000000000111111111000000000000000000000000000000000000,
                                   0b000000000000000000000000000111111111000000000000000000000000000000000000000000000,
                                   0b000000000000000000111111111000000000000000000000000000000000000000000000000000000,
                                   0b000000000111111111000000000000000000000000000000000000000000000000000000000000000,
                                   0b111111111000000000000000000000000000000000000000000000000000000000000000000000000];

pub const WIN_LOOKUP: [u16; 8] = [0b000000111, 0b000111000, 0b001001001, 0b001010100, 0b010010010, 0b100010001, 0b100100100, 0b111000000];

// pub const DIAG_LOOKUP: [u16; 2] = [0b001010100, 0b100010001];
// pub const SIDE_LOOKUP: [u16; 4] = [0b000000111, 0b001001001, 0b100100100, 0b111000000];
// pub const CENT_LOOKUP: [u16; 2] = [0b000111000, 0b010010010];

pub const WIN_LOOKUP_INDEXED: [u16; 24] = [0b000000111, 0b001001001, 0b100010001,
                                           0b000000111, 0b010010010,
                                           0b000000111, 0b001010100, 0b100100100,
                                           0b000111000, 0b001001001,
                                           0b000111000, 0b001010100, 0b010010010, 0b100010001,
                                           0b000111000, 0b100100100,
                                           0b001001001, 0b001010100, 0b111000000,
                                           0b010010010, 0b111000000,
                                           0b100010001, 0b100100100, 0b111000000];

pub const WIN_LOOKUP_INDICES: [[usize; 2]; 9] = [[0, 3], [3, 2], [5, 3], [8, 2], [10, 4], [14, 2], [16, 3], [19, 2], [21, 3]];

// pub const BAD_LOOKUP: [u16; 4] = [0b001100010, 0b010001100, 0b010100001, 0b100001010];


/* Eval lookups */

pub const POS_CNT: [u8; 4] = [12, 8, 4, 0];
pub const POS_MASK: u16 = 0b1111;

pub fn gen_local_maps(xlocal: &mut [u16], olocal: &mut [u16]) {
    for permut in 0usize..262144 {
        let xbits = (permut & 0b111111111) as u16;
        let obits = ((permut >> 9) & 0b111111111) as u16;

        // impossible
        if xbits & obits != 0 {
            continue;
        }

        let mut x_left = [0; 4];
        let mut o_left = [0; 4];

        for lookup in WIN_LOOKUP {
            let maskx = xbits & lookup;
            let masko = obits & lookup;
            
            if maskx != 0 && masko != 0 {
                continue;
            }
            if maskx == 0 && masko == 0 {
                x_left[3] += 1;
                o_left[3] += 1;
                continue
            }
            if maskx != 0 {
                match maskx.count_ones() {
                    1 => { x_left[2] += 1; },
                    2 => { x_left[1] += 1; },
                    3 => {
                        x_left[0] = 1;
                        break;
                    },
                    _ => {}
                }
            } else {
                match masko.count_ones() {
                    1 => { o_left[2] += 1; },
                    2 => { o_left[1] += 1; },
                    3 => {
                        o_left[0] = 1;
                        break;
                    },
                    _ => {}
                }
            }
        }

        if x_left[0] != 0 {
            xlocal[permut] = 1 << POS_CNT[0];
            continue;
        }
        if o_left[0] != 0 {
            olocal[permut] = 1 << POS_CNT[0];
            continue;
        }
        xlocal[permut] = (x_left[1] << POS_CNT[1]) | (x_left[2] << POS_CNT[2]) | (x_left[3] << POS_CNT[3]);
        olocal[permut] = (o_left[1] << POS_CNT[1]) | (o_left[2] << POS_CNT[2]) | (o_left[3] << POS_CNT[3]);
    }
}


#[cfg(test)]
mod tests {
    use crate::bitboard::GetBit;
    use super::*;

    #[test]
    fn gen_local_maps_test() {
        let mut xl = [0; 262144];
        let mut ol = [0; 262144];
        gen_local_maps(&mut xl, &mut ol);

        let t1 = 0b_000000000_000000000;
        assert_eq!(xl[t1], 8 << POS_CNT[3]);
        assert_eq!(ol[t1], 8 << POS_CNT[3]);

        let t2 = 0b_000000010_000000001;
        assert_eq!(xl[t2], 4 << POS_CNT[3] | 2 << POS_CNT[2]);
        assert_eq!(ol[t2], 4 << POS_CNT[3] | 1 << POS_CNT[2]);

        let t3 = 0b_000000101_000010000;
        assert_eq!(xl[t3], 1 << POS_CNT[3] | 2 << POS_CNT[2]);
        assert_eq!(ol[t3], 1 << POS_CNT[3] | 2 << POS_CNT[2] | 1 << POS_CNT[1]);

        let t4 = 0b_101000000_000010000;
        assert_eq!(xl[t4], 1 << POS_CNT[3] | 2 << POS_CNT[2]);
        assert_eq!(ol[t4], 1 << POS_CNT[3] | 2 << POS_CNT[2] | 1 << POS_CNT[1]);

        let t5 = 0b_000000100_011000000;
        assert_eq!(xl[t5], 2 << POS_CNT[3] | 2 << POS_CNT[2] | 1 << POS_CNT[1]);
        assert_eq!(ol[t5], 2 << POS_CNT[3] | 2 << POS_CNT[2]);

        let t6 = 0b_000010100_111000000;
        assert!(xl[t6].get_bit(POS_CNT[0]) != 0);

        let t7 = 0b_110101011_000000000;
        assert_eq!(xl[t7], 1 << POS_CNT[3]);
        assert_eq!(ol[t7], 1 << POS_CNT[3] | 7 << POS_CNT[1]);

        let t8 = 0b_110101011_000010000;
        assert_eq!(xl[t8], 1 << POS_CNT[2]);
        assert_eq!(ol[t8], 4 << POS_CNT[1]);

        let t9 = 0b_000000000_100011010;
        assert_eq!(xl[t9], 5 << POS_CNT[2] | 3 << POS_CNT[1]);
        assert_eq!(ol[t9], 0);

        let t10 = 0b_011100101_100011010;
        assert_eq!(xl[t10], 0);
        assert_eq!(ol[t10], 0);

        let t11 = 0b_001001000_110110000;
        assert_eq!(xl[t11], 1 << POS_CNT[3] | 3 << POS_CNT[1]);
        assert_eq!(ol[t11], 1 << POS_CNT[3] | 1 << POS_CNT[1]);

        let t12 = 0b_000000001_000000010;
        assert_eq!(xl[t12], 4 << POS_CNT[3] | 1 << POS_CNT[2]);
        assert_eq!(ol[t12], 4 << POS_CNT[3] | 2 << POS_CNT[2]);

        let t13 = 0b_000010000_000000101;
        assert_eq!(xl[t13], 1 << POS_CNT[3] | 2 << POS_CNT[2] | 1 << POS_CNT[1]);
        assert_eq!(ol[t13], 1 << POS_CNT[3] | 2 << POS_CNT[2]);

        let t14 = 0b_000010000_101000000;
        assert_eq!(xl[t14], 1 << POS_CNT[3] | 2 << POS_CNT[2] | 1 << POS_CNT[1]);
        assert_eq!(ol[t14], 1 << POS_CNT[3] | 2 << POS_CNT[2]);

        let t15 = 0b_011000000_000000100;
        assert_eq!(xl[t15], 2 << POS_CNT[3] | 2 << POS_CNT[2]);
        assert_eq!(ol[t15], 2 << POS_CNT[3] | 2 << POS_CNT[2] | 1 << POS_CNT[1]);

        let t16 = 0b_111000000_000010100;
        assert!(ol[t16].get_bit(POS_CNT[0]) != 0);

        let t17 = 0b_000000000_110101011;
        assert_eq!(xl[t17], 1 << POS_CNT[3] | 7 << POS_CNT[1]);
        assert_eq!(ol[t17], 1 << POS_CNT[3]);

        let t18 = 0b_000010000_110101011;
        assert_eq!(xl[t18], 4 << POS_CNT[1]);
        assert_eq!(ol[t18], 1 << POS_CNT[2]);

        let t19 = 0b_100011010_000000000;
        assert_eq!(xl[t19], 0);
        assert_eq!(ol[t19], 5 << POS_CNT[2] | 3 << POS_CNT[1]);

        let t20 = 0b_100011010_011100101;
        assert_eq!(xl[t20], 0);
        assert_eq!(ol[t20], 0);
    }
}