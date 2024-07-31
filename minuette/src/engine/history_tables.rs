use cozy_chess::{Board, Color, Move, Piece, Square};

pub struct HistoryTables {
    piece_to: [[[i32; Square::NUM]; Piece::NUM]; Color::NUM],
}

impl HistoryTables {
    pub const MAX_HISTORY: i32 = 512;

    pub fn new() -> Self {
        Self {
            piece_to: [[[0; Square::NUM]; Piece::NUM]; Color::NUM],
        }
    }

    pub fn get_quiet_score(&self, board: &Board, mv: Move) -> i32 {
        let color = board.side_to_move();
        let piece = board.piece_on(mv.from).expect("missing piece?");
        self.piece_to[color as usize][piece as usize][mv.to as usize]
    }

    pub fn update_move(&mut self, board: &Board, mv: Move, change: i32) {
        let color = board.side_to_move();
        let piece = board.piece_on(mv.from).expect("missing piece?");
        let score = &mut self.piece_to[color as usize][piece as usize][mv.to as usize];

        *score += change - change.abs() * *score / Self::MAX_HISTORY;
        *score = (*score).clamp(-Self::MAX_HISTORY, Self::MAX_HISTORY);
    }
}
