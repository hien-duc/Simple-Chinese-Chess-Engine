use crate::board::{Board, Color, Piece};


const SOLDIER_VALUE: i32 = 30;
const CANNON_VALUE: i32 = 285;
const HORSE_VALUE: i32 = 270;
const ELEPHANT_VALUE: i32 = 120;
const ADVISOR_VALUE: i32 = 120;
const CHARIOT_VALUE: i32 = 600;

// tables for position evaluation
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

pub fn evaluate_position(board: &Board) -> i32 {
    let mut score = 0;

    // evaluate material and position
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
                    Piece::Cannon => CANNON_VALUE,
                    Piece::Horse => HORSE_VALUE,
                    Piece::Elephant => ELEPHANT_VALUE,
                    Piece::Advisor => ADVISOR_VALUE,
                    Piece::Chariot => CHARIOT_VALUE,
                    Piece::General => 6000,
                };

                // Score is always from Red's perspective
                if color == Color::Black {
                    piece_value = -piece_value;
                }

                score += piece_value;
            }
        }
    }

    // Always return score from Red's perspective
    score
}
