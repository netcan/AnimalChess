pub mod alpha_beta;
use crate::board::*;
pub use alpha_beta::AlphaBeta;

pub trait Player {
    fn get_move(&mut self) -> MOVE;
}

