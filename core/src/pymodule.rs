use crate::board::Board as Brd;
use crate::board::{ROW_NUM, COL_NUM, MOVE};
use crate::chess::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::wrap_pyfunction;

#[pyclass]
struct Board {
    board: Brd
}


#[pyclass]
struct Role { }

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

#[pymodule]
/// A Python module implemented in Rust.
fn animal_chess_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Role>()?;
    m.add_class::<Board>()?;

    Ok(())
}
