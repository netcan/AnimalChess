use animal_chess_core::board::Board as Brd;
use animal_chess_core::board::MOVE;
use animal_chess_core::chess::*;
use pyo3::prelude::*;

#[pyclass]
struct Board {
    board: Brd
}

#[pyclass]
struct Role { }

/// Role type, which has two constanst value: RED and BLACK
/// for Board::check_win() to get win roles, None if current nobody wins.
#[pymethods]
impl Role {
    #[classattr]
    #[name = "RED"]
    fn red() -> i32 { 0 }

    #[classattr]
    #[name = "BLACK"]
    fn black() -> i32 { 1 }
}

#[pymethods]
impl Board {
    #[new]
    fn new(fen: &str) -> Self {
        let mut board = Brd::new();
        board.load_fen(fen);
        Self { board }
    }

    fn check_win(&self) -> PyResult<Option<i32>> {
        Ok(
            match self.board.check_win() {
                RoleType::RED => Some(Role::red()),
                RoleType::BLACK => Some(Role::black()),
                _ => None
            }
        )
    }

    fn generate_all_steps(&self) -> PyResult<Vec<MOVE>> {
        Ok(self.board.generate_all_steps())
    }

    fn move_chess(&mut self, mv: MOVE) -> PyResult<()> {
        Ok(self.board.move_chess(mv))
    }

    fn undo_move(&mut self) -> PyResult<()> {
        Ok(self.board.undo_move())
    }
}

/// This module is wrap animal chess board to python3, for reinforcement learning training.
#[pymodule]
fn animal_chess_pymodule(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Role>()?;
    m.add_class::<Board>()?;

    Ok(())
}
