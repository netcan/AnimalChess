use sdl2::render::{Texture, WindowCanvas};
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::image::LoadTexture;
use std::time::Duration;
use std::collections::LinkedList;
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
    selected_chess: Option<(usize, usize)>,
    selected_frame: Texture,
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
            selected_frame: texture_creator.load_texture("assets/oos.gif").unwrap(),
            board_size: (0, 0),
            chess_size: (0, 0),
            selected_chess: None,
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

    fn get_dst_rect(&self, row: usize, col: usize) -> Rect {
        Rect::new(
            col as i32 * self.chess_size.0 as i32 + Self::CHESS_OFFSET.0,
            row as i32 * self.chess_size.1 as i32 + Self::CHESS_OFFSET.1,
            self.chess_size.0,
            self.chess_size.1
        )
    }

    fn draw_frame(&mut self, tgtPos: LinkedList<(usize, usize)>) -> Result<(), String> {
        for pos in tgtPos {
            self.canvas.copy(&self.selected_frame, None, self.get_dst_rect(pos.0, pos.1))?;
        }
        Ok(())
    }

    fn process_move(&mut self) -> Result<(), String> {
        if let Some((row, col)) = self.selected_chess {
            let mut tgtPos = LinkedList::new();
            tgtPos.push_back((row, col));

            match get_chess_type(self.chesses[row][col].id) {
                PAWN => {
                    tgtPos.append(&mut self.move_pawn((row, col)));
                }
                _ => {}
            }

            self.draw_frame(tgtPos)?;
        }
        Ok(())
    }

    fn render(&mut self) -> Result<(), String> {
        self.canvas.clear();
        self.canvas.copy(&self.board, None, None)?;
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                self.canvas.copy(&self.chesses[i][j].texture,
                    None, self.get_dst_rect(i, j))?;
            }
        }

        self.process_move()?;

        self.canvas.present();
        Ok(())
    }

    // check pos is outrange
    fn check_pos_inrange(pos: &(usize, usize)) -> bool {
        return pos.0 < ROW_NUM && pos.1 < COL_NUM;
    }

    fn move_pawn(&self, pos: (usize, usize)) -> LinkedList<(usize, usize)> {
        let mut result = LinkedList::new();
        match get_chess_role(self.chesses[pos.0][pos.1].id) {
            RED => {
                result.push_back((pos.0 - 1, pos.1));
                if pos.0 < 5 {
                    result.push_back((pos.0, pos.1 - 1));
                    result.push_back((pos.0, pos.1 + 1));
                }
            },
            BLACK => {
                result.push_back((pos.0 + 1, pos.1));
                if pos.0 >= 5 {
                    result.push_back((pos.0, pos.1 - 1));
                    result.push_back((pos.0, pos.1 + 1));
                }
            },
            _ => { unreachable!(); }
        }

        result.into_iter().filter(Self::check_pos_inrange).collect()
    }

    fn get_click_rect(&self, mut pos: (i32, i32)) -> Option<(usize, usize)> {
        pos.0 -= Self::CHESS_OFFSET.0;
        pos.1 -= Self::CHESS_OFFSET.1;
        if pos.0 < 0 || pos.1 < 0 { return None; }

        let row = (pos.1 / self.chess_size.1 as i32) as usize;
        let col = (pos.0 / self.chess_size.0 as i32) as usize;

        Some((row, col))
    }

    fn process_click(&mut self, pos: (i32, i32)) {
        if let Some((row, col)) = self.get_click_rect(pos) {
            if self.chesses[row][col].id == EMPTY || get_chess_role(self.chesses[row][col].id) != self.role {
                self.selected_chess = None;
                return;
            }
            println!("selected_chess: ({}, {})", row, col);
            self.selected_chess = Some((row, col));
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        'running: loop {
            // handle event
            let mut click_pos = (0, 0);
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'running }
                    Event::MouseButtonDown {x, y, ..} => { click_pos = (x, y); }
                    _ => {}
                }
            }

            self.process_click(click_pos);

            // update
            self.render()?;

            // time management
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }
}
