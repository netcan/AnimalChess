pub mod alpha_beta;
pub mod mcts;
use crate::board::*;

pub use alpha_beta::AlphaBeta;
pub use mcts::MCTSPlayer;

pub trait Player {
    fn get_move(&mut self) -> MOVE;
}

