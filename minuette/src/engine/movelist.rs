use arrayvec::ArrayVec;
use cozy_chess::{Board, Rank, Square, Piece, Move};

use super::tt::TtEntry;

type MoveList = ArrayVec<Move, 218>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MoveScore {
    Quiet,
    Capture(i32),
    PvMove,
}

pub fn get_ordered_moves(board: &Board, qsearch: bool, tt_entry: Option<TtEntry>) -> MoveList {
    let mut movelist = MoveList::new();
    board.generate_moves(|packed_moves| {
        movelist.extend(packed_moves);
        false
    });

    if qsearch {
        movelist.retain(|&mut mv| captured_piece(mv, board).is_some());
    }

    let key_fn = |mv| {
        if Some(mv) == tt_entry.map(|entry| entry.best_move) {
            return MoveScore::PvMove;
        }

        if let Some(victim) = captured_piece(mv, board) {
            let attacker = board.piece_on(mv.from).expect("missing attacker?");
            return MoveScore::Capture(victim as i32 * 8 - attacker as i32);
        }

        MoveScore::Quiet
    };
    movelist.sort_by_key(|&mv| std::cmp::Reverse(key_fn(mv)));

    movelist
}

fn captured_piece(mv: Move, board: &Board) -> Option<Piece> {
    let enemy_pieces = board.colors(!board.side_to_move());
    if enemy_pieces.has(mv.to) {
        return board.piece_on(mv.to);
    }

    let is_pawn_move = board.pieces(Piece::Pawn).has(mv.from);
    let ep_square = board.en_passant().map(|file| {
        let rank = Rank::Sixth.relative_to(board.side_to_move());
        Square::new(file, rank)
    });
    if is_pawn_move && Some(mv.to) == ep_square {
        return Some(Piece::Pawn);
    }

    None
}
