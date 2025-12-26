use std::io::stdin;
use crate::{bitboard::GetBit, board::{transform_move, Board, ERR_MOV}, engine::eval, lookups::DIV_LOOKUP};


pub fn user_box() {
    let mut board = Board::default();

    let mut hint_used = false;
    let mut show_movegen = false;
    let mut show_bestline = false;
    let mut show_history = true;
    let mut show_ken = true;
    let mut eval_as_is = false;

    loop {
        // print current status
        print_board(&board);
        println!();

        let legals = board.generate_legal_moves();
        let game_ended = board.status < 3;
        if game_ended {
            match board.status {
                0 => {println!("Game ended | Victory: X")},
                1 => {println!("Game ended | Victory: O")},
                2 => {println!("Game ended | Draw")},
                _ => {}
            }
        } else {
            if show_movegen {
                println!("Legal moves: {:81b}", legals);
            }
        }
        if show_ken {
            println!("KEN: {}", board.export_ken());
        }
        if show_history {
            println!("History: {}", board.export_history(1));
        }
        if !hint_used {
            println!("Hint: type \"help\" to see the list of commands.");
            hint_used = true;
        }
        println!();

        loop {
            let mut input_line = String::new();
            stdin().read_line(&mut input_line).expect("Failed to read a line");
            let mut cmd = input_line.trim().split(' ').collect::<Vec<&str>>();

            /* quick move */
            let b = cmd[0].as_bytes();
            if b.len() == 2 {  // && (b'a'..=b'i').contains(&b[0]) && (b'1'..=b'9').contains(&b[1]) {
                if cmd.len() > 1 {
                    cmd[1] = cmd[0];
                } else {
                    cmd.push(cmd[0]);
                }
                cmd[0] = "move";
            }

            match cmd[0] {
                "move" => {
                    let mov = transform_move(cmd[1], legals);
                    if mov != ERR_MOV {
                        board.make_move(mov);
                        break;
                    }
                },
                "undo" => {
                    if !board.moves.is_empty() {
                        board.undo_move();
                        break;
                    }
                    println!("Unable to undo a move: No move history left!");
                },
                "engine" => {
                    // TODO
                },
                "bestline" => {
                    show_bestline = !show_bestline;
                    if show_bestline {
                        println!("Analysis line will be shown.");
                    } else {
                        println!("Analysis line will remain hidden.");
                    }
                },
                "eval" => {
                    if cmd.len() > 1 {
                        if cmd[1].contains('s') {
                            eval_as_is = !eval_as_is;
                            if eval_as_is {
                                println!("Eval will be printed as is.");
                            } else {
                                println!("Eval will be printed in similar to chess manner and values.");
                            }
                        } else {
                            let depth = cmd[1].parse::<u8>().unwrap();
                            let ev = eval(&board);
                            let score = if depth != 0 {
                                // TODO
                                "TODO".to_string()
                            } else {
                                format_eval(ev)
                            };
                            if eval_as_is {
                                println!("Score (depth {depth}): {}", ev);
                            } else {
                                println!("Score (depth {depth}): {}", score);
                            }
                        }
                    } else {
                        println!("Specify params. Maybe you want 'eval 1', 'eval 0' or 'eval switch'?");
                    }
                },
                "history" => {
                    show_history = !show_history;
                    if show_history {
                        println!("Move history will be visible.");
                    } else {
                        println!("Move history will be hidden.");
                    }
                },
                "ken" => {
                    show_ken = !show_ken;
                    if show_ken {
                        println!("KEN will be visible.");
                    } else {
                        println!("KEN will be hidden.");
                    }
                },
                "movegen" => {
                    show_movegen = !show_movegen;
                    if show_movegen {
                        println!("Movegen string will be shown.");
                    } else {
                        println!("Movegen string will remain hidden.");
                    }
                },
                "import" => {
                    if cmd[1].contains("-") {
                        if cmd.len() < 3 {
                            cmd.push("-");
                        }
                        board.import_ken(&(cmd[1].to_owned() + " " + cmd[2]));
                    } else {
                        board.import_history(&cmd.iter().skip(1).cloned().collect::<Vec<_>>().join(" "));
                    }
                    println!("Import successful.");
                    break;
                },
                "export" => {
                    println!("KEN: {}", board.export_ken());
                    println!("PGN: {}", board.export_history(1));
                    if cmd.len() > 1 {
                        println!("locals[0]: {:128b}", board.locals[0]);
                        println!("locals[1]: {:128b}", board.locals[1]);
                        println!("global[0]: {:16b}", board.global[0]);
                        println!("global[1]: {:16b}", board.global[1]);
                        println!("global[2]: {:16b}", board.global[2]);
                        println!("lwbits:    {:128b}", board.lwbits);
                        println!("turn:      {}", board.turn);
                        println!("status:    {}", board.status);
                    }
                },
                "reload" => {
                    break;
                },
                "clear" => {
                    board.clear();
                    break;
                },
                "quit" => {
                    return;
                },
                "help" => {
                    println!("List of commands:");
                    println!("___");
                    println!("a1             - make move (a1-i9), short and convenient form!");
                    println!("move a1        - make move (a1-i9)");
                    println!("undo           - undo last move");
                    println!("engine 10      - ask engine to make move (depth in half-moves, 1-81, rec. max. 10)");
                    println!("bestline       - show/hide proposed bestline by engine after engine/eval calls");
                    println!("eval 1         - evaluate position (depth in half-moves, 0-81, rec. max. 8, 0 won't show any line)");
                    println!("eval switch    - switch between printing chess-like score format and printing eval as is");
                    println!("history        - hide/show move history after each move");
                    println!("ken            - hide/show Kochergin-Efimov Notation after each move");
                    println!("movegen        - show/hide movegen string after each move");
                    println!("import <ken>   - import position from ken");
                    println!("import <moves> - import position from move history");
                    println!("export         - export position");
                    println!("export d       - export position with debug information");
                    println!("reload         - show board again");
                    println!("clear          - clear board");
                    println!("quit           - shutdown application (bro just close the window)");
                    println!("___");
                    println!("Note: try not to make any typos, this interface is freaky.");
                },
                _ => {
                    println!("Unknown command?");
                }
            }
            println!();
        }
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
    let sign = if eval > 0 {"+"} else if eval < 0 {"-"} else {""};
    let mut abs = eval.abs();
    abs /= 6;  // make it similar to chess for human players to look at?
    let fpart = abs / 100;
    let spart = abs % 100;
    format!("{}{}.{:02}", sign, fpart, spart)
}