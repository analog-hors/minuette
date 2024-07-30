use cozy_chess::{Board, Move};

pub struct BoardStack {
    history: Vec<u64>,
    stack: Vec<Board>,
}

impl BoardStack {
    pub fn new(init_pos: &Board, moves_played: &[Move]) -> Self {
        let mut history = Vec::with_capacity(256);
        let mut board = init_pos.clone();
        for &mv in moves_played {
            history.push(board.hash());
            board.play(mv);
        }
        history.push(board.hash());

        let mut stack = Vec::with_capacity(256);
        stack.push(board);

        Self { history, stack }
    }

    pub fn get(&self) -> &Board {
        self.stack.last().expect("missing board?")
    }

    pub fn play_unchecked(&mut self, mv: Move) {
        let mut next = self.get().clone();
        next.play_unchecked(mv);

        self.history.push(next.hash());
        self.stack.push(next);
    }

    pub fn undo(&mut self) {
        self.history.pop();
        self.stack.pop();
    }

    pub fn repetitions(&self) -> usize {
        let hash = self.get().hash();
        self.history.iter().filter(|&&h| h == hash).count()
    }
}
