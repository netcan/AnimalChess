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

const ROW_NUM: usize = 9;
const COL_NUM: usize = 7;

const BOARD_WIDTH: u32 = 500;
const BOARD_HEIGHT: u32 = 636;
const CELL_WIDTH: u32 = 70;
const CELL_HEIGHT: u32 = 70;

const CHESS_WIDTH: u32 = 64;
const CHESS_HEIGHT: u32 = 64;

const RED_DEN:   (usize, usize) = (8, 3);
const BLACK_DEN: (usize, usize) = (0, 3);

pub struct Game {
    chesses: [[Box<Chess>; COL_NUM]; ROW_NUM],
    role: Role, // 轮到谁下
    board: Texture,
    canvas: WindowCanvas,
    event_pump: EventPump,
    selected_chess: Option<(usize, usize)>,
    selected_frame: Texture,
    movable_pos: LinkedList<(usize, usize)>,
}

impl Game {
    const CHESS_OFFSET: (i32, i32) = (5, 3);

    // "l5t/1d3c1/r1p1w1e/7/7/7/E1W1P1R/1C3D1/T5L w - - 0 1"
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
                c @ b'e' | c @ b'E' => { chess_id = get_chess_id(get_role(c), ELEPHANT); }
                c @ b'l' | c @ b'L' => { chess_id = get_chess_id(get_role(c), LION);     }
                c @ b't' | c @ b'T' => { chess_id = get_chess_id(get_role(c), TIGER);    }
                c @ b'p' | c @ b'P' => { chess_id = get_chess_id(get_role(c), PANTHER);  }
                c @ b'w' | c @ b'W' => { chess_id = get_chess_id(get_role(c), WOLF);     }
                c @ b'd' | c @ b'D' => { chess_id = get_chess_id(get_role(c), DOG);      }
                c @ b'c' | c @ b'C' => { chess_id = get_chess_id(get_role(c), CAT);      }
                c @ b'r' | c @ b'R' => { chess_id = get_chess_id(get_role(c), RAT);      }
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
            board: texture_creator.load_texture("assets/board.png").unwrap(),
            selected_frame: texture_creator.load_texture("assets/oos.gif").unwrap(),
            selected_chess: None,
            movable_pos: LinkedList::new(),
            canvas,
            event_pump,
        };

        game.load_fen("l5t/1d3c1/r1p1w1e/7/7/7/E1W1P1R/1C3D1/T5L w - - 0 1");

        game.canvas.set_logical_size(
            BOARD_WIDTH,
            BOARD_HEIGHT,
        ).expect("set logical_size failed");
        game
    }

    fn get_dst_rect(&self, row: usize, col: usize) -> Rect {
        Rect::new(
            col as i32 * CELL_WIDTH as i32 + (CELL_WIDTH - CHESS_WIDTH) as i32 / 2 + Self::CHESS_OFFSET.0,
            row as i32 * CELL_HEIGHT as i32 + (CELL_HEIGHT - CHESS_HEIGHT) as i32 / 2 + Self::CHESS_OFFSET.1,
            CHESS_WIDTH,
            CHESS_HEIGHT,
        )
    }

    fn draw_frame(&mut self, tgt_pos: &LinkedList<(usize, usize)>) -> Result<(), String> {
        for pos in tgt_pos {
            self.canvas.copy(&self.selected_frame, None, self.get_dst_rect(pos.0, pos.1))?;
        }
        Ok(())
    }

    fn process_selected_chess(&mut self) -> Result<(), String> {
        if let Some((row, col)) = self.selected_chess {
            let mut tgt_pos = LinkedList::new();
            tgt_pos.push_back((row, col));

            let mut movable_pos = LinkedList::new();
            match get_chess_type(self.chesses[row][col].id) {
                RAT =>          { movable_pos = self.generate_basic_steps(&(row, col), true);  }
                TIGER | LION => { movable_pos = self.generate_tl_steps(&(row, col));           }
                _ =>            { movable_pos = self.generate_basic_steps(&(row, col), false); }
            }

            self.draw_frame(&tgt_pos)?;
            self.draw_frame(&movable_pos)?;
            self.movable_pos = movable_pos;
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

        self.process_selected_chess()?;

        self.canvas.present();
        Ok(())
    }

    fn get_src_dst_chess(&self, src: &(usize, usize), dst: &(usize, usize)) -> (ChessId, ChessId) {
        (self.chesses[src.0][src.1].id,
         self.chesses[dst.0][dst.1].id)
    }

    fn check_movable(&self, src: &(usize, usize), dst: &(usize, usize)) -> bool {
        match (get_chess_role(self.chesses[src.0][src.1].id), dst) {
            (RED, &RED_DEN) | (BLACK, &BLACK_DEN) => return false,
            _ => {}
        }

        let (src_chess, dst_chess) = self.get_src_dst_chess(src, dst);
        if dst_chess == EMPTY { return true; }
        if get_chess_role(src_chess) == get_chess_role(dst_chess) { return false; }

        let (src_chess_type, dst_chess_type) = (
            get_chess_type(src_chess), get_chess_type(dst_chess)
        );

        match (src_chess_type, dst_chess_type) {
            (RAT, ELEPHANT) => ! Self::check_in_water(src),
            (ELEPHANT, RAT) => false,
            (s, d)          => s <= d || self.check_in_traps(&dst)
        }
    }

    fn check_in_water(src: &(usize, usize)) -> bool {
        src.0 >= 3 && src.0 <= 5 && src.1 % 3 != 0
    }

    fn check_rat(&self, src: &(usize, usize), dst: &(usize, usize)) -> bool {
        if src.0 == dst.0 {
            for j in src.1.min(dst.1) ..= src.1.max(dst.1) {
                if get_chess_type(self.chesses[src.0][j].id) == RAT && Self::check_in_water(&(src.0, j)) {
                    return true;
                }
            }
        } else {
            for i in src.0.min(dst.0) ..= src.0.max(dst.0) {
                if get_chess_type(self.chesses[i][src.1].id) == RAT && Self::check_in_water(&(i, src.1)) {
                    return true;
                }
            }
        }
        false
    }

    fn pos_to_idx(pos: &(usize, usize)) -> usize {
        pos.0 * COL_NUM + pos.1
    }

    fn check_at_bank(pos: &(usize, usize)) -> bool {
        const BANK: u64 = 0xda4c992d8000;
        BANK & (1 << Self::pos_to_idx(pos)) > 0
    }

    fn check_in_traps(&self, pos: &(usize, usize)) -> bool {
        const TRAP: u64 = 0x1410000000000414;

        if TRAP & (1 << Self::pos_to_idx(pos)) > 0 {
            if get_chess_role(self.chesses[pos.0][pos.1].id) == RED {
                return pos.0 <= 1;
            } else {
                return pos.0 >= 7;
            }
        }

        false
    }

    fn generate_tl_steps(&self, src: &(usize, usize)) -> LinkedList<(usize, usize)> {
        let mut basic_steps = self.generate_basic_steps(src, false);
        if Self::check_at_bank(src) {
            // [2, 6]
            if (src.0 + 2) % 4 == 0 {
                basic_steps.push_back(((src.0 + 4) % 8, src.1));
            } else {
                if src.1 % 6 == 0 {
                    basic_steps.push_back((src.0, 3));
                } else {
                    basic_steps.push_back((src.0, 0));
                    basic_steps.push_back((src.0, 6));
                }
            }
        }
        basic_steps.into_iter().filter(|dst| {
            self.check_movable(src, dst) && !self.check_rat(src, dst)
        }).collect()
    }

    fn generate_basic_steps(&self, src: &(usize, usize), to_water: bool) -> LinkedList<(usize, usize)> {
        const DX: [i32; 4] = [1, 0, -1, 0];
        const DY: [i32; 4] = [0, 1, 0, -1];
        let (x, y) = (src.0 as i32, src.1 as i32);
        let mut result = LinkedList::new();
        for i in 0..4 {
            let (xx, yy) = (x + DX[i], y + DY[i]);
            if xx < 0 || xx >= ROW_NUM as i32 ||
                yy < 0 || yy >= COL_NUM as i32 {
                    continue;
            }
            let dst = (xx as usize, yy as usize);
            if self.check_movable(src, &dst) {
                if ! Self::check_in_water(&dst) || to_water {
                    result.push_back(dst)
                }
            }
        }
        result
    }

    fn get_empty_chess(&self) -> Box<Chess> {
        let texture_creator = self.canvas.texture_creator();
        Box::new(Chess::new(EMPTY, &texture_creator))
    }

    fn get_click_rect(&self, mut pos: (i32, i32)) -> Option<(usize, usize)> {
        pos.0 -= Self::CHESS_OFFSET.0;
        pos.1 -= Self::CHESS_OFFSET.1;
        if pos.0 < 0 || pos.1 < 0 { return None; }

        let row = (pos.1 / CELL_HEIGHT as i32) as usize;
        let col = (pos.0 / CELL_WIDTH as i32) as usize;

        Some((row, col))
    }

    fn switch_player(&mut self) {
        self.role = if self.role == RED { BLACK }
                    else { RED }
    }

    unsafe fn move_chess(&mut self, src: (usize, usize), dst: (usize, usize)) {
        let p_src: *mut Box<Chess> = &mut self.chesses[src.0][src.1];
        let p_dst: *mut Box<Chess> = &mut self.chesses[dst.0][dst.1];
        std::mem::swap(&mut *p_src, &mut *p_dst);
        *p_src = self.get_empty_chess();

        println!("{:?} -> {:?}", src, dst);
        self.switch_player()
    }

    fn process_click(&mut self, pos: (i32, i32)) {
        if let Some((row, col)) = self.get_click_rect(pos) {
            if self.chesses[row][col].id == EMPTY || get_chess_role(self.chesses[row][col].id) != self.role {
                // may be move
                if let Some(_) = self.movable_pos.iter().find(|&&(r, l)| { return (r, l) == (row, col) }) {
                    let src_pos = self.selected_chess.unwrap();
                    unsafe { self.move_chess(src_pos, (row, col)); }
                }
                self.selected_chess = None;
            } else {
                println!("selected_chess: ({}, {})", row, col);
                self.selected_chess = Some((row, col));
                return;
            }

            self.movable_pos.clear();
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
