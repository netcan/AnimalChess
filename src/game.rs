use sdl2::render::{Texture, WindowCanvas};
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::image::LoadTexture;
use std::time::Duration;
use crate::chess::*;

const ROW_NUM: usize = 10;
const COL_NUM: usize = 9;

pub struct Game {
    chesses: [[Box<Chess>; COL_NUM]; ROW_NUM],
    role: Role, // 轮到谁下
    board: Texture,
    board_size: (u32, u32),
    chess_size: (u32, u32),
    canvas: WindowCanvas,
    event_pump: EventPump,
}

impl Game {
    const CHESS_OFFSET: (i32, i32) = (4, 3);

    // rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1
    fn load_fen(&mut self, fen: &str) {
        let texture_creator = self.canvas.texture_creator();
        let mut pos = 0usize;

        let fen_u8 = fen.as_bytes();
        let mut fen_idx = 0;

        let get_role = |c: u8| -> Role {
            if (c as char).is_lowercase() { BLACK }
            else { RED }
        };

        while fen_idx < fen_u8.len() {
            let mut chess_id = EMPTY;
            match fen_u8[fen_idx] {
                c @ b'k' | c @ b'K' => { chess_id = get_chess_id(get_role(c), KING);    }
                c @ b'a' | c @ b'A' => { chess_id = get_chess_id(get_role(c), ADVISOR); }
                c @ b'b' | c @ b'B' => { chess_id = get_chess_id(get_role(c), BISHOP);  }
                c @ b'n' | c @ b'N' => { chess_id = get_chess_id(get_role(c), KNIGHT);  }
                c @ b'r' | c @ b'R' => { chess_id = get_chess_id(get_role(c), ROOK);    }
                c @ b'c' | c @ b'C' => { chess_id = get_chess_id(get_role(c), CANNON);  }
                c @ b'p' | c @ b'P' => { chess_id = get_chess_id(get_role(c), PAWN);    }
                n @ b'1' ..= b'9'   => { pos += (n - b'0') as usize; }
                b'/' => { }
                b' ' => { break; }
                _    => { unreachable!() }
            }

            if chess_id != EMPTY {
                self.chesses[pos / COL_NUM][pos % COL_NUM] = Box::new(Chess::new(chess_id, &texture_creator));
                pos += 1;
            }
            fen_idx += 1;
        }
        fen_idx += 1; // eat ' '
        self.role = if fen_u8[fen_idx] == b'w' { RED }
                    else { BLACK };

    }

    pub fn new(window: Window, event_pump: EventPump) -> Self {
        let canvas = window.into_canvas()
            .present_vsync()
            .build().expect("could not make a canvas");

        let texture_creator = canvas.texture_creator();

        let mut game = Game {
            chesses: array![
                array![Box::new(Chess::new(EMPTY, &texture_creator)); COL_NUM];
            ROW_NUM],
            role: RED,
            board: texture_creator.load_texture("assets/board.jpg").unwrap(),
            board_size: (0, 0),
            chess_size: (0, 0),
            canvas,
            event_pump
        };

        game.load_fen("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1");

        {
            let query = game.board.query();
            game.board_size = (query.width, query.height);
            let query = game.chesses[0][0].texture.query();
            game.chess_size = (query.width, query.height);
        }

        game.canvas.set_logical_size(
            game.board_size.0,
            game.board_size.1,
        ).expect("set logical_size failed");
        game
    }

    pub fn render(&mut self) -> Result<(), String> {
        self.canvas.clear();
        self.canvas.copy(&self.board, None, None)?;
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                self.canvas.copy(&self.chesses[i][j].texture, None, Rect::new(
                    j as i32 * self.chess_size.0 as i32 + Self::CHESS_OFFSET.0,
                    i as i32 * self.chess_size.1 as i32 + Self::CHESS_OFFSET.1,
                    self.chess_size.0,
                    self.chess_size.1
                ))?;
            }
        }
        self.canvas.present();
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), String> {
        'running: loop {
            // handle event
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                            break 'running
                        },
                    _ => {}
                }
            }
            // update
            self.render()?;

            // time management
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }
}
