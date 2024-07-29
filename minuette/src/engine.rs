use cozy_chess::{Board, Move};

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub wtime: u32,
    pub btime: u32,
    pub winc: u32,
    pub binc: u32,
}

pub struct EngineInfo {
    pub best_move: Move,
}

pub struct Engine {
    rng: u128,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            rng: 0x6609AAF0E34B81963138062ACF2EDF39u128,
        }
    }

    pub fn reset(&mut self) {
        
    }

    pub fn think(
        &mut self,
        init_pos: &Board,
        moves_played: &[Move],
        _clock: Clock,
        _on_iter: &mut dyn FnMut(EngineInfo),
    ) -> EngineInfo {
        
        let mut current_pos = init_pos.clone();
        for &mv in moves_played {
            current_pos.play(mv);
        }
        
        let mut moves = Vec::new();
        current_pos.generate_moves(|packed_moves| {
            moves.extend(packed_moves);
            false
        });

        self.rng = self.rng.wrapping_mul(0xDA942042E4DD58B5);
        EngineInfo {
            best_move: moves[(self.rng >> 96) as usize % moves.len()],
        }
    }
}
