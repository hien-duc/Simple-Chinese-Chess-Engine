use crate::board::{Board, Color, Piece};
use crate::evaluation::evaluate_position;
use crate::moves::{generate_legal_moves, Move};
use std::collections::HashMap;
use std::time::Instant;

const INFINITY: i32 = 50000;
const MATE_SCORE: i32 = 49000;
const MAX_DEPTH: i32 = 128; // Increased from 64 to allow deeper searches
const LMR_LIMIT: i32 = 3; // Minimum depth for LMR
const IID_DEPTH: i32 = 5; // Minimum depth for Internal Iterative Deepening
const HISTORY_PRUNING_THRESHOLD: i32 = -4000; // History score threshold for pruning
const LATE_MOVE_PRUNING_LIMIT: i32 = 8;  // Number of moves to search fully before pruning
const DELTA_PRUNING_MARGIN: i32 = 200;  // Margin for delta pruning in quiescence search
const FUTILITY_MARGIN: [i32; 4] = [0, 100, 200, 300]; // Margins for depths 0-3
const RAZOR_MARGIN: [i32; 4] = [0, 300, 500, 900]; // Razoring margins for depths 1-3
const SEE_PIECE_VALUES: [i32; 7] = [0, 100, 450, 450, 650, 900, 10000]; // Pawn to King values for SEE

// Piece values for MVV-LVA
const MVV_LVA_SCORES: [[i32; 7]; 7] = [
    [105, 205, 305, 405, 505, 605, 705],  // Victim Pawn
    [104, 204, 304, 404, 504, 604, 704],  // Victim Knight
    [103, 203, 303, 403, 503, 603, 703],  // Victim Bishop
    [102, 202, 302, 402, 502, 602, 702],  // Victim Rook
    [101, 201, 301, 401, 501, 601, 701],  // Victim Queen
    [100, 200, 300, 400, 500, 600, 700],  // Victim King
    [100, 200, 300, 400, 500, 600, 700],  // Victim None (for non-captures)
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
            killer_moves: vec![
                KillerMoves {
                    moves: [None, None]
                };
                MAX_DEPTH as usize
            ],
            tt: HashMap::new(),
        }
    }

    pub fn should_stop(&self) -> bool {
        self.start_time.elapsed().as_millis() as u64 >= self.time_limit
    }

    fn update_killer_move(&mut self, mv: &Move, ply: usize) {
        if self.killer_moves[ply].moves[0].as_ref() != Some(mv) {
            self.killer_moves[ply].moves[1] = self.killer_moves[ply].moves[0].clone();
            self.killer_moves[ply].moves[0] = Some(mv.clone());
        }
    }

    fn update_history_score(&mut self, mv: &Move, depth: i32) {
        let from_idx = mv.from.0.min(9) * 9 + mv.from.1.min(8);
        let to_idx = mv.to.0.min(9) * 9 + mv.to.1.min(8);
        self.history_table[from_idx][to_idx] += depth * depth;
    }

    fn get_history_score(&self, mv: &Move) -> i32 {
        let from_idx = mv.from.0.min(9) * 9 + mv.from.1.min(8);
        let to_idx = mv.to.0.min(9) * 9 + mv.to.1.min(8);
        self.history_table[from_idx][to_idx]
    }
}

pub fn find_best_move(board: &Board) -> Option<Move> {
    let mut info = SearchInfo::new(1000);
    iterative_deepening(board, &mut info)
}

fn iterative_deepening(board: &Board, info: &mut SearchInfo) -> Option<Move> {
    let mut best_move = None;
    let mut prev_depth_time = 0;
    let mut prev_score = 0;
    let mut window_size = 50;

    for depth in 1..=MAX_DEPTH {
        let depth_start = info.start_time.elapsed().as_millis() as u64;

        let (score, mv) = if depth > 4 {
            let mut alpha = prev_score - window_size;
            let mut beta = prev_score + window_size;
            let mut current_result = negamax_root(board, depth, alpha, beta, info);

            loop {
                if current_result.0 <= alpha {
                    window_size *= 2;
                    alpha = current_result.0 - window_size;
                    current_result = negamax_root(board, depth, alpha, beta, info);
                } else if current_result.0 >= beta {
                    window_size *= 2;
                    beta = current_result.0 + window_size;
                    current_result = negamax_root(board, depth, alpha, beta, info);
                } else {
                    window_size = 50;
                    break;
                }

                if info.should_stop() {
                    break;
                }
            }
            current_result
        } else {
            negamax_root(board, depth, -INFINITY, INFINITY, info)
        };

        if !info.should_stop() {
            best_move = mv;
            prev_score = score;
            let depth_time = info.start_time.elapsed().as_millis() as u64 - depth_start;
            let total_time = info.start_time.elapsed().as_millis() as u64;

            if score.abs() > MATE_SCORE - 1000 {
                break;
            }

            if depth > 4 {
                if depth_time > prev_depth_time * 2 && total_time > info.time_limit / 2 {
                    break;
                }
                if total_time > info.time_limit * 3 / 4 {
                    break;
                }
            }
            prev_depth_time = depth_time;
        }
    }

    best_move
}

fn negamax_root(
    board: &Board,
    depth: i32,
    alpha: i32,
    beta: i32,
    info: &mut SearchInfo,
) -> (i32, Option<Move>) {
    let mut best_move = None;
    let mut best_score = -INFINITY;
    let hash = compute_hash(board);

    if let Some(tt_entry) = info.tt.get(&hash) {
        if tt_entry.depth >= depth {
            if tt_entry.node_type == NodeType::Exact {
                return (tt_entry.score, tt_entry.best_move.clone());
            }
        }
    }

    let mut moves = generate_legal_moves(board);
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
        }

        if info.should_stop() {
            break;
        }
    }

    info.tt.insert(
        hash,
        TTEntry {
            depth,
            score: best_score,
            node_type: NodeType::Exact,
            best_move: best_move.clone(),
        },
    );

    (best_score, best_move)
}

fn negamax(
    board: &Board,
    mut depth: i32,
    mut alpha: i32,
    mut beta: i32,
    info: &mut SearchInfo,
    ply: usize,
) -> i32 {
    info.nodes += 1;

    if info.should_stop() {
        return 0;
    }

    let hash = compute_hash(board);
    let mut tt_move = None;
    if let Some(tt_entry) = info.tt.get(&hash) {
        tt_move = tt_entry.best_move.clone();
        if tt_entry.depth >= depth {
            match tt_entry.node_type {
                NodeType::Exact => return tt_entry.score,
                NodeType::LowerBound => alpha = alpha.max(tt_entry.score),
                NodeType::UpperBound => beta = beta.min(tt_entry.score),
            }
            if alpha >= beta {
                return tt_entry.score;
            }
        }
    }

    let is_in_check = board.is_in_check(if board.red_to_move {
        Color::Red
    } else {
        Color::Black
    });

    if is_in_check {
        depth += 1;
    }

    if depth <= 0 {
        return quiescence_search(board, alpha, beta, info);
    }

    if !is_in_check && depth <= 3 {
        let eval = evaluate_position(board);
        let razor_margin = RAZOR_MARGIN[depth as usize];

        if eval + razor_margin <= alpha {
            let q_score = quiescence_search(board, alpha - razor_margin, alpha - razor_margin + 1, info);
            if q_score + razor_margin <= alpha {
                return q_score;
            }
        }
    }

    let mut moves = generate_legal_moves(board);
    if moves.is_empty() {
        if is_in_check {
            return -MATE_SCORE + ply as i32;
        }
        return 0;
    }

    if depth >= IID_DEPTH && tt_move.is_none() {
        let iid_depth = depth - 2;
        negamax(board, iid_depth, alpha, beta, info, ply);
        if let Some(tt_entry) = info.tt.get(&hash) {
            tt_move = tt_entry.best_move.clone();
        }
    }

    sort_moves(board, &mut moves, info, ply, tt_move.as_ref());

    let mut best_score = -INFINITY;
    let mut node_type = NodeType::UpperBound;
    let mut best_move = None;
    let mut moves_searched = 0;
    let static_eval = evaluate_position(board);

    for mv in &moves {
        let mut new_board = board.clone();
        if !new_board.make_move(mv.from, mv.to) {
            continue;
        }

        let mut score;
        moves_searched += 1;

        if is_capture(board, mv) && moves_searched > 1 {
            let see_score = see(board, mv);
            if see_score < -50 {
                continue;
            }
        }

        if depth <= 3 && !is_in_check && moves_searched > 1 && !is_capture(board, mv) {
            let margin = FUTILITY_MARGIN[depth as usize];
            if static_eval + margin <= alpha {
                continue;
            }
        }

        if depth >= LMR_LIMIT && moves_searched > 3 && !is_in_check && !is_capture(board, mv) {
            let history_score = info.get_history_score(mv);

            if history_score < HISTORY_PRUNING_THRESHOLD && depth <= 3 {
                continue;
            }

            let reduction = if history_score < 0 { 2 } else { 1 };
            score = -negamax(&new_board, depth - 1 - reduction, -beta, -alpha, info, ply + 1);

            if score > alpha {
                score = -negamax(&new_board, depth - 1, -beta, -alpha, info, ply + 1);
            }
        } else {
            score = -negamax(&new_board, depth - 1, -beta, -alpha, info, ply + 1);
        }

        if score > best_score {
            best_score = score;
            best_move = Some(mv.clone());

            if score > alpha {
                node_type = NodeType::Exact;
                alpha = score;

                if !is_capture(board, mv) {
                    info.update_killer_move(mv, ply);
                    info.update_history_score(mv, depth);
                }
            }
        }

        if alpha >= beta {
            node_type = NodeType::LowerBound;
            if !is_capture(board, mv) {
                info.update_killer_move(mv, ply);
                info.update_history_score(mv, depth * 2);
            }
            break;
        }

        if moves_searched > LATE_MOVE_PRUNING_LIMIT {
            if score <= alpha - DELTA_PRUNING_MARGIN {
                break;
            }
        }
    }

    info.tt.insert(
        hash,
        TTEntry {
            depth,
            score: best_score,
            node_type,
            best_move,
        },
    );

    best_score
}

fn quiescence_search(board: &Board, mut alpha: i32, beta: i32, info: &mut SearchInfo) -> i32 {
    info.nodes += 1;

    if info.should_stop() {
        return 0;
    }

    let stand_pat = evaluate_position(board);
    
    if stand_pat >= beta {
        return beta;
    }

    // Delta pruning
    if stand_pat < alpha - DELTA_PRUNING_MARGIN {
        return alpha;
    }

    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let mut moves = generate_legal_moves(board);
    sort_moves(board, &mut moves, info, 0, None);

    // Only search captures
    moves.retain(|mv| is_capture(board, mv));

    for mv in moves {
        let mut new_board = board.clone();
        if !new_board.make_move(mv.from, mv.to) {
            continue;
        }

        let score = -quiescence_search(&new_board, -beta, -alpha, info);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn is_capture(board: &Board, mv: &Move) -> bool {
    board.squares[mv.to.0][mv.to.1].piece.is_some()
}

fn sort_moves(
    board: &Board,
    moves: &mut Vec<Move>,
    info: &SearchInfo,
    ply: usize,
    tt_move: Option<&Move>,
) {
    // Score struct to help with move ordering
    #[derive(Clone)]
    struct MoveScore {
        mv: Move,
        score: i32,
    }

    let mut move_scores: Vec<MoveScore> = moves
        .iter()
        .map(|mv| {
            let mut score = 0;
            
            // TT move gets highest priority
            if let Some(ttm) = tt_move {
                if ttm == mv {
                    score += 20000;
                }
            }
            
            // Captures scored by MVV/LVA and SEE
            if let Some((_, victim_piece)) = board.squares[mv.to.0][mv.to.1].piece {
                if let Some((_, attacker_piece)) = board.squares[mv.from.0][mv.from.1].piece {
                    score += MVV_LVA_SCORES[get_piece_value_for_see(&victim_piece)]
                        [get_piece_value_for_see(&attacker_piece)];
                    
                    // Add SEE score for captures
                    score += see(board, mv);
                }
            }
            
            // Killer moves
            if let Some(killer1) = &info.killer_moves[ply].moves[0] {
                if killer1 == mv {
                    score += 9000;
                }
            }
            if let Some(killer2) = &info.killer_moves[ply].moves[1] {
                if killer2 == mv {
                    score += 8000;
                }
            }
            
            // History heuristic
            score += info.get_history_score(mv);
            
            MoveScore { mv: mv.clone(), score }
        })
        .collect();

    // Sort moves by score
    move_scores.sort_by_key(|ms| -ms.score);
    
    // Update moves vector with sorted moves
    *moves = move_scores.into_iter().map(|ms| ms.mv).collect();
}

fn see(board: &Board, mv: &Move) -> i32 {
    let mut gain = [0; 32];
    let mut depth = 0;

    if let Some((_, target_piece)) = board.squares[mv.to.0][mv.to.1].piece {
        gain[depth] = SEE_PIECE_VALUES[get_piece_value_for_see(&target_piece)];

        if let Some((_, attacker_piece)) = board.squares[mv.from.0][mv.from.1].piece {
            let attacker_value = get_piece_value_for_see(&attacker_piece);
            let target_value = get_piece_value_for_see(&target_piece);

            if SEE_PIECE_VALUES[attacker_value] <= SEE_PIECE_VALUES[target_value] {
                return gain[depth];
            }

            gain[depth] -= SEE_PIECE_VALUES[attacker_value];
            depth += 1;

            if gain[0] < -500 {
                return gain[0];
            }

            gain[depth] = SEE_PIECE_VALUES[attacker_value];
            let score = -gain[depth - 1];
            return score;
        }
    }
    0
}

fn get_piece_value_for_see(piece: &Piece) -> usize {
    match piece {
        Piece::Soldier => 1,  // Pawn
        Piece::Horse => 2,    // Knight
        Piece::Elephant => 3, // Bishop
        Piece::Chariot => 4,  // Rook
        Piece::Cannon => 5,   // Cannon
        Piece::Advisor => 2,  // Advisor (similar to knight)
        Piece::General => 6,  // King
    }
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
