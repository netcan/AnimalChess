use animal_chess_core::board::Board as Brd;
use animal_chess_core::board::{MOVE, ROW_NUM, COL_NUM, to_pos, RED_DEN, BLACK_DEN, TRAP};
use animal_chess_core::chess::{*, ChessKind::*};
use pyo3::class::basic::PyObjectProtocol;
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
    const RED: i32 = 0;

    #[classattr]
    const BLACK: i32 = 1;
}

#[pymethods]
impl Board {
    #[new]
    fn new(fen: Option<&str>) -> Self {
        let mut board = Brd::new();
        if let Some(fen) = fen {
            board.load_fen(fen);
        }
        Self { board }
    }

    fn check_win(&self) -> PyResult<Option<i32>> {
        Ok(match self.board.check_win() {
            RoleType::RED   => Some(Role::RED),
            RoleType::BLACK => Some(Role::BLACK),
            _ => None
        })
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


#[pyproto]
impl<'a> PyObjectProtocol<'a> for Board {
    fn __repr__(&'a self) -> PyResult<String>
    {
        let mut rep = String::new();
        const RED_COLOR: &'static str = "\x1b[31m";
        const BLACK_COLOR: &'static str = "\x1b[37m"; // use white color instead

        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let ChessId {role, kind} = self.board.chesses[i][j];
                let mut color: &str = "";

                let mut k = if i >= 3 && i <= 5 && j % 3 != 0 {
                    '~'
                } else {
                    ' '
                };

                if (TRAP & (1 << (i * COL_NUM + j) as u64)) > 0 {
                    color = if i <= 1 { BLACK_COLOR } else { RED_COLOR };
                    k = '#';
                }

                match to_pos(&(i, j)) {
                    RED_DEN => {
                        k = '@';
                        color = RED_COLOR;
                    }
                    BLACK_DEN => {
                        k = '@';
                        color = BLACK_COLOR;
                    }
                    _ => {}
                };

                k = match kind {
                    ELEPHANT => 'E',
                    LION     => 'L',
                    TIGER    => 'T',
                    PANTHER  => 'P',
                    WOLF     => 'W',
                    DOG      => 'D',
                    CAT      => 'C',
                    RAT      => 'R',
                    _        => k,
                };

                match role {
                    RoleType::RED => {
                        rep += &format!("{}{}\x1b[0m", RED_COLOR, k);
                    },
                    RoleType::BLACK => {
                        rep += &format!("{}\x1b[37m{}\x1b[0m", BLACK_COLOR, k.to_lowercase());
                    }
                    _ => {
                        rep += &format!("{}{}\x1b[0m", color, k);
                    }
                };
            }
            rep.push('\n');
        }

        Ok(rep)
    }
}

/// This module is wrap animal chess board to python3, for reinforcement learning training.
#[pymodule]
fn animal_chess_pymodule(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Role>()?;
    m.add_class::<Board>()?;

    Ok(())
}
