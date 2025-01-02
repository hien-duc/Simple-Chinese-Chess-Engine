use crate::board::{Board, Color, Piece};

const SOLDIER_VALUE: i32 = 30;
const CANNON_VALUE: i32 = 285;
const HORSE_VALUE: i32 = 270;
const ELEPHANT_VALUE: i32 = 120;
const ADVISOR_VALUE: i32 = 120;
const CHARIOT_VALUE: i32 = 600;
const GENERAL_VALUE: i32 = 6000;

// Piece-Square tables
const SOLDIER_BONUS_RED: [[i32; 9]; 10] = [
    [0,  0,  0,  0,  0,  0,  0,  0,  0],
    [0,  0,  0,  0,  0,  0,  0,  0,  0],
    [0,  0,  0,  0,  0,  0,  0,  0,  0],
    [2,  4,  6,  6,  6,  6,  6,  4,  2],
    [6,  8,  10, 12, 12, 12, 10, 8,  6],
    [10, 12, 14, 16, 18, 16, 14, 12, 10],
    [14, 16, 18, 20, 20, 20, 18, 16, 14],
    [18, 20, 22, 24, 24, 24, 22, 20, 18],
    [22, 24, 26, 28, 28, 28, 26, 24, 22],
    [26, 28, 30, 32, 32, 32, 30, 28, 26],
];

const CHARIOT_BONUS: [[i32; 9]; 10] = [
    [14, 14, 12, 18, 16, 18, 12, 14, 14],
    [16, 20, 18, 24, 26, 24, 18, 20, 16],
    [12, 12, 12, 18, 18, 18, 12, 12, 12],
    [12, 18, 16, 22, 22, 22, 16, 18, 12],
    [12, 14, 12, 18, 18, 18, 12, 14, 12],
    [12, 16, 14, 20, 20, 20, 14, 16, 12],
    [6,  10, 8,  14, 14, 14, 8,  10, 6],
    [4,  8,  6,  14, 12, 14, 6,  8,  4],
    [8,  4,  8,  16, 8,  16, 8,  4,  8],
    [-2, 10, 6,  14, 12, 14, 6,  10, -2],
];

const HORSE_BONUS: [[i32; 9]; 10] = [
    [4,  8,  16, 12, 4,  12, 16, 8,  4],
    [4,  10, 28, 16, 8,  16, 28, 10, 4],
    [12, 14, 16, 20, 18, 20, 16, 14, 12],
    [8,  24, 18, 24, 20, 24, 18, 24, 8],
    [6,  16, 14, 18, 16, 18, 14, 16, 6],
    [4,  12, 16, 14, 12, 14, 16, 12, 4],
    [2,  6,  8,  6,  10, 6,  8,  6,  2],
    [-2, 4,  4,  4,  4,  4,  4,  4,  -2],
    [0,  2,  4,  4,  -2, 4,  4,  2,  0],
    [0,  -4, 0,  0,  0,  0,  0,  -4, 0],
];

const CANNON_BONUS: [[i32; 9]; 10] = [
    [6,  4,  0,  -10,-12,-10,0,  4,  6],
    [2,  2,  0,  -4, -14,-4, 0,  2,  2],
    [2,  2,  0,  -10,-8, -10,0,  2,  2],
    [0,  0,  -2, 4,  10, 4,  -2, 0,  0],
    [0,  0,  0,  2,  8,  2,  0,  0,  0],
    [-2, 0,  4,  2,  6,  2,  4,  0,  -2],
    [0,  0,  0,  2,  4,  2,  0,  0,  0],
    [4,  0,  8,  6,  10, 6,  8,  0,  4],
    [0,  2,  4,  6,  6,  6,  4,  2,  0],
    [0,  0,  2,  6,  6,  6,  2,  0,  0],
];

const ADVISOR_BONUS: [[i32; 9]; 10] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 20,0, 20,0, 0, 0],
    [0, 0, 0, 0, 23,0, 0, 0, 0],
    [0, 0, 0, 20,0, 20,0, 0, 0],
];

const ELEPHANT_BONUS: [[i32; 9]; 10] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 20,0, 0, 0, 20,0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [18,0, 0, 0, 23,0, 0, 0, 18],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 20,0, 0, 0, 20,0, 0],
];

pub fn evaluate_position(board: &Board) -> i32 {
    let mut score = 0;
    let mut red_pieces = 0;
    let mut black_pieces = 0;

    // Heavy penalty for flying general (should never happen due to move validation, but just in case)
    if board.is_flying_general() {
        return if board.red_to_move { -50000 } else { 50000 };
    }

    // Check for potential flying general threat
    let (red_general_file, black_general_file) = find_general_files(board);
    if let (Some(red_file), Some(black_file)) = (red_general_file, black_general_file) {
        if red_file == black_file {
            // Penalize having generals on the same file
            score -= 50;
        }
    }

    // Evaluate material and position
    for rank in 0..10 {
        for file in 0..9 {
            if let Some((color, piece)) = board.squares[rank][file].piece {
                let mut piece_value = match piece {
                    Piece::Soldier => {
                        SOLDIER_VALUE + if color == Color::Red {
                            SOLDIER_BONUS_RED[rank][file]
                        } else {
                            SOLDIER_BONUS_RED[9 - rank][file]
                        }
                    },
                    Piece::Cannon => {
                        CANNON_VALUE + if color == Color::Red {
                            CANNON_BONUS[rank][file]
                        } else {
                            CANNON_BONUS[9 - rank][file]
                        }
                    },
                    Piece::Horse => {
                        HORSE_VALUE + if color == Color::Red {
                            HORSE_BONUS[rank][file]
                        } else {
                            HORSE_BONUS[9 - rank][file]
                        }
                    },
                    Piece::Elephant => {
                        ELEPHANT_VALUE + if color == Color::Red {
                            ELEPHANT_BONUS[rank][file]
                        } else {
                            ELEPHANT_BONUS[9 - rank][file]
                        }
                    },
                    Piece::Advisor => {
                        ADVISOR_VALUE + if color == Color::Red {
                            ADVISOR_BONUS[rank][file]
                        } else {
                            ADVISOR_BONUS[9 - rank][file]
                        }
                    },
                    Piece::Chariot => {
                        CHARIOT_VALUE + if color == Color::Red {
                            CHARIOT_BONUS[rank][file]
                        } else {
                            CHARIOT_BONUS[9 - rank][file]
                        }
                    },
                    Piece::General => GENERAL_VALUE,
                };

                // Count pieces for endgame detection
                if color == Color::Red {
                    red_pieces += 1;
                } else {
                    black_pieces += 1;
                }

                // Adjust value based on piece color
                if color == Color::Black {
                    piece_value = -piece_value;
                }

                score += piece_value;
            }
        }
    }

    // Endgame adjustments
    let total_pieces = red_pieces + black_pieces;
    if total_pieces <= 12 {  // Endgame threshold
        // Increase value of soldiers in endgame
        for rank in 0..10 {
            for file in 0..9 {
                if let Some((color, Piece::Soldier)) = board.squares[rank][file].piece {
                    score += if color == Color::Red { 10 } else { -10 };
                }
            }
        }
    }

    // Mobility evaluation
    let moves = crate::moves::generate_legal_moves(board);
    let mobility_bonus = (moves.len() as i32).saturating_mul(5);
    score = score.saturating_add(if board.red_to_move { mobility_bonus } else { -mobility_bonus });

    // King safety evaluation
    if let Some(red_king_pos) = find_king(board, Color::Red) {
        score = score.saturating_add(evaluate_king_safety(board, red_king_pos, Color::Red));
    }
    if let Some(black_king_pos) = find_king(board, Color::Black) {
        score = score.saturating_sub(evaluate_king_safety(board, black_king_pos, Color::Black));
    }

    // Negate score for black's turn
    if !board.red_to_move {
        -score
    } else {
        score
    }
}

fn find_king(board: &Board, color: Color) -> Option<(usize, usize)> {
    for rank in 0..10 {
        for file in 0..9 {
            if let Some((piece_color, Piece::General)) = board.squares[rank][file].piece {
                if piece_color == color {
                    return Some((rank, file));
                }
            }
        }
    }
    None
}

fn evaluate_king_safety(board: &Board, king_pos: (usize, usize), color: Color) -> i32 {
    let mut safety_score = 0;
    
    // Bonus for having advisors and elephants near the king
    let rank_range = if color == Color::Red { 7..10 } else { 0..3 };
    let mut protector_count = 0;
    
    for rank in rank_range {
        for file in 3..6 {
            if let Some((piece_color, piece)) = board.squares[rank][file].piece {
                if piece_color == color {
                    match piece {
                        Piece::Advisor | Piece::Elephant => protector_count += 1,
                        _ => {}
                    }
                }
            }
        }
    }
    
    safety_score += protector_count * 15;
    
    // Penalty for exposed king
    let (rank, _) = king_pos;
    let exposed_penalty = if color == Color::Red {
        if rank <= 7 {
            0
        } else {
            ((rank - 7) as i32) * 10
        }
    } else {
        if rank >= 2 {
            ((rank - 2) as i32) * 10
        } else {
            0
        }
    };
    
    safety_score - exposed_penalty
}

fn find_general_files(board: &Board) -> (Option<usize>, Option<usize>) {
    let mut red_general_file = None;
    let mut black_general_file = None;

    for rank in 0..10 {
        for file in 0..9 {
            if let Some((color, Piece::General)) = board.squares[rank][file].piece {
                if color == Color::Red {
                    red_general_file = Some(file);
                } else {
                    black_general_file = Some(file);
                }
            }
        }
    }

    (red_general_file, black_general_file)
}
