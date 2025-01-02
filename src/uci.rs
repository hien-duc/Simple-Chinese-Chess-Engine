use std::io::{self, BufRead, Write};
use crate::board::Board;
use crate::search::find_best_move;

pub struct UCIEngine {
    board: Board,
    running: bool,
}

impl UCIEngine {
    pub fn new() -> Self {
        UCIEngine {
            board: Board::new(),
            running: true,
        }
    }

    fn process_position(&mut self, tokens: &[String]) {
        if tokens.len() < 2 {
            println!("Error: position command requires more arguments");
            println!("Usage: position startpos");
            println!("       position startpos moves e2e4 e7e5 ...");
            println!("       position fen <fenstring>");
            return;
        }
        
        match tokens[1].as_str() {
            "fen" => {
                if tokens.len() >= 8 {
                    let fen = tokens[2..8].join(" ");
                    match Board::from_fen(&fen) {
                        Ok(new_board) => {
                            self.board = new_board;
                            println!("Position set from FEN successfully");
                            
                            // apply any moves after the FEN if present
                            if tokens.len() > 9 && tokens[8] == "moves" {
                                println!("Applying moves: {:?}", &tokens[9..]);
                                for move_str in tokens[9..].iter() {
                                    let from = (
                                        (move_str.chars().nth(1).unwrap() as u8 - b'0') as usize,
                                        (move_str.chars().nth(0).unwrap() as u8 - b'a') as usize,
                                    );
                                    let to = (
                                        (move_str.chars().nth(3).unwrap() as u8 - b'0') as usize,
                                        (move_str.chars().nth(2).unwrap() as u8 - b'a') as usize,
                                    );
                                    self.board.make_move(from, to);
                                }
                            }
                        }
                        Err(e) => println!("Error parsing FEN: {}", e),
                    }
                } else {
                    println!("Error: Invalid FEN string - not enough parts");
                    println!("Usage: position fen <fen_parts> [moves <move1> <move2> ...]");
                    println!("FEN should have 6 parts: position pieces active_color castling en_passant halfmove fullmove");
                }
            }
            "startpos" => {
                println!("Setting up initial position...");
                self.board.setup_initial_position();
                if tokens.len() > 3 && tokens[2] == "moves" {
                    println!("Applying moves: {:?}", &tokens[3..]);
                    // starting with moves if haved
                    for move_str in tokens[3..].iter() {
                        let from = (
                            (move_str.chars().nth(1).unwrap() as u8 - b'0') as usize,
                            (move_str.chars().nth(0).unwrap() as u8 - b'a') as usize,
                        );
                        let to = (
                            (move_str.chars().nth(3).unwrap() as u8 - b'0') as usize,
                            (move_str.chars().nth(2).unwrap() as u8 - b'a') as usize,
                        );
                        if !self.board.make_move(from, to) {
                            println!("Error: Invalid move {}", move_str);
                            break;
                        }
                    }
                }
                // show current board
                //println!("\nCurrent position:");
                // println!("{}", self.board);
            }
            _ => {
                println!("Error: Unknown position subcommand");
                println!("Usage: position startpos");
                println!("       position startpos moves e2e4 e7e5 ...");
                println!("       position fen <fenstring>");
            }
        }
        io::stdout().flush().unwrap();
    }

    fn process_go(&self) {
        println!("Calculating best move...");
        if let Some(best_move) = find_best_move(&self.board) {
            println!("bestmove {}", best_move);
        } else {
            println!("bestmove none");
        }
        io::stdout().flush().unwrap();
    }

    pub fn main_loop(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        
        for line in stdin.lock().lines() {
            if let Ok(input) = line {
                let tokens: Vec<String> = input
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                
                if tokens.is_empty() {
                    continue;
                }

                println!("Received command: {}", tokens[0]);
                match tokens[0].as_str() {
                    "uci" => {
                        println!("id name XiangqiEngine");
                        println!("id author Hien Duc");
                        println!("option name Hash type spin default 16 min 1 max 1024");
                        println!("option name Style type combo default normal var solid var normal var risky");
                        println!("uciok");
                        stdout.flush().unwrap();
                    }
                    "isready" => {
                        println!("readyok");
                        stdout.flush().unwrap();
                    }
                    "position" => self.process_position(&tokens),
                    "go" => self.process_go(),
                    "ucinewgame" => {
                        self.board = Board::new();
                        println!("info string New game started");
                        stdout.flush().unwrap();
                    },
                    "quit" => {
                        println!("Goodbye!");
                        self.running = false;
                        break;
                    }
                    // "d" | "display" => {
                    //     println!("\nCurrent position:");
                    //     println!("{}", self.board);
                    //     stdout.flush().unwrap();
                    // }
                    _ => {
                        println!("Unknown command: {}", tokens[0]);
                        println!("Available commands:");
                        println!("  uci        - Initialize the engine");
                        println!("  isready    - Check if engine is ready");
                        println!("  ucinewgame - Reset the engine state for a new game");
                        println!("  position   - Set up a position");
                        println!("  go         - Start calculating");
                        // println!("  d          - Display current position");
                        println!("  quit       - Exit the engine");
                        stdout.flush().unwrap();
                    }
                }
            }
        }
    }
}