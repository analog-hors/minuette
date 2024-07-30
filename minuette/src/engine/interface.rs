use cozy_chess::{Board, Move};

use super::tt::TranspositionTable;
use super::search::{Search, SearchLimits, SearchInfo};

pub struct Engine {
    tt: TranspositionTable,
}

impl Engine {
    pub fn new(tt_bytes: usize) -> Self {
        Self {
            tt: TranspositionTable::new(tt_bytes),
        }
    }

    pub fn resize_tt(&mut self, tt_bytes: usize) {
        self.tt = TranspositionTable::new(tt_bytes);
    }

    pub fn reset(&mut self) {
        self.tt.clear();
    }

    pub fn think(
        &mut self,
        init_pos: &Board,
        moves_played: &[Move],
        limits: SearchLimits,
        on_iter: &mut dyn FnMut(SearchInfo),
    ) {
        let search = Search::new(&mut self.tt, limits);
        search.start(init_pos, moves_played, on_iter);
    }
}
