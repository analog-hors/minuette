use arrayvec::ArrayVec;
use cozy_chess::{Board, Move};

use super::tt::TtEntry;
use super::history_tables::HistoryTables;
use super::helpers::{move_is_capture, captured_piece};

type MoveList = ArrayVec<Move, 218>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MoveScore {
    Quiet(i32),
    Capture(i32),
    PvMove,
}

pub fn get_ordered_moves(board: &Board, tt_entry: Option<TtEntry>, history: &HistoryTables, qsearch: bool) -> MoveList {
    let mut movelist = MoveList::new();
    board.generate_moves(|packed_moves| {
        movelist.extend(packed_moves);
        false
    });

    if qsearch {
        movelist.retain(|&mut mv| move_is_capture(board, mv));
    }

    let key_fn = |mv| {
        if Some(mv) == tt_entry.and_then(|entry| entry.best_move) {
            return MoveScore::PvMove;
        }

        if let Some(victim) = captured_piece(board, mv) {
            let attacker = board.piece_on(mv.from).expect("missing attacker?");
            return MoveScore::Capture(victim as i32 * 8 - attacker as i32);
        }

        MoveScore::Quiet(history.get_quiet_score(board, mv))
    };
    movelist.sort_by_key(|&mv| std::cmp::Reverse(key_fn(mv)));

    movelist
}
