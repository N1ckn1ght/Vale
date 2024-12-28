use crate::frame::lookups::WIN_LOOKUP;

pub const SHIFT_X_POS: u8 = 0;
pub const SHIFT_X_UPO: u8 = 4;
pub const SHIFT_X_THR: u8 = 8;
pub const SHIFT_O_POS: u8 = 12;
pub const SHIFT_O_UPO: u8 = 16;
pub const SHIFT_O_THR: u8 = 20;
pub const MASK_POS: u16 = 0b1111;

pub fn gen_maps() -> [u16; 0b1000000000000000000] {
    let mut maps = [0; 0b1000000000000000000];
    
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
        // o_pos = 8 - x_upo
        let mut x_upo = 0;
        // o_upo = 8 - x_pos
        let mut x_thr = 0;
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
                let os = (lookup & obits).count_ones();

                match os {
                    0 => {},
                    1 => {},
                    2 => {
                        o_thr += 1;
                    },
                    _ => {
                        illegal = true;
                    }
                }
            }
            if illegal {
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
        let value = (x_pos << SHIFT_X_POS) | (x_upo << SHIFT_X_UPO) | (x_thr << SHIFT_X_THR) | (o_pos << SHIFT_O_POS) | (o_upo << SHIFT_O_UPO) | (o_thr << SHIFT_O_THR);
        maps[key as usize] = value;
    }

    maps
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_maps_manual_values_test() {
        let maps = gen_maps();

       assert_eq!(maps[0],  0);
       assert_eq!(maps[0],  0);
       assert_eq!(maps[0],  0);
       assert_eq!(maps[0],  0);
       assert_eq!(maps[0],  0);
       assert_eq!(maps[0],  0);
       assert_eq!(maps[0],  0);
       assert_eq!(maps[0],  0);
    }
}