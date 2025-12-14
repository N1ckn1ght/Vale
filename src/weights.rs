use crate::lookups::gen_local_maps;


static POS_SCORE: [[i16; 9]; 4] = [
    [   0, 300, 300, 300, 300, 300, 300, 300, 300],
    [   0,  64, 128,  32,   8,   8,   8,   8,   8],  // chance degradation; if you avoid winning and stack attacks, then it's not possible to win in the first place (Zugzwang)
    [   0,   8,  16,  24,  32,  40,   8,   8,   8],
    [   0,   1,   2,   3,   4,   5,   6,   8,   8]
];

fn gen_local_scores(xscores: &mut [u16], oscores: &mut [i16]) {
    let mut xlocal = [0; 262144];
    let mut olocal = [0; 262144];
    gen_local_maps(&mut xlocal, &mut olocal);
    for i in 0..262144 {
        
    }
}
