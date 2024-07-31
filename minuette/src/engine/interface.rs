use cozy_chess::{Board, Move};

use super::tt::TranspositionTable;
use super::search::{Search, SearchInfo, SearchLimits};
use super::history_tables::HistoryTables;

pub struct Engine {
    tt: TranspositionTable,
    history: HistoryTables,
}

impl Engine {
    pub fn new(tt_bytes: usize) -> Self {
        Self {
            tt: TranspositionTable::new(tt_bytes),
            history: HistoryTables::new(),
        }
    }

    pub fn resize_tt(&mut self, tt_bytes: usize) {
        self.tt = TranspositionTable::new(tt_bytes);
    }

    pub fn reset(&mut self) {
        self.tt.clear();
        self.history = HistoryTables::new();
    }

    pub fn think(
        &mut self,
        init_pos: &Board,
        moves_played: &[Move],
        limits: SearchLimits,
        on_iter: &mut dyn FnMut(SearchInfo),
    ) {
        let search = Search::new(&mut self.tt, &mut self.history, limits);
        search.start(init_pos, moves_played, on_iter);
    }
}
