use std::time::{Duration, Instant};

use cozy_chess::{Board, Move, GameStatus};

use super::board_stack::BoardStack;
use super::movelist::get_ordered_moves;
use super::eval::{evaluate, CHECKMATE, INFINITY};
use super::tt::{TranspositionTable, TtEntry, TtBound};

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

pub struct Search<'s> {
    tt: &'s mut TranspositionTable,
    search_start: Instant,
    soft_limit: Duration,
    hard_limit: Duration,
    max_depth: u8,
    best_move: Option<Move>,
    nodes: u64,
}

impl<'s> Search<'s> {
    pub fn new(tt: &'s mut TranspositionTable, limits: SearchLimits) -> Self {
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
            tt,
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
            let Some(eval) = self.negamax(&mut board, -INFINITY, INFINITY, target_depth, 0) else {
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

    fn negamax(&mut self, board: &mut BoardStack, mut alpha: i16, beta: i16, depth: u8, ply: u16) -> Option<i16> {
        assert!((-INFINITY..=INFINITY).contains(&alpha));
        assert!((-INFINITY..=INFINITY).contains(&beta));
        assert!(alpha < beta);

        if depth == 0 {
            if board.repetitions() >= 3 {
                return Some(0);
            }
            return Some(self.qsearch(board, alpha, beta, ply));
        }

        self.nodes += 1;

        if self.nodes % 1024 == 0 && self.best_move.is_some() && self.search_start.elapsed() >= self.hard_limit {
            return None;
        }

        match board.get().status() {
            GameStatus::Won => return Some(-CHECKMATE + ply as i16),
            GameStatus::Drawn => return Some(0),
            GameStatus::Ongoing => {},
        }
        if board.repetitions() >= 3 {
            return Some(0);
        }

        let is_pv = alpha + 1 != beta;
        let init_alpha = alpha;
        let tt_entry = self.tt.load(board.get().hash());
        if let Some(tt_entry) = tt_entry {
            let should_cutoff = !is_pv && tt_entry.depth >= depth && match tt_entry.bound {
                TtBound::Exact => true,
                TtBound::Lower => tt_entry.score >= beta,
                TtBound::Upper => tt_entry.score <= alpha,
            };
            if should_cutoff {
                return Some(tt_entry.score);
            }
        }

        let mut best_move = None;
        let mut best_score = -INFINITY;
        let movelist = get_ordered_moves(board.get(), false, tt_entry);
        for (i, mv) in movelist.into_iter().enumerate() {
            let mut child_score = -INFINITY;

            board.play_unchecked(mv);
            if i != 0 {
                child_score = -self.negamax(board, -alpha - 1, -alpha, depth - 1, ply + 1)?;
            }
            if i == 0 || child_score > alpha && child_score < beta {
                child_score = -self.negamax(board, -beta, -alpha, depth - 1, ply + 1)?;
            }
            board.undo();

            if child_score > best_score {
                best_move = Some(mv);
                best_score = child_score;
                alpha = alpha.max(child_score);
            }

            if child_score >= beta {
                break;
            }
        }

        let best_move = best_move.expect("missing best move?");
        if ply == 0 {
            self.best_move = Some(best_move);
        }

        // TODO mate correction
        self.tt.store(board.get().hash(), TtEntry {
            best_move,
            depth,
            score: best_score,
            bound: match () {
                _ if alpha >= beta => TtBound::Lower,
                _ if alpha > init_alpha => TtBound::Exact,
                _ => TtBound::Upper,
            }
        });
        Some(best_score)
    }

    fn qsearch(&mut self, board: &mut BoardStack, mut alpha: i16, beta: i16, ply: u16) -> i16 {
        assert!((-INFINITY..=INFINITY).contains(&alpha));
        assert!((-INFINITY..=INFINITY).contains(&beta));
        assert!(alpha < beta);

        self.nodes += 1;

        match board.get().status() {
            GameStatus::Won => return -CHECKMATE + ply as i16,
            GameStatus::Drawn => return 0,
            GameStatus::Ongoing => {},
        }

        let tt_entry = self.tt.load(board.get().hash());

        let mut best_score = evaluate(board.get());
        alpha = alpha.max(best_score);
        if best_score >= beta {
            return best_score;
        }

        for mv in get_ordered_moves(board.get(), true, tt_entry) {
            board.play_unchecked(mv);
            let child_score = -self.qsearch(board, -beta, -alpha, ply + 1);
            board.undo();

            if child_score > best_score {
                best_score = child_score;
                alpha = alpha.max(child_score);
            }

            if child_score >= beta {
                break;
            }
        }

        best_score
    }
}
