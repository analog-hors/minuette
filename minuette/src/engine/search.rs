use std::time::{Duration, Instant};

use arrayvec::ArrayVec;
use cozy_chess::{Board, Move, Color, Piece, GameStatus};

use super::board_stack::BoardStack;

type MoveList = ArrayVec<Move, 218>;

const CHECKMATE: i16 = 30_000;
const INFINITY: i16 = 31_000;

#[derive(Debug, Clone, Copy)]
pub enum SearchLimits {
    PerGame {
        clock: Duration,
        increment: Duration,
    },
    PerMove {
        depth: u8,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct SearchInfo {
    pub depth: u8,
    pub nodes: u64,
    pub eval: i16,
    pub time: Duration,
    pub best_move: Move,
}

pub struct Search {
    search_start: Instant,
    soft_limit: Duration,
    hard_limit: Duration,
    max_depth: u8,
    best_move: Option<Move>,
    nodes: u64,
}

impl Search {
    pub fn new(limits: SearchLimits) -> Self {
        let mut soft_limit = Duration::MAX;
        let mut hard_limit = Duration::MAX;
        let mut max_depth = u8::MAX;
        match limits {
            SearchLimits::PerGame { clock, .. } => {
                soft_limit = clock / 40;
                hard_limit = clock / 4;
            }
            SearchLimits::PerMove { depth } => {
                max_depth = depth;
            }
        }

        Self {
            search_start: Instant::now(),
            soft_limit,
            hard_limit,
            max_depth,
            best_move: None,
            nodes: 0,
        }
    }

    pub fn start(mut self, init_pos: &Board, moves_played: &[Move], on_iter: &mut dyn FnMut(SearchInfo)) {
        let mut board = BoardStack::new(init_pos, moves_played);
        for target_depth in 1..=self.max_depth {
            let Some(eval) = self.negamax(&mut board, target_depth, 0) else {
                break;
            };

            on_iter(SearchInfo {
                depth: target_depth,
                nodes: self.nodes,
                eval,
                time: self.search_start.elapsed(),
                best_move: self.best_move.expect("missing best move?"),
            });

            if self.search_start.elapsed() >= self.soft_limit {
                break;
            }
        }
    }

    fn negamax(&mut self, board: &mut BoardStack, depth: u8, ply: u8) -> Option<i16> {
        self.nodes += 1;

        if self.nodes % 1024 == 0 && self.best_move.is_some() && self.search_start.elapsed() >= self.hard_limit {
            return None;
        }

        if depth == 0 {
            return Some(evaluate(board.get()));
        }

        match board.get().status() {
            GameStatus::Won => return Some(-CHECKMATE + ply as i16),
            GameStatus::Drawn => return Some(0),
            GameStatus::Ongoing => {},
        }

        if board.repetitions() >= 3 {
            return Some(0);
        }

        let mut movelist = MoveList::new();
        board.get().generate_moves(|packed_moves| {
            movelist.extend(packed_moves);
            false
        });
    
        let mut best_move = None;
        let mut best_score = -INFINITY;
        for mv in movelist {
            board.play_unchecked(mv);
            let child_score = -self.negamax(board, depth - 1, ply + 1)?;
            board.undo();

            if child_score > best_score {
                best_move = Some(mv);
                best_score = child_score;
            }
        }

        if ply == 0 {
            self.best_move = best_move;
        }

        Some(best_score)
    }
}

fn evaluate(board: &Board) -> i16 {
    let mut eval = 0;
    let weights = [100, 300, 300, 500, 900, 0];
    for piece in Piece::ALL {
        let white_pieces = board.colored_pieces(Color::White, piece).len();
        let black_pieces = board.colored_pieces(Color::Black, piece).len();
        eval += (white_pieces as i16 - black_pieces as i16) * weights[piece as usize];
    }

    if board.side_to_move() == Color::Black {
        eval *= -1;
    }
    eval
}
