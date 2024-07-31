use cozy_chess::{Board, Rank, Square, Piece, Move};

pub fn move_is_capture(board: &Board, mv: Move) -> bool {
    captured_piece(board, mv).is_some()
}

pub fn captured_piece(board: &Board, mv: Move) -> Option<Piece> {
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
