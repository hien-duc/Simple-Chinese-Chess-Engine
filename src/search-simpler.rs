use crate::board::Board;
use crate::evaluation::evaluate_position;
use crate::moves::{generate_legal_moves, Move};
use std::time::{Duration, Instant};

const MAX_DEPTH: i32 = 4;
const INFINITY: i32 = 50000;
const MATE_SCORE: i32 = 40000;

// implement negamax with alpha-beta pruning
// using the MVV_VLA algorithm

pub struct SearchInfo {
    pub nodes: u64,
    pub start_time: Instant,
    pub max_time: Duration,
}

impl SearchInfo {
    pub fn new(max_time_ms: u64) -> Self {
        SearchInfo {
            nodes: 0,
            start_time: Instant::now(),
            max_time: Duration::from_millis(max_time_ms),
        }
    }

    pub fn should_stop(&self) -> bool {
        self.start_time.elapsed() >= self.max_time
    }
}

pub fn find_best_move(board: &Board) -> Option<Move> {
    let mut search_info = SearchInfo::new(5000); // 5 seconds max
    iterative_deepening(board, &mut search_info)
}

fn iterative_deepening(board: &Board, info: &mut SearchInfo) -> Option<Move> {
    let mut best_move = None;
    let mut depth = 1;

    while depth <= MAX_DEPTH && !info.should_stop() {
        let (_, current_move) = negamax_root(board, depth, info);
        if info.should_stop() {
            break;
        }
        best_move = current_move;
        depth += 1;
    }

    best_move
}

fn negamax_root(board: &Board, depth: i32, info: &mut SearchInfo) -> (i32, Option<Move>) {
    let mut best_move = None;
    let mut best_score = -INFINITY;

    let legal_moves = generate_legal_moves(board);
    if legal_moves.is_empty() {
        return (-MATE_SCORE, None);
    }

    for mv in legal_moves {
        let mut new_board = board.clone();
        if !new_board.make_move(mv.from, mv.to) {
            continue;
        }

        let score = -negamax(&new_board, depth - 1, -INFINITY, -best_score, info);

        if score > best_score {
            best_score = score;
            best_move = Some(mv.clone());
        }

        if info.should_stop() {
            break;
        }
    }

    (best_score, best_move)
}

fn negamax(board: &Board, depth: i32, alpha: i32, beta: i32, info: &mut SearchInfo) -> i32 {
    info.nodes += 1;

    if info.should_stop() {
        return 0;
    }

    let legal_moves = generate_legal_moves(board);
    if legal_moves.is_empty() {
        return -MATE_SCORE + (MAX_DEPTH - depth) as i32;
    }

    if depth == 0 {
        return evaluate_position(board);
    }

    let mut alpha = alpha;
    for mv in legal_moves {
        let mut new_board = board.clone();
        if !new_board.make_move(mv.from, mv.to) {
            continue;
        }

        let score = -negamax(&new_board, depth - 1, -beta, -alpha, info);

        if score >= beta {
            return beta;
        }
        alpha = alpha.max(score);

        if info.should_stop() {
            break;
        }
    }

    alpha
}
