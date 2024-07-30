use cozy_chess::{Board, Move};

use super::search::{Search, SearchLimits, SearchInfo};

pub struct Engine {
    _priv: ()
}

impl Engine {
    pub fn new() -> Self {
        Self {
            _priv: (),
        }
    }

    pub fn reset(&mut self) {

    }

    pub fn think(
        &mut self,
        init_pos: &Board,
        moves_played: &[Move],
        limits: SearchLimits,
        on_iter: &mut dyn FnMut(SearchInfo),
    ) {
        let search = Search::new(limits);
        search.start(init_pos, moves_played, on_iter);
    }
}
