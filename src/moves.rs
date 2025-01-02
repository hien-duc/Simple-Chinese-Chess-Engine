use std::fmt;
use crate::board::{Board, Color, Piece};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

impl Move {
    pub fn new(from: (usize, usize), to: (usize, usize)) -> Self {
        Move { from, to }
    }

    pub fn to_uci(&self) -> String {
        // convert internal coordinates to UCI format
        // for black's perspective, we need to flip the coordinates
        format!(
            "{}{}{}{}",
            (b'a' + self.from.1 as u8) as char,
            (b'9' - self.from.0 as u8) as char,
            (b'a' + self.to.1 as u8) as char,
            (b'9' - self.to.0 as u8) as char,
        )
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

pub fn generate_legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    let color = if board.red_to_move { Color::Red } else { Color::Black };

    // generate moves based on the current side to move
    for rank in 0..10 {
        for file in 0..9 {
            if let Some((piece_color, _)) = board.squares[rank][file].piece {
                if piece_color == color {
                    let mut piece_moves = generate_piece_moves(board, (rank, file));
                    moves.append(&mut piece_moves);
                }
            }
        }
    }

    moves
}

fn generate_piece_moves(board: &Board, pos: (usize, usize)) -> Vec<Move> {
    let mut moves = Vec::new();
    if let Some((color, piece_type)) = board.squares[pos.0][pos.1].piece {
        match piece_type {
            Piece::General => generate_general_moves(board, pos, color, &mut moves),
            Piece::Advisor => generate_advisor_moves(board, pos, color, &mut moves),
            Piece::Elephant => generate_elephant_moves(board, pos, color, &mut moves),
            Piece::Horse => generate_horse_moves(board, pos, color, &mut moves),
            Piece::Chariot => generate_chariot_moves(board, pos, color, &mut moves),
            Piece::Cannon => generate_cannon_moves(board, pos, color, &mut moves),
            Piece::Soldier => generate_soldier_moves(board, pos, color, &mut moves),
        }
    }
    moves
}

fn generate_chariot_moves(board: &Board, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    for &(dx, dy) in &directions {
        let mut x = pos.0 as i32;
        let mut y = pos.1 as i32;
        loop {
            x += dx;
            y += dy;
            if x < 0 || x >= 10 || y < 0 || y >= 9 {
                break;
            }
            let new_pos = (x as usize, y as usize);
            match board.squares[new_pos.0][new_pos.1].piece {
                None => moves.push(Move::new(pos, new_pos)),
                Some((piece_color, _)) => {
                    if piece_color != color {
                        moves.push(Move::new(pos, new_pos));
                    }
                    break;
                }
            }
        }
    }
}

fn generate_horse_moves(board: &Board, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
    let (rank, file) = pos;
    let possible_moves = [
        // only add moves if the starting position allows them
        (if rank >= 2 && file <= 7 { Some((rank - 2, file + 1)) } else { None }),
        (if rank >= 2 && file >= 1 { Some((rank - 2, file - 1)) } else { None }),
        (if rank + 2 <= 9 && file <= 7 { Some((rank + 2, file + 1)) } else { None }),
        (if rank + 2 <= 9 && file >= 1 { Some((rank + 2, file - 1)) } else { None }),
        (if rank >= 1 && file <= 6 { Some((rank - 1, file + 2)) } else { None }),
        (if rank >= 1 && file >= 2 { Some((rank - 1, file - 2)) } else { None }),
        (if rank + 1 <= 9 && file <= 6 { Some((rank + 1, file + 2)) } else { None }),
        (if rank + 1 <= 9 && file >= 2 { Some((rank + 1, file - 2)) } else { None }),
    ];

    for possible_move in possible_moves.iter().flatten() {
        let (new_rank, new_file) = *possible_move;
        if !is_horse_blocked(board, pos, (new_rank, new_file)) {
            if let Some((piece_color, _)) = board.squares[new_rank][new_file].piece {
                if piece_color != color {
                    moves.push(Move::new(pos, (new_rank, new_file)));
                }
            } else {
                moves.push(Move::new(pos, (new_rank, new_file)));
            }
        }
    }
}

fn is_horse_blocked(board: &Board, from: (usize, usize), to: (usize, usize)) -> bool {
    let blocking_pos = if to.0 > from.0 {
        // move down
        if to.1 > from.1 {
            // move right
            if to.0 - from.0 == 2 {
                (from.0 + 1, from.1) // Blocked vertically
            } else {
                (from.0, from.1 + 1) // Blocked horizontally
            }
        } else {
            // move left
            if to.0 - from.0 == 2 {
                (from.0 + 1, from.1) // Blocked vertically
            } else {
                (from.0, from.1 - 1) // Blocked horizontally
            }
        }
    } else {
        // move up
        if to.1 > from.1 {
            // move right
            if from.0 - to.0 == 2 {
                (from.0 - 1, from.1) // Blocked vertically
            } else {
                (from.0, from.1 + 1) // Blocked horizontally
            }
        } else {
            // move left
            if from.0 - to.0 == 2 {
                (from.0 - 1, from.1) // Blocked vertically
            } else {
                (from.0, from.1 - 1) // Blocked horizontally
            }
        }
    };
    
    board.squares[blocking_pos.0][blocking_pos.1].piece.is_some()
}

fn generate_cannon_moves(board: &Board, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    for &(dx, dy) in &directions {
        let mut x = pos.0 as i32;
        let mut y = pos.1 as i32;
        let mut platform_found = false;
        
        loop {
            x += dx;
            y += dy;
            if x < 0 || x >= 10 || y < 0 || y >= 9 {
                break;
            }
            let new_pos = (x as usize, y as usize);
            
            if !platform_found {
                if board.squares[new_pos.0][new_pos.1].piece.is_none() {
                    moves.push(Move::new(pos, new_pos));
                } else {
                    platform_found = true;
                }
            } else {
                if let Some((piece_color, _)) = board.squares[new_pos.0][new_pos.1].piece {
                    if piece_color != color {
                        moves.push(Move::new(pos, new_pos));
                    }
                    break;
                }
            }
        }
    }
}

fn generate_general_moves(board: &Board, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
    let (rank, file) = pos;
    let palace_moves = match color {
        Color::Red => [(7, 3), (7, 4), (7, 5), (8, 3), (8, 4), (8, 5), (9, 3), (9, 4), (9, 5)],
        Color::Black => [(0, 3), (0, 4), (0, 5), (1, 3), (1, 4), (1, 5), (2, 3), (2, 4), (2, 5)],
    };

    for &(new_rank, new_file) in &palace_moves {
        if (new_rank as i32 - rank as i32).abs() + (new_file as i32 - file as i32).abs() == 1 {
            if let Some((piece_color, _)) = board.squares[new_rank][new_file].piece {
                if piece_color != color {
                    moves.push(Move::new(pos, (new_rank, new_file)));
                }
            } else {
                moves.push(Move::new(pos, (new_rank, new_file)));
            }
        }
    }
}

fn generate_advisor_moves(board: &Board, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
    let (rank, file) = pos;
    let palace_moves = match color {
        Color::Red => [(7, 3), (7, 5), (8, 4), (9, 3), (9, 5)],
        Color::Black => [(0, 3), (0, 5), (1, 4), (2, 3), (2, 5)],
    };

    for &(new_rank, new_file) in &palace_moves {
        if (new_rank as i32 - rank as i32).abs() == 1 && (new_file as i32 - file as i32).abs() == 1 {
            if let Some((piece_color, _)) = board.squares[new_rank][new_file].piece {
                if piece_color != color {
                    moves.push(Move::new(pos, (new_rank, new_file)));
                }
            } else {
                moves.push(Move::new(pos, (new_rank, new_file)));
            }
        }
    }
}

fn generate_elephant_moves(board: &Board, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
    let (rank, file) = pos;
    
    // check each possible diagonal move if it's within bounds
    // forward-right diagonal
    if rank + 2 <= 9 && file + 2 <= 8 {
        add_elephant_move(board, pos, (rank + 2, file + 2), color, moves);
    }
    
    // forward-left diagonal
    if rank + 2 <= 9 && file >= 2 {
        add_elephant_move(board, pos, (rank + 2, file - 2), color, moves);
    }
    
    // backward-right diagonal
    if rank >= 2 && file + 2 <= 8 {
        add_elephant_move(board, pos, (rank - 2, file + 2), color, moves);
    }
    
    // backward-left diagonal
    if rank >= 2 && file >= 2 {
        add_elephant_move(board, pos, (rank - 2, file - 2), color, moves);
    }
}

fn add_elephant_move(board: &Board, pos: (usize, usize), new_pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
    let (rank, file) = pos;
    let (new_rank, new_file) = new_pos;
    
    // check if move stays on correct side of river
    if (color == Color::Red && new_rank >= 5) || (color == Color::Black && new_rank <= 4) {
        // check if elephant's eye is blocked
        let eye_rank = (rank + new_rank) / 2;
        let eye_file = (file + new_file) / 2;
        if board.squares[eye_rank][eye_file].piece.is_none() {
            if let Some((piece_color, _)) = board.squares[new_rank][new_file].piece {
                if piece_color != color {
                    moves.push(Move::new(pos, (new_rank, new_file)));
                }
            } else {
                moves.push(Move::new(pos, (new_rank, new_file)));
            }
        }
    }
}

fn generate_soldier_moves(board: &Board, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
    let (rank, file) = pos;
    let mut possible_moves = Vec::new();

    match color {
        Color::Red => {
            if rank > 0 { possible_moves.push((rank - 1, file)); }
            if rank < 5 {
                if file > 0 { possible_moves.push((rank, file - 1)); }
                if file < 8 { possible_moves.push((rank, file + 1)); }
            }
        }
        Color::Black => {
            if rank < 9 { possible_moves.push((rank + 1, file)); }
            if rank > 4 {
                if file > 0 { possible_moves.push((rank, file - 1)); }
                if file < 8 { possible_moves.push((rank, file + 1)); }
            }
        }
    }

    for &(new_rank, new_file) in &possible_moves {
        if let Some((piece_color, _)) = board.squares[new_rank][new_file].piece {
            if piece_color != color {
                moves.push(Move::new(pos, (new_rank, new_file)));
            }
        } else {
            moves.push(Move::new(pos, (new_rank, new_file)));
        }
    }
}