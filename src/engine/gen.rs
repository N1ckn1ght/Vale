use crate::frame::lookups::WIN_LOOKUP;

pub const SHIFT_POS: u8 = 0;
pub const SHIFT_UPO: u8 = 4;
pub const SHIFT_THR: u8 = 8;
pub const SHIFT_REA: u8 = 12;
pub const MASK_CNT: u16 = 0b1111;

// (x_maps, o_maps)
pub fn gen_maps() -> (Vec<u16>, Vec<u16>) {
    let mut x_maps = vec![0; 0b1000000000000000000];
    let mut o_maps = vec![0; 0b1000000000000000000];
    
    for permut in 0..3_u32.pow(9) {
        let mut xbits: u16 = 0;
        let mut obits: u16 = 0;

        let mut p = permut;
        let mut bit = 1;
        while p != 0 {
            let br = p % 3;
            if br == 1 {
                xbits |= bit;
            } else if br == 2 {
                obits |= bit;
            }
            bit <<= 1;
            p /= 3;
        }

        let mut x_pos = 0;
        let mut x_upo = 0;
        let mut x_thr = 0;
        let mut o_pos = 0;
        let mut o_upo = 0;
        let mut o_thr = 0;

        let mut illegal = false;

        for lookup in WIN_LOOKUP {
            if lookup & obits == 0 {
                x_pos += 1;
                let xs = (lookup & xbits).count_ones();
                match xs {
                    0 => {},
                    1 => {
                        x_upo += 1;
                    },
                    2 => {
                        x_upo += 1;
                        x_thr += 1;
                    },
                    _ => {
                        illegal = true;
                    }
                }
            }            
            if lookup & xbits == 0 {
                o_pos += 1;
                let os = (lookup & obits).count_ones();
                match os {
                    0 => {},
                    1 => {
                        o_upo += 1;
                    },
                    2 => {
                        o_upo += 1;
                        o_thr += 1;
                    },
                    _ => {
                        illegal = true;
                    }
                }
            }
            if illegal {    // aka if this board is already won by someone
                break;
            }
        }

        if illegal {
            x_pos = 0;
            x_upo = 0;
            x_thr = 0;
            o_pos = 0;
            o_upo = 0;
            o_thr = 0;
        }

        // bleh
        let key = ((obits as u32) << 9) | xbits as u32;
        let x_value = (x_pos << SHIFT_POS) | (x_upo << SHIFT_UPO) | (x_thr << SHIFT_THR);
        let o_value = (o_pos << SHIFT_POS) | (o_upo << SHIFT_UPO) | (o_thr << SHIFT_THR);
        x_maps[key as usize] = x_value;
        o_maps[key as usize] = o_value;
    }

    (x_maps, o_maps)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_maps_manual_sequential_test() {
        let maps = gen_maps();

        let keys = [
            0b000000000000000000,
            0b000000000000000001,
            0b000000001000000000,
            0b000000000000000010,
            0b000000000000000011,
            0b000000001000000010,
            0b000000010000000000,
            0b000000010000000001,
            0b000000011000000000,
            0b000000000000000100
        ];
        let values = [
            (8, 0, 0, 8, 0, 0),
            (8, 3, 0, 5, 0, 0),
            (5, 0, 0, 8, 3, 0),
            (8, 2, 0, 6, 0, 0),
            (8, 4, 1, 4, 0, 0),
            (5, 1, 0, 6, 2, 0),
            (6, 0, 0, 8, 2, 0),
            (6, 2, 0, 5, 1, 0),
            (4, 0, 0, 8, 4, 1),
            (8, 3, 0, 5, 0, 0)
        ];

        for (i, key) in keys.into_iter().enumerate() {
            // println!("iter={}, key=[{:018b}]\nx=[{:09b}], o=[{:09b}]\n", i, key, maps.0[key], maps.1[key]);
            let x_pos = (maps.0[key] >> SHIFT_POS) & MASK_CNT;
            let x_upo = (maps.0[key] >> SHIFT_UPO) & MASK_CNT;
            let x_thr = (maps.0[key] >> SHIFT_THR) & MASK_CNT;
            let o_pos = (maps.1[key] >> SHIFT_POS) & MASK_CNT;
            let o_upo = (maps.1[key] >> SHIFT_UPO) & MASK_CNT;
            let o_thr = (maps.1[key] >> SHIFT_THR) & MASK_CNT;
            assert_eq!(x_pos, values[i].0);
            assert_eq!(x_upo, values[i].1);
            assert_eq!(x_thr, values[i].2);
            assert_eq!(o_pos, values[i].3);
            assert_eq!(o_upo, values[i].4);
            assert_eq!(o_thr, values[i].5);
        }
    }

    #[test]
    fn gen_maps_manual_random_test() {
        let maps = gen_maps();

        let keys = [
            0b000010000100000001,
            0b001010000100000001,
        ];
        let values = [
            (4, 4, 0, 3, 3, 0),
            (2, 2, 0, 3, 3, 1),
        ];
        
        for (i, key) in keys.into_iter().enumerate() {
            // println!("iter={}, key=[{:018b}]\nx=[{:09b}], o=[{:09b}]\n", i, key, maps.0[key], maps.1[key]);
            let x_pos = (maps.0[key] >> SHIFT_POS) & MASK_CNT;
            let x_upo = (maps.0[key] >> SHIFT_UPO) & MASK_CNT;
            let x_thr = (maps.0[key] >> SHIFT_THR) & MASK_CNT;
            let o_pos = (maps.1[key] >> SHIFT_POS) & MASK_CNT;
            let o_upo = (maps.1[key] >> SHIFT_UPO) & MASK_CNT;
            let o_thr = (maps.1[key] >> SHIFT_THR) & MASK_CNT;
            assert_eq!(x_pos, values[i].0);
            assert_eq!(x_upo, values[i].1);
            assert_eq!(x_thr, values[i].2);
            assert_eq!(o_pos, values[i].3);
            assert_eq!(o_upo, values[i].4);
            assert_eq!(o_thr, values[i].5);
        }
    }
}