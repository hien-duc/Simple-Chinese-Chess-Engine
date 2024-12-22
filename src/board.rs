use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Piece {
    Soldier,  // 兵/卒
    Horse,    // 马/馬
    Elephant, // 相/象
    Chariot,  // 车/車
    Cannon,   // 炮/砲
    Advisor,  // 士/仕
    General,  // 帅/將
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    Red,
    Black,
}

#[derive(Copy, Clone)]
pub struct Square {
    pub piece: Option<(Color, Piece)>,
}

#[derive(Clone)]
pub struct Board {
    pub squares: [[Square; 9]; 10],
    pub red_to_move: bool,
    #[allow(dead_code)]
    pub halfmove_clock: u16,
    #[allow(dead_code)]
    pub fullmove_number: u16,
}

impl Board {
    pub fn new() -> Self {
        Board {
            squares: [[Square { piece: None }; 9]; 10],
            red_to_move: true,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = Board::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() != 6 {
            return Err("Invalid FEN string: must have 6 parts".to_string());
        }

        // parse board position
        let ranks: Vec<&str> = parts[0].split('/').collect();
        if ranks.len() != 10 {
            return Err("Invalid FEN string: must have 10 ranks".to_string());
        }

        for (rank_idx, rank) in ranks.iter().enumerate() {
            let mut file_idx = 0;
            for c in rank.chars() {
                if file_idx >= 9 {
                    return Err(format!(
                        "Invalid FEN string: rank {} is too long",
                        rank_idx + 1
                    ));
                }
                if c.is_digit(10) {
                    let empty_squares = c.to_digit(10).unwrap() as usize;
                    file_idx += empty_squares;
                } else {
                    let (color, piece) = match c {
                        'K' => (Color::Red, Piece::General),
                        'k' => (Color::Black, Piece::General),
                        'A' => (Color::Red, Piece::Advisor),
                        'a' => (Color::Black, Piece::Advisor),
                        'E' | 'B' => (Color::Red, Piece::Elephant),
                        'e' | 'b' => (Color::Black, Piece::Elephant),
                        'H' | 'N' => (Color::Red, Piece::Horse),
                        'h' | 'n' => (Color::Black, Piece::Horse),
                        'R' => (Color::Red, Piece::Chariot),
                        'r' => (Color::Black, Piece::Chariot),
                        'C' => (Color::Red, Piece::Cannon),
                        'c' => (Color::Black, Piece::Cannon),
                        'P' => (Color::Red, Piece::Soldier),
                        'p' => (Color::Black, Piece::Soldier),
                        _ => return Err(format!("Invalid piece character in FEN: {}", c)),
                    };
                    board.squares[rank_idx][file_idx].piece = Some((color, piece));
                    file_idx += 1;
                }
            }
            if file_idx != 9 {
                return Err(format!(
                    "Invalid FEN string: rank {} is incomplete",
                    rank_idx + 1
                ));
            }
        }

        // parse active color
        board.red_to_move = match parts[1] {
            "r" | "w" => true,
            "b" => false,
            _ => return Err(format!("Invalid active color in FEN: {}", parts[1])),
        };

        println!("Active color from FEN: {}", if board.red_to_move { "Red" } else { "Black" });

        // parse halfmove clock
        if let Ok(halfmove) = parts[4].parse() {
            board.halfmove_clock = halfmove;
        } else {
            return Err("Invalid halfmove clock in FEN".to_string());
        }

        // parse fullmove number
        if let Ok(fullmove) = parts[5].parse() {
            board.fullmove_number = fullmove;
        } else {
            return Err("Invalid fullmove number in FEN".to_string());
        }

        Ok(board)
    }

    pub fn setup_initial_position(&mut self) {
        // clear the board
        self.squares = [[Square { piece: None }; 9]; 10];
        
        // set up red pieces (bottom side)
        // back rank (rank 0)
        self.squares[0][0].piece = Some((Color::Red, Piece::Chariot));
        self.squares[0][1].piece = Some((Color::Red, Piece::Horse));
        self.squares[0][2].piece = Some((Color::Red, Piece::Elephant));
        self.squares[0][3].piece = Some((Color::Red, Piece::Advisor));
        self.squares[0][4].piece = Some((Color::Red, Piece::General));
        self.squares[0][5].piece = Some((Color::Red, Piece::Advisor));
        self.squares[0][6].piece = Some((Color::Red, Piece::Elephant));
        self.squares[0][7].piece = Some((Color::Red, Piece::Horse));
        self.squares[0][8].piece = Some((Color::Red, Piece::Chariot));
        
        // cannons (rank 2)
        self.squares[2][1].piece = Some((Color::Red, Piece::Cannon));
        self.squares[2][7].piece = Some((Color::Red, Piece::Cannon));
        
        // soldiers (rank 3)
        self.squares[3][0].piece = Some((Color::Red, Piece::Soldier));
        self.squares[3][2].piece = Some((Color::Red, Piece::Soldier));
        self.squares[3][4].piece = Some((Color::Red, Piece::Soldier));
        self.squares[3][6].piece = Some((Color::Red, Piece::Soldier));
        self.squares[3][8].piece = Some((Color::Red, Piece::Soldier));

        // set up black pieces (top side)
        // back rank (rank 9)
        self.squares[9][0].piece = Some((Color::Black, Piece::Chariot));
        self.squares[9][1].piece = Some((Color::Black, Piece::Horse));
        self.squares[9][2].piece = Some((Color::Black, Piece::Elephant));
        self.squares[9][3].piece = Some((Color::Black, Piece::Advisor));
        self.squares[9][4].piece = Some((Color::Black, Piece::General));
        self.squares[9][5].piece = Some((Color::Black, Piece::Advisor));
        self.squares[9][6].piece = Some((Color::Black, Piece::Elephant));
        self.squares[9][7].piece = Some((Color::Black, Piece::Horse));
        self.squares[9][8].piece = Some((Color::Black, Piece::Chariot));
        
        // cannons (rank 7)
        self.squares[7][1].piece = Some((Color::Black, Piece::Cannon));
        self.squares[7][7].piece = Some((Color::Black, Piece::Cannon));
        
        // soldiers (rank 6)
        self.squares[6][0].piece = Some((Color::Black, Piece::Soldier));
        self.squares[6][2].piece = Some((Color::Black, Piece::Soldier));
        self.squares[6][4].piece = Some((Color::Black, Piece::Soldier));
        self.squares[6][6].piece = Some((Color::Black, Piece::Soldier));
        self.squares[6][8].piece = Some((Color::Black, Piece::Soldier));

        // red moves first
        self.red_to_move = true;
        self.halfmove_clock = 0;
        self.fullmove_number = 1;
    }

    pub fn make_move(&mut self, from: (usize, usize), to: (usize, usize)) -> bool {
        // Validate move coordinates
        if from.0 >= 10 || from.1 >= 9 || to.0 >= 10 || to.1 >= 9 {
            return false;
        }

        // Check if there is a piece at the source square
        if let Some((color, _)) = self.squares[from.0][from.1].piece {
            // Check if it's the correct side's turn
            if (color == Color::Red) != self.red_to_move {
                return false;
            }

            // Check if destination has a piece of the same color
            if let Some((dest_color, _)) = self.squares[to.0][to.1].piece {
                if color == dest_color {
                    return false;
                }
            }

            // Make the move
            self.squares[to.0][to.1].piece = self.squares[from.0][from.1].piece;
            self.squares[from.0][from.1].piece = None;

            // Switch turns
            self.red_to_move = !self.red_to_move;
            
            true
        } else {
            false
        }
    }

    // Check if a side is in check
    pub fn is_in_check(&self, color: Color) -> bool {
        // Find the general's position
        let mut general_pos = None;
        for rank in 0..10 {
            for file in 0..9 {
                if let Some((piece_color, Piece::General)) = self.squares[rank][file].piece {
                    if piece_color == color {
                        general_pos = Some((rank, file));
                        break;
                    }
                }
            }
        }

        if let Some(general_pos) = general_pos {
            // Check if any opponent's piece can capture the general
            let opponent_color = if color == Color::Red {
                Color::Black
            } else {
                Color::Red
            };
            for rank in 0..10 {
                for file in 0..9 {
                    if let Some((piece_color, _)) = self.squares[rank][file].piece {
                        if piece_color == opponent_color {
                            let moves = generate_piece_moves(self, (rank, file));
                            for mv in moves {
                                if mv == general_pos {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "   ┌────────────────────────────┐")?;

        for rank in (0..10).rev() {  
            write!(f, " {} │", rank)?;

            for file in 0..9 {
                let piece_str = match self.squares[rank][file].piece {
                    Some((Color::Red, piece)) => match piece {
                        Piece::General => " 帅",
                        Piece::Advisor => " 仕",
                        Piece::Elephant => " 相",
                        Piece::Horse => " 马",
                        Piece::Chariot => " 车",
                        Piece::Cannon => " 炮",
                        Piece::Soldier => " 兵",
                    },
                    Some((Color::Black, piece)) => match piece {
                        Piece::General => " 將",
                        Piece::Advisor => " 士",
                        Piece::Elephant => " 象",
                        Piece::Horse => " 馬",
                        Piece::Chariot => " 車",
                        Piece::Cannon => " 砲",
                        Piece::Soldier => " 卒",
                    },
                    None => "  ·",
                };

                write!(f, "{}", piece_str)?;
            }

            writeln!(f, " │")?;

            if rank == 5 {  
                writeln!(f, "   ├─────────楚 河 汉 界────────┤")?;
            }
        }

        writeln!(f, "   └────────────────────────────┘")?;
        writeln!(f, "      a  b  c  d  e  f  g  h  i")?;

        // show turn
        if self.red_to_move {
            writeln!(f, "\nRed (下) to move")?;
        } else {
            writeln!(f, "\nBlack (上) to move")?;
        }

        Ok(())
    }
}

// Generate moves for a piece
fn generate_piece_moves(board: &Board, pos: (usize, usize)) -> Vec<(usize, usize)> {
    let piece = board.squares[pos.0][pos.1].piece.unwrap();
    let (color, piece_type) = piece;
    let mut moves = Vec::new();

    match piece_type {
        Piece::General => {
            // General can move one square in any direction (horizontally or vertically)
            for (dr, dc) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let new_rank = pos.0 as i32 + dr;
                let new_file = pos.1 as i32 + dc;
                if new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                    let new_pos = (new_rank as usize, new_file as usize);
                    if board.squares[new_pos.0][new_pos.1].piece.is_none()
                        || board.squares[new_pos.0][new_pos.1].piece.unwrap().0 != color
                    {
                        moves.push(new_pos);
                    }
                }
            }
        }
        Piece::Advisor => {
            // Advisor can move one square diagonally
            for (dr, dc) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let new_rank = pos.0 as i32 + dr;
                let new_file = pos.1 as i32 + dc;
                if new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                    let new_pos = (new_rank as usize, new_file as usize);
                    if board.squares[new_pos.0][new_pos.1].piece.is_none()
                        || board.squares[new_pos.0][new_pos.1].piece.unwrap().0 != color
                    {
                        moves.push(new_pos);
                    }
                }
            }
        }
        Piece::Elephant => {
            // Elephant can move two squares diagonally
            for (dr, dc) in [(2, 2), (2, -2), (-2, 2), (-2, -2)] {
                let new_rank = pos.0 as i32 + dr;
                let new_file = pos.1 as i32 + dc;
                if new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                    let new_pos = (new_rank as usize, new_file as usize);
                    if board.squares[new_pos.0][new_pos.1].piece.is_none()
                        || board.squares[new_pos.0][new_pos.1].piece.unwrap().0 != color
                    {
                        moves.push(new_pos);
                    }
                }
            }
        }
        Piece::Horse => {
            // Horse can move one square horizontally or vertically, then one square diagonally
            for (dr, dc) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let mid_rank = pos.0 as i32 + dr;
                let mid_file = pos.1 as i32 + dc;
                if mid_rank >= 0 && mid_rank < 10 && mid_file >= 0 && mid_file < 9 {
                    for (ddr, ddc) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                        let new_rank = mid_rank + ddr;
                        let new_file = mid_file + ddc;
                        if new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                            let new_pos = (new_rank as usize, new_file as usize);
                            if board.squares[new_pos.0][new_pos.1].piece.is_none()
                                || board.squares[new_pos.0][new_pos.1].piece.unwrap().0 != color
                            {
                                moves.push(new_pos);
                            }
                        }
                    }
                }
            }
        }
        Piece::Chariot => {
            // Chariot can move any number of squares horizontally or vertically
            for (dr, dc) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let mut new_rank = pos.0 as i32 + dr;
                let mut new_file = pos.1 as i32 + dc;
                while new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                    let new_pos = (new_rank as usize, new_file as usize);
                    if board.squares[new_pos.0][new_pos.1].piece.is_none()
                        || board.squares[new_pos.0][new_pos.1].piece.unwrap().0 != color
                    {
                        moves.push(new_pos);
                    }
                    if board.squares[new_pos.0][new_pos.1].piece.is_some() {
                        break;
                    }
                    new_rank += dr;
                    new_file += dc;
                }
            }
        }
        Piece::Cannon => {
            // Cannon can move any number of squares horizontally or vertically, but must jump over exactly one piece
            for (dr, dc) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let mut new_rank = pos.0 as i32 + dr;
                let mut new_file = pos.1 as i32 + dc;
                let mut jumped = false;
                while new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                    let new_pos = (new_rank as usize, new_file as usize);
                    if board.squares[new_pos.0][new_pos.1].piece.is_none() {
                        if jumped {
                            moves.push(new_pos);
                        }
                    } else if board.squares[new_pos.0][new_pos.1].piece.unwrap().0 != color {
                        if !jumped {
                            jumped = true;
                        } else {
                            moves.push(new_pos);
                            break;
                        }
                    } else {
                        break;
                    }
                    new_rank += dr;
                    new_file += dc;
                }
            }
        }
        Piece::Soldier => {
            // Soldier can move one square forward, but captures diagonally
            if color == Color::Red {
                let new_rank = pos.0 as i32 + 1;
                let new_file = pos.1 as i32;
                if new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                    let new_pos = (new_rank as usize, new_file as usize);
                    if board.squares[new_pos.0][new_pos.1].piece.is_none() {
                        moves.push(new_pos);
                    }
                }
                for (dr, dc) in [(1, 1), (1, -1)] {
                    let new_rank = pos.0 as i32 + dr;
                    let new_file = pos.1 as i32 + dc;
                    if new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                        let new_pos = (new_rank as usize, new_file as usize);
                        if board.squares[new_pos.0][new_pos.1].piece.is_some()
                            && board.squares[new_pos.0][new_pos.1].piece.unwrap().0 != color
                        {
                            moves.push(new_pos);
                        }
                    }
                }
            } else {
                let new_rank = pos.0 as i32 - 1;
                let new_file = pos.1 as i32;
                if new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                    let new_pos = (new_rank as usize, new_file as usize);
                    if board.squares[new_pos.0][new_pos.1].piece.is_none() {
                        moves.push(new_pos);
                    }
                }
                for (dr, dc) in [(-1, 1), (-1, -1)] {
                    let new_rank = pos.0 as i32 + dr;
                    let new_file = pos.1 as i32 + dc;
                    if new_rank >= 0 && new_rank < 10 && new_file >= 0 && new_file < 9 {
                        let new_pos = (new_rank as usize, new_file as usize);
                        if board.squares[new_pos.0][new_pos.1].piece.is_some()
                            && board.squares[new_pos.0][new_pos.1].piece.unwrap().0 != color
                        {
                            moves.push(new_pos);
                        }
                    }
                }
            }
        }
    }

    moves
}
