use std::time::{Duration, Instant};

use cozy_chess::{Board, Piece, Move, GameStatus};

use super::board_stack::BoardStack;
use super::movelist::get_ordered_moves;
use super::eval::{evaluate, CHECKMATE, INFINITY};
use super::tt::{TranspositionTable, TtEntry, TtBound};
use super::history_tables::HistoryTables;
use super::helpers::move_is_capture;

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
    history: &'s mut HistoryTables,
    search_start: Instant,
    soft_limit: Duration,
    hard_limit: Duration,
    max_depth: u8,
    best_move: Option<Move>,
    nodes: u64,
}

impl<'s> Search<'s> {
    pub fn new(tt: &'s mut TranspositionTable, history: &'s mut HistoryTables, limits: SearchLimits) -> Self {
        let mut soft_limit = Duration::MAX;
        let mut hard_limit = Duration::MAX;
        let mut max_depth = u8::MAX;
        match limits {
            SearchLimits::PerGame { clock, increment } => {
                soft_limit = clock / 40;
                hard_limit = clock / 4;
                let _ = increment; // pretend we account for increment
            }
            SearchLimits::PerMove { depth } => {
                max_depth = depth;
            }
        }

        Self {
            tt,
            history,
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
            let Some(eval) = self.negamax(&mut board, -INFINITY, INFINITY, target_depth as i32, 0) else {
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

    fn negamax(&mut self, board: &mut BoardStack, mut alpha: i16, beta: i16, mut depth: i32, ply: u16) -> Option<i16> {
        assert!((-INFINITY..=INFINITY).contains(&alpha));
        assert!((-INFINITY..=INFINITY).contains(&beta));
        assert!(alpha < beta);

        if !board.get().checkers().is_empty() {
            depth = depth.max(0) + 1;
        }

        if depth <= 0 {
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
            let should_cutoff = !is_pv && tt_entry.depth as i32 >= depth && match tt_entry.bound {
                TtBound::Exact => true,
                TtBound::Lower => tt_entry.score >= beta,
                TtBound::Upper => tt_entry.score <= alpha,
            };
            if should_cutoff {
                return Some(tt_entry.score);
            }
        }

        let static_eval = evaluate(board.get());
        
        if !is_pv && depth <= 4 {
            let rfp_margin = depth as i16 * 80;
            if static_eval - rfp_margin >= beta {
                return Some(static_eval - rfp_margin);
            }
        }

        let kings = board.get().pieces(Piece::King);
        let pawns = board.get().pieces(Piece::Pawn);
        let only_pawns = board.get().occupied() == kings | pawns;
        if !is_pv && !only_pawns && depth >= 2 && static_eval >= beta && board.null_move() {
            let reduction = 2 + (static_eval as i32 - beta as i32) / 200;
            let score = -self.negamax(board, -beta, -beta + 1, depth - 1 - reduction, ply + 1)?;
            board.undo();

            if score >= beta {
                return Some(score);
            }
        }

        let mut quiets_to_check = if !is_pv {
            match depth {
                1 => 10,
                2 => 13,
                3 => 16,
                4 => 19,
                _ => i32::MAX,
            }
        } else {
            i32::MAX
        };

        let mut best_move = None;
        let mut best_score = -INFINITY;
        let movelist = get_ordered_moves(board.get(), tt_entry, self.history, false);
        for (i, &mv) in movelist.iter().enumerate() {
            let is_capture = move_is_capture(board.get(), mv);
            let mut reduction = (i as i32 * 10 + depth * 15) / 100;
            reduction -= self.history.get_quiet_score(board.get(), mv) / 200;
            if reduction < 0 || is_capture {
                reduction = 0;
            }

            if i != 0 && !is_capture {
                if quiets_to_check == 0 {
                    break;
                }
                quiets_to_check -= 1;
            }

            let mut score = -INFINITY;
            board.play_unchecked(mv);

            if i != 0 {
                score = -self.negamax(board, -alpha - 1, -alpha, depth - 1 - reduction, ply + 1)?;
            }
            
            if i != 0 && reduction != 0 && score > alpha {
                score = -self.negamax(board, -alpha - 1, -alpha, depth - 1, ply + 1)?;
            }
            
            if i == 0 || score > alpha {
                score = -self.negamax(board, -beta, -alpha, depth - 1, ply + 1)?;
            }

            board.undo();

            if score > best_score {
                best_move = Some(mv);
                best_score = score;
                alpha = alpha.max(score);
            }

            if score >= beta {
                if !is_capture {
                    let change = depth as i32 * depth as i32;
                    for &mv in &movelist[..i] {
                        if !move_is_capture(board.get(), mv) {
                            self.history.update_move(board.get(), mv, -change);
                        }
                    }
                    self.history.update_move(board.get(), mv, change);
                }

                break;
            }
        }

        let best_move = best_move.expect("missing best move?");
        if ply == 0 {
            self.best_move = Some(best_move);
        }

        // TODO mate correction
        self.tt.store(board.get().hash(), TtEntry {
            best_move: match alpha > init_alpha {
                true => Some(best_move),
                false => tt_entry.and_then(|entry| entry.best_move),
            },
            depth: depth.clamp(0, u8::MAX as i32) as u8,
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

        for mv in get_ordered_moves(board.get(), tt_entry, self.history, true) {
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
