use cozy_chess::Move;

#[derive(Debug, Clone, Copy)]
pub enum TtBound {
    Upper,
    Exact,
    Lower,
}

#[derive(Debug, Clone, Copy)]
pub struct TtEntry {
    pub best_move: Option<Move>,
    pub depth: u8,
    pub score: i16,
    pub bound: TtBound,
}

type FullTtEntry = Option<(u64, TtEntry)>;

const _ASSERT_TT_ENTRY_SIZE: () = assert!(std::mem::size_of::<FullTtEntry>() == 16);

pub struct TranspositionTable {
    table: Vec<FullTtEntry>,
}

impl TranspositionTable {
    pub fn new(tt_bytes: usize) -> Self {
        Self {
            table: vec![None; tt_bytes / std::mem::size_of::<FullTtEntry>()],
        }
    }

    pub fn load(&self, hash: u64) -> Option<TtEntry> {
        let (entry_hash, entry) = self.table[self.index(hash)]?;
        (entry_hash == hash).then_some(entry)
    }

    pub fn store(&mut self, hash: u64, entry: TtEntry) {
        let index = self.index(hash);
        self.table[index] = Some((hash, entry));
    }

    pub fn clear(&mut self) {
        self.table.fill(None);
    }

    fn index(&self, hash: u64) -> usize {
        (hash as u128 * self.table.len() as u128 >> 64) as u64 as usize
    }
}
