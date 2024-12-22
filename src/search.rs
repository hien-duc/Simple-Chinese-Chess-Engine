use std::collections::HashMap;
use std::time::Instant;
use crate::board::{Board, Color, Piece};
use crate::evaluation::evaluate_position;
use crate::moves::{generate_legal_moves, Move};

const INFINITY: i32 = 50000;
const MATE_SCORE: i32 = 49000;
const MAX_DEPTH: i32 = 64;
const MAX_PLY: usize = 64;

// Piece values for MVV-LVA
const MVV_LVA_SCORES: [[i32; 7]; 7] = [
    [0, 50, 50, 50, 50, 50, 0],    // victim General
    [0, 45, 45, 45, 45, 45, 0],    // victim Chariot
    [0, 40, 40, 40, 40, 40, 0],    // victim Cannon
    [0, 35, 35, 35, 35, 35, 0],    // victim Horse
    [0, 30, 30, 30, 30, 30, 0],    // victim Advisor
    [0, 25, 25, 25, 25, 25, 0],    // victim Elephant
    [0, 20, 20, 20, 20, 20, 0],    // victim Soldier
];

#[derive(Clone, Copy, PartialEq)]
enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone)]
struct TTEntry {
    depth: i32,
    score: i32,
    node_type: NodeType,
    best_move: Option<Move>,
}

#[derive(Clone)]
struct KillerMoves {
    moves: [Option<Move>; 2],
}

pub struct SearchInfo {
    pub nodes: u64,
    pub start_time: Instant,
    pub time_limit: u64,
    history_table: [[i32; 90]; 90],
    killer_moves: Vec<KillerMoves>,
    tt: HashMap<u64, TTEntry>,
}

impl SearchInfo {
    pub fn new(time_limit: u64) -> Self {
        SearchInfo {
            nodes: 0,
            start_time: Instant::now(),
            time_limit,
            history_table: [[0; 90]; 90],
            killer_moves: vec![KillerMoves { moves: [None, None] }; MAX_PLY],
            tt: HashMap::new(),
        }
    }

    pub fn should_stop(&self) -> bool {
        self.start_time.elapsed().as_millis() as u64 >= self.time_limit
    }

    fn update_killer_move(&mut self, mv: &Move, ply: usize) {
        if ply < MAX_PLY && self.killer_moves[ply].moves[0].as_ref() != Some(mv) {
            self.killer_moves[ply].moves[1] = self.killer_moves[ply].moves[0].clone();
            self.killer_moves[ply].moves[0] = Some(mv.clone());
        }
    }

    fn update_history_score(&mut self, mv: &Move, depth: i32) {
        let from_idx = mv.from.0 * 9 + mv.from.1;
        let to_idx = mv.to.0 * 9 + mv.to.1;
        self.history_table[from_idx][to_idx] += depth * depth;
    }
}

pub fn find_best_move(board: &Board) -> Option<Move> {
    let mut info = SearchInfo::new(1000);
    iterative_deepening(board, &mut info)
}

fn iterative_deepening(board: &Board, info: &mut SearchInfo) -> Option<Move> {
    let mut best_move = None;

    for depth in 1..=MAX_DEPTH {
        if info.should_stop() {
            break;
        }

        let (_, mv) = negamax_root(board, depth, info);
        if !info.should_stop() {
            best_move = mv;
        }
    }

    best_move
}

fn negamax_root(board: &Board, depth: i32, info: &mut SearchInfo) -> (i32, Option<Move>) {
    let mut best_move = None;
    let mut best_score = -INFINITY;
    let mut alpha = -INFINITY;
    let beta = INFINITY;
    let hash = compute_hash(board);
    
    if let Some(tt_entry) = info.tt.get(&hash) {
        if tt_entry.depth >= depth {
            if tt_entry.node_type == NodeType::Exact {
                let score = if board.red_to_move { tt_entry.score } else { -tt_entry.score };
                return (score, tt_entry.best_move.clone());
            }
        }
    }

    let mut moves = generate_legal_moves(board);
    if moves.is_empty() {
        return (-MATE_SCORE, None);
    }
    
    sort_moves(board, &mut moves, info, 0, None);

    for mv in moves {
        let mut new_board = board.clone();
        if !new_board.make_move(mv.from, mv.to) {
            continue;
        }
        
        let score = -negamax(&new_board, depth - 1, -beta, -alpha, info, 1);
        
        if score > best_score {
            best_score = score;
            best_move = Some(mv.clone());
            alpha = score;
        }

        if info.should_stop() {
            break;
        }
    }

    // Store score in TT from Red's perspective
    let tt_score = if board.red_to_move { best_score } else { -best_score };
    info.tt.insert(hash, TTEntry {
        depth,
        score: tt_score,
        node_type: NodeType::Exact,
        best_move: best_move.clone(),
    });

    // Return score from side-to-move perspective
    (best_score, best_move)
}

fn negamax(board: &Board, depth: i32, mut alpha: i32, mut beta: i32, info: &mut SearchInfo, ply: usize) -> i32 {
    // Early return if we've exceeded maximum ply
    if ply >= MAX_PLY {
        let eval = evaluate_position(board);
        return if board.red_to_move { eval } else { -eval };
    }

    info.nodes += 1;

    if info.should_stop() {
        return 0;
    }

    let hash = compute_hash(board);

    if let Some(tt_entry) = info.tt.get(&hash) {
        if tt_entry.depth >= depth {
            let score = if board.red_to_move { tt_entry.score } else { -tt_entry.score };
            match tt_entry.node_type {
                NodeType::Exact => return score,
                NodeType::LowerBound => alpha = alpha.max(score),
                NodeType::UpperBound => beta = beta.min(score),
            }
            if alpha >= beta {
                return score;
            }
        }
    }

    let is_in_check = board.is_in_check(if board.red_to_move { Color::Red } else { Color::Black });
    let depth = if is_in_check { depth + 1 } else { depth };

    if depth <= 0 {
        let eval = quiescence_search(board, alpha, beta, info);
        return if board.red_to_move { eval } else { -eval };
    }

    let mut moves = generate_legal_moves(board);
    if moves.is_empty() {
        if is_in_check {
            return -MATE_SCORE + ply as i32;  // Checkmate
        } else {
            return 0;  // Stalemate
        }
    }

    let tt_move = info.tt.get(&hash).and_then(|entry| entry.best_move.clone());
    sort_moves(board, &mut moves, info, ply, tt_move.as_ref());

    let mut best_score = -INFINITY;
    let mut best_move = None;
    let mut node_type = NodeType::UpperBound;

    for mv in moves {
        let mut new_board = board.clone();
        if !new_board.make_move(mv.from, mv.to) {
            continue;
        }

        let score = -negamax(&new_board, depth - 1, -beta, -alpha, info, ply + 1);

        if score > best_score {
            best_score = score;
            best_move = Some(mv.clone());
            alpha = alpha.max(score);
            if alpha >= beta {
                info.update_killer_move(&mv, ply);
                info.update_history_score(&mv, depth);
                node_type = NodeType::LowerBound;
                break;
            }
            node_type = NodeType::Exact;
        }
    }

    // Store score in TT from Red's perspective
    let tt_score = if board.red_to_move { best_score } else { -best_score };
    info.tt.insert(hash, TTEntry {
        depth,
        score: tt_score,
        node_type,
        best_move,
    });

    best_score
}

fn quiescence_search(board: &Board, mut alpha: i32, beta: i32, info: &mut SearchInfo) -> i32 {
    info.nodes += 1;

    let eval = evaluate_position(board);
    let stand_pat = if board.red_to_move { eval } else { -eval };
    
    if stand_pat >= beta {
        return beta;
    }
    
    alpha = alpha.max(stand_pat);

    let mut moves = generate_legal_moves(board);
    moves.retain(|mv| is_capture(board, mv));
    sort_moves(board, &mut moves, info, 0, None);

    for mv in moves {
        let mut new_board = board.clone();
        if !new_board.make_move(mv.from, mv.to) {
            continue;
        }

        let score = -quiescence_search(&new_board, -beta, -alpha, info);

        if score >= beta {
            return beta;
        }
        alpha = alpha.max(score);
    }

    alpha
}

fn sort_moves(board: &Board, moves: &mut Vec<Move>, info: &SearchInfo, ply: usize, tt_move: Option<&Move>) {
    let mut move_scores: Vec<(i32, usize)> = moves
        .iter()
        .enumerate()
        .map(|(i, mv)| {
            let mut score = 0;
            
            // Prioritize transposition table move
            if let Some(tt_mv) = tt_move {
                if mv == tt_mv {
                    score += 20000;
                }
            }

            // Score captures using MVV-LVA
            if let Some((_, victim_piece)) = board.squares[mv.to.0][mv.to.1].piece {
                if let Some((_, attacker_piece)) = board.squares[mv.from.0][mv.from.1].piece {
                    score += MVV_LVA_SCORES[get_piece_value(victim_piece)][get_piece_value(attacker_piece)];
                }
            }

            // Score killer moves
            if ply < MAX_PLY {
                if let Some(killer1) = &info.killer_moves[ply].moves[0] {
                    if mv == killer1 {
                        score += 10000;
                    }
                }
                if let Some(killer2) = &info.killer_moves[ply].moves[1] {
                    if mv == killer2 {
                        score += 9000;
                    }
                }
            }

            // Score history moves
            let from_idx = mv.from.0 * 9 + mv.from.1;
            let to_idx = mv.to.0 * 9 + mv.to.1;
            score += info.history_table[from_idx][to_idx];

            (score, i)
        })
        .collect();

    move_scores.sort_by_key(|(score, _)| -score);
    
    let mut sorted_moves = vec![Move::new((0, 0), (0, 0)); moves.len()];
    for (i, (_, original_index)) in move_scores.iter().enumerate() {
        sorted_moves[i] = moves[*original_index].clone();
    }
    moves.clone_from(&sorted_moves);
}

fn is_capture(board: &Board, mv: &Move) -> bool {
    board.squares[mv.to.0][mv.to.1].piece.is_some()
}

fn compute_hash(board: &Board) -> u64 {
    let zobrist = get_zobrist();
    let mut hash = 0;

    for rank in 0..10 {
        for file in 0..9 {
            if let Some((color, piece)) = board.squares[rank][file].piece {
                let color_idx = if color == Color::Red { 0 } else { 1 };
                let piece_idx = get_piece_value(piece);
                let square_idx = rank * 9 + file;
                hash ^= zobrist.piece_square[color_idx][piece_idx][square_idx];
            }
        }
    }

    if board.red_to_move {
        hash ^= zobrist.side_to_move;
    }

    hash
}

fn get_piece_value(piece: Piece) -> usize {
    match piece {
        Piece::General => 0,
        Piece::Chariot => 1,
        Piece::Cannon => 2,
        Piece::Horse => 3,
        Piece::Advisor => 4,
        Piece::Elephant => 5,
        Piece::Soldier => 6,
    }
}

struct Zobrist {
    piece_square: [[[u64; 90]; 7]; 2], // [color][piece_type][square]
    side_to_move: u64,
}

impl Zobrist {
    fn new() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut z = Zobrist {
            piece_square: [[[0; 90]; 7]; 2],
            side_to_move: rng.gen(),
        };
        
        for color in 0..2 {
            for piece in 0..7 {
                for square in 0..90 {
                    z.piece_square[color][piece][square] = rng.gen();
                }
            }
        }
        z
    }
}

static ZOBRIST: std::sync::OnceLock<Zobrist> = std::sync::OnceLock::new();

fn get_zobrist() -> &'static Zobrist {
    ZOBRIST.get_or_init(Zobrist::new)
}