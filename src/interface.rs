use std::io::stdin;
use crate::{bitboard::GetBit, board::Board, engine::eval, lookups::DIV_LOOKUP};


pub fn user_box() {
    let mut board = Board::default();
    // board.import_history("e5 d6 b8 e6 e8 f6 h8");
    // let mut board = Board::init("9-9-9-9-9-9-9-9-9 -");
    // board.import("9-9-9-9-9-9-9-9-7ox b1");
    while board.status > 2 {
        // print current status
        print_board(&board);
        let legals = board.generate_legal_moves();
        println!("movegen string: {}", format!("{:81b}", legals));
        println!("zero-depth eval score: {}", format_eval(eval(&board, &legals)));

        // return;

        // ask for user input
        // let mut input_line = String::new();
        // stdin().read_line(&mut input_line).expect("Failed to read a line");
        // input_line.trim().split_whitespace().next().map(|s| s.to_string());

        let mov = user_input_move(legals);
        println!("move index accepted: {}", mov);
        board.make_move(mov);
        // println!("mov history test 0");
        // println!("{}", board.export_history(0));
        // println!("mov history test 1");
        // println!("{}", board.export_history(1));
        // println!("mov history test 2");
        // println!("{}", board.export_history(2));
    }
}

pub fn print_board(board: &Board) {
    let legals = board.generate_legal_moves();
    for rank in (0..9).rev() {
        match rank {
            2 | 5 | 8 => {
                println!("  +-------+-------+-------+");
                print!("{} ", rank + 1);
            },
            _ => {
                print!("{} ", rank + 1);
            }
        }
        for file in 0..9 {
            let realbit = grb(rank, file);
            match file {
                0 | 3 | 6 => {
                    print!("| {} ", get_char(board, legals, realbit));
                },
                8 => {
                    print!("{} |\n", get_char(board, legals, realbit));
                }
                _ => {
                    print!("{} ", get_char(board, legals, realbit));
                }
            }
        }
    }
    println!("  +-------+-------+-------+");
    println!("    a b c   d e f   g h i");
}

pub fn get_char(board: &Board, legals: u128, bit: u8) -> char {
    if board.global[0].get_bit(DIV_LOOKUP[bit as usize]) != 0 {
        return 'X';
    }
    if board.global[1].get_bit(DIV_LOOKUP[bit as usize]) != 0 {
        return 'O';
    }
    if board.global[2].get_bit(DIV_LOOKUP[bit as usize]) != 0 {
        return '/';
    }
    if board.locals[0].get_bit(bit) != 0 {
        return 'x';
    }
    if board.locals[1].get_bit(bit) != 0 {
        return 'o';
    }
    if legals.get_bit(bit) != 0 {
        return '.'
    }
    ' '
}

pub fn user_input_move(legals: u128) -> u8 {
    loop {
        let mut input_line = String::new();
        stdin().read_line(&mut input_line).expect("Failed to read a line");
        let chars = input_line.trim().chars();
        if chars.count() != 2 {
            println!("#DEBUG Wrong user input: must be from a1 to i9 (expected 2 chars)");
            continue;
        }
        let mut chars = input_line.trim().chars();
        let file = chars.next().unwrap().to_ascii_lowercase();
        let rank = chars.next().unwrap();
        if !(('a'..='i').contains(&file) && ('1'..='9').contains(&rank)) {
            println!("#DEBUG Wrong user input: must be from a1 to i9");
            continue;
        }
        let realbit = grb((rank as u32 - '1' as u32) as u8, (file as u32 - 'a' as u32) as u8);
        if legals.get_bit(realbit) == 0 {
            println!("#DEBUG Wrong user input: illegal move");
            continue;
        }
        return realbit;
    }
}

#[inline]
fn grb(rank: u8, file: u8) -> u8 {
    (rank / 3) * 27 + (rank % 3) * 3 + (file / 3) * 9 + (file % 3)
}

fn format_eval(eval: i16) -> String {
    if eval == 0 {
        return "0".to_string();
    }
    let sign = if eval > 0 {"+"} else {"-"};
    let abs = eval.abs();
    let fpart = abs / 100;
    let spart = abs % 100;
    format!("{}{}.{:02}", sign, fpart, spart)
}