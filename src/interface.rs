use std::{cmp::max, io::{self, stdin}, sync::mpsc::channel, thread, time::Duration};
use crate::{bitboard::GetBit, board::{Board, ERR_MOV, transform_move, transform_move_back}, engine::{Engine, eval}, lookups::DIV_LOOKUP};


pub fn format_eval(eval: i16) -> String {
    let sign = match eval {
        x if x > 0 => "+",
        x if x < 0 => "-",
        _ => "",
    };
    let mut abs = eval.abs();
    abs /= 6;  // make it similar to chess for human players to look at?
    let fpart = abs / 100;
    let spart = abs % 100;
    format!("{}{}.{:02}", sign, fpart, spart)
}

pub fn user_box() {
    let mut board = Board::default();

    let mut hint_used = false;
    let mut show_movegen = false;
    let mut show_history = true;
    let mut show_ken = true;

    let mut tdswitch = false;
    let mut time= 1000;
    let mut depth = 10;
    let mut engineplays = false;
    let mut auto: u16 = 0b00;

    let mut engine = Engine::default();

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
            auto = 0;
        } else if show_movegen {
            println!("Legal moves: {:81b}", legals);
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
            let mut cmd: Vec<&str>;
            let mut input_line = String::new();

            if auto.get_bit(board.turn as u8) != 0 {
                cmd = vec!["go"];
            } else {
                stdin().read_line(&mut input_line).expect("Failed to read a line");
                cmd = input_line.trim().split(' ').collect::<Vec<&str>>();
            }

            /* quick move */
            let b = cmd[0].as_bytes();
            if b.len() == 2 && b[1] != b'o' {  // && (b'a'..=b'i').contains(&b[0]) && (b'1'..=b'9').contains(&b[1]) {
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
                    if auto != 0 {
                        println!("Autoplay disabled.");
                        auto = 0;
                    }
                    if !board.moves.is_empty() {
                        board.undo_move();
                        break;
                    }
                    println!("Unable to undo a move: No move history left!");
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
                    if cmd.len() > 1 && (cmd[1].contains("engine") || cmd[1].contains("2")) {
                        println!("List of engine commands:");
                        println!("___");
                        println!("go             - evaluate position for the set time or depth (same as \"eval\")");
                        println!("time 1000      - set the amount of time for engine to ponder in ms");
                        println!("                 time and depth are mutually exclusive, one cancels another");
                        println!("depth 5        - set the depth for engine to go for");
                        println!("                 don't go for more than 8 or you'll die of age");
                        println!("                 LMR is used means depth X is not guaranteed to see all lines");
                        println!("                 \"depth 0\" will show the pure eval of the current position");
                        println!();
                        println!("engineplays    - make engine automatically play the best move after thinking");
                        println!("auto x         - play auto for X");
                        println!("auto o         - play auto for O");
                        println!("auto d         - autoplay for both sides, cannot be stopped until the game is finished");
                        println!();
                        println!("post           - hide/show engine output (eval score and principal variation)");
                        println!("evm            - switch between chess-like value and real eval engine sees");
                        println!("___");
                    } else {
                        println!("List of commands:");
                        println!("___");
                        println!("a1             - make move (a1-i9), short and convenient form!");
                        println!("move a1        - make move (a1-i9)");
                        println!("undo           - undo last move");
                        println!();
                        println!("help engine    - help page on engine commands (there are a lot)");
                        println!();
                        println!("import <ken>   - import position from ken");
                        println!("import <moves> - import position from move history");
                        println!("export         - export position");
                        println!("export d       - export position with debug information");
                        println!();
                        println!("history        - hide/show move history after each move");
                        println!("ken            - hide/show Kochergin-Efimov Notation after each move");
                        println!("movegen        - show/hide movegen string after each move");
                        println!();
                        println!("reload         - show board again");
                        println!("clear          - clear board");
                        println!("quit           - shutdown application (bro just close the window)");
                        println!("___");
                        println!("Note: try not to make any typos, this interface is freaky.");
                    }
                },
                "time" => {
                    time = cmd[1].parse::<u128>().unwrap();
                    tdswitch = false;
                },
                "depth" => {
                    depth = cmd[1].parse::<usize>().unwrap() * 2;
                    tdswitch = true;
                },
                "post" => {
                    engine.post = !engine.post;
                    if engine.post {
                        println!("Engine eval and bestline will be visible.");
                    } else {
                        println!("Engine eval and bestline will be hidden.");
                    }
                },
                "evm" => {
                    engine.evm = !engine.evm;
                    if engine.evm {
                        println!("Eval will be shown as is.");
                    } else {
                        println!("Eval will be shown modified.");
                    }
                },
                "go" | "eval" => {
                    if tdswitch && depth == 0 {
                        let ev = eval(&board);
                        if engine.evm {
                            println!("Eval: {}", ev);
                        } else {
                            let score = format_eval(ev);
                            println!("Eval: {}", score);
                        }
                    } else {
                        let (mov, sc) = if tdswitch {
                            run_engine(&mut engine, &mut board, None, Some(depth))
                        } else {
                            let tp = time / 100;
                            run_engine(&mut engine, &mut board, Some(max(time - tp, 50)), None)
                        };
                        print!("Best move: {}", transform_move_back(mov));
                        if engine.post {
                            if engine.evm {
                                print!(", score: {}", sc);
                            } else {
                                let score = format_eval(sc);
                                print!(", score: {}", score);
                            }
                        }
                        println!();
                        if engineplays {
                            board.make_move(mov);
                            break;
                        }
                    }
                },
                "engineplays" => {
                    engineplays = !engineplays;
                    if engineplays {
                        println!("Engine will play the move after thinking.");
                    } else {
                        println!("Engine won't play the move after thinking.");
                    }
                },
                "auto" => {
                    if cmd.len() > 1 {
                        let mut correct = true;
                        if cmd[1].contains('x') {
                            println!("Autoplay set for X");
                            auto = 0b01;
                        } else if cmd[1].contains('o') {
                            println!("Autoplay set for O");
                            auto = 0b10;
                        } else if cmd[1].contains('d') {
                            println!("Autplay set for both sides! Wait for the game end.");
                            auto = 0b11;
                        } else {
                            correct = false;
                        }
                        if correct {
                            if !engineplays {
                                println!("Engine plays set to true.");
                                engineplays = true;
                            }
                        } else {
                            println!("Wrong argument for autoplay?");
                        }
                    } else {
                        println!("Missing argument for autoplay.");
                    }
                },
                _ => {
                    println!("Unknown command?");
                }
            }
            println!();
        }
    }
}

fn run_engine(engine: &mut Engine, board: &mut Board, tl: Option<u128>, td: Option<usize>) -> (u8, i16) {
    let (mv, sc) = engine.search(board, tl, td);
    (mv, sc)
}

fn print_board(board: &Board) {
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

fn get_char(board: &Board, legals: u128, bit: u8) -> char {
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

#[inline]
fn grb(rank: u8, file: u8) -> u8 {
    (rank / 3) * 27 + (rank % 3) * 3 + (file / 3) * 9 + (file % 3)
}