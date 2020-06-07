use sdl2::render::{Texture, WindowCanvas};
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::image::LoadTexture;
use std::time::Duration;
use std::collections::VecDeque;
use crate::chess::*;

pub type POS = u8;
pub type MOVE = u16;
pub type ScoreType = i32;
type HisTblType = [[[ScoreType; COL_NUM]; ROW_NUM]; 16];

pub const ROW_NUM: usize = 9;
pub const COL_NUM: usize = 7;

const BOARD_WIDTH: u32 = 500;
const BOARD_HEIGHT: u32 = 636;
const CELL_WIDTH: u32 = 70;
const CELL_HEIGHT: u32 = 70;

const CHESS_WIDTH: u32 = 64;
const CHESS_HEIGHT: u32 = 64;

const RED_DEN:   POS = 0x83;
const BLACK_DEN: POS = 0x3;

fn get_pos(pos: POS) -> (usize, usize) {
    ((pos >> 4) as usize, (pos & 0xf) as usize)
}

pub fn get_move(mv: MOVE) -> ((usize, usize), (usize, usize)) {
    let (src, dst) = (get_src_pos(mv), get_dst_pos(mv));
    (get_pos(src as POS), get_pos(dst as POS))
}

fn get_src_pos(mv: MOVE) -> POS {
    (mv >> 8) as POS
}

fn get_dst_pos(mv: MOVE) -> POS {
    (mv & 0xff) as POS
}

pub fn to_pos(pos: &(usize, usize)) -> POS {
    ((pos.0 << 4) | pos.1) as POS
}

fn to_move(mv: &((usize, usize), (usize, usize))) -> MOVE {
    ((to_pos(&mv.0) as MOVE) << 8) | to_pos(&mv.1) as MOVE
}

struct Context {
    eated: ChessId,
    mv: MOVE,
}

impl Context {
    fn new(eated: ChessId, mv: MOVE) -> Self {
        Self { eated, mv }
    }
}

pub struct Game {
    pub chesses: [[ChessId; COL_NUM]; ROW_NUM],
    chesses_textures: Vec<Texture>,
    pub role: Role, // 轮到谁下
    board: Texture,
    canvas: WindowCanvas,
    event_pump: EventPump,
    selected_chess: Option<POS>,
    selected_frame: Texture,
    movable_pos: Vec<MOVE>,
    pub compture_turn: bool,
    pub compture_mv: Option<MOVE>,
    ctx: VecDeque<Context>,
    pub history_table: HisTblType,
}

impl Game {
    const CHESS_OFFSET: (i32, i32) = (5, 3);

    // "l5t/1d3c1/r1p1w1e/7/7/7/E1W1P1R/1C3D1/T5L w - - 0 1"
    fn load_fen(&mut self, fen: &str) {
        let fen_u8 = fen.as_bytes();
        let mut fen_idx = 0;

        let get_role = |c: u8| -> Role {
            if (c as char).is_lowercase() { BLACK }
            else { RED }
        };

        let mut pos = 0usize;
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
                self.chesses[pos / COL_NUM][pos % COL_NUM] = chess_id;
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
            chesses: [[EMPTY; COL_NUM]; ROW_NUM],
            chesses_textures: Vec::new(),
            role: RED,
            board: texture_creator.load_texture("assets/board.png").unwrap(),
            selected_frame: texture_creator.load_texture("assets/oos.gif").unwrap(),
            selected_chess: None,
            movable_pos: Vec::new(),
            compture_turn: false,
            compture_mv: None,
            ctx: VecDeque::new(),
            history_table: [[[0; COL_NUM]; ROW_NUM]; 16],
            canvas,
            event_pump,
        };

        for chess_id in 8..24 {
            game.chesses_textures.push(get_chess_texture(chess_id, &texture_creator));
        }

        game.load_fen("l5t/1d3c1/r1p1w1e/7/7/7/E1W1P1R/1C3D1/T5L w - - 0 1");

        game.canvas.set_logical_size(
            BOARD_WIDTH,
            BOARD_HEIGHT,
        ).expect("set logical_size failed");
        game
    }

    fn get_dst_rect(&self, pos: POS) -> Rect {
        let (row, col) = get_pos(pos);
        Rect::new(
            col as i32 * CELL_WIDTH as i32 + (CELL_WIDTH - CHESS_WIDTH) as i32 / 2 + Self::CHESS_OFFSET.0,
            row as i32 * CELL_HEIGHT as i32 + (CELL_HEIGHT - CHESS_HEIGHT) as i32 / 2 + Self::CHESS_OFFSET.1,
            CHESS_WIDTH,
            CHESS_HEIGHT,
        )
    }

    fn draw_frame(&mut self, tgt_pos: &Vec<POS>) -> Result<(), String> {
        for pos in tgt_pos {
            self.canvas.copy(&self.selected_frame, None, self.get_dst_rect(*pos))?;
        }
        Ok(())
    }

    fn process_selected_chess(&mut self) -> Result<(), String> {
        if let Some(pos) = self.selected_chess {
            let mut tgt_pos = Vec::new();
            tgt_pos.push(pos);
            self.draw_frame(&tgt_pos)?;

            let movable_pos = self.generate_steps(pos);
            self.draw_frame(&movable_pos.iter().map(|&mv| { get_dst_pos(mv) }).collect::<Vec<POS>>())?;
            self.movable_pos = movable_pos;
        }
        Ok(())
    }

    fn render(&mut self) -> Result<(), String> {
        self.canvas.clear();
        self.canvas.copy(&self.board, None, None)?;
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let chess = self.chesses[i][j];
                if chess != EMPTY {
                    self.canvas.copy(&self.chesses_textures[get_chess_idx(chess)],
                        None, self.get_dst_rect(to_pos(&(i, j))))?;
                }
            }
        }

        self.process_selected_chess()?;

        self.canvas.present();
        Ok(())
    }

    fn get_src_dst_chess(&self, src: POS, dst: POS) -> (ChessId, ChessId) {
        let (src, dst) = (get_pos(src), get_pos(dst));
        (self.chesses[src.0][src.1],
         self.chesses[dst.0][dst.1])
    }

    fn check_movable(&self, src: POS, dst: POS) -> bool {
        {
            let src = get_pos(src);
            match (get_chess_role(self.chesses[src.0][src.1]), dst) {
                (RED, RED_DEN) | (BLACK, BLACK_DEN) => return false,
                _ => {}
            }
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
            (s, d)          => s <= d || self.check_in_traps(dst)
        }
    }

    fn check_in_water(pos: POS) -> bool {
        let pos = get_pos(pos);
        pos.0 >= 3 && pos.0 <= 5 && pos.1 % 3 != 0
    }

    fn check_rat(&self, src: POS, dst: POS) -> bool {
        let (src, dst) = (get_pos(src), get_pos(dst));
        if src.0 == dst.0 {
            for j in src.1.min(dst.1) ..= src.1.max(dst.1) {
                if get_chess_type(self.chesses[src.0][j]) == RAT && Self::check_in_water(to_pos(&(src.0, j))) {
                    return true;
                }
            }
        } else {
            for i in src.0.min(dst.0) ..= src.0.max(dst.0) {
                if get_chess_type(self.chesses[i][src.1]) == RAT && Self::check_in_water(to_pos(&(i, src.1))) {
                    return true;
                }
            }
        }
        false
    }

    fn pos_to_idx(pos: POS) -> usize {
        let pos = get_pos(pos);
        pos.0 * COL_NUM + pos.1
    }

    fn check_at_bank(pos: POS) -> bool {
        const BANK: u64 = 0xda4c992d8000;
        BANK & (1 << Self::pos_to_idx(pos)) > 0
    }

    pub fn check_in_den(&self, pos: POS) -> bool {
        let pos_ = get_pos(pos);
        match (get_chess_role(self.chesses[pos_.0][pos_.1]), pos) {
            (RED, BLACK_DEN) | (BLACK, RED_DEN) => return true,
            _ => { return false }
        }
    }

    pub fn check_in_traps(&self, pos: POS) -> bool {
        const TRAP: u64 = 0x1410000000000414;

        let pos_ = get_pos(pos);
        if TRAP & (1 << Self::pos_to_idx(pos)) > 0 {
            if get_chess_role(self.chesses[pos_.0][pos_.1]) == RED {
                return pos_.0 <= 1;
            } else {
                return pos_.0 >= 7;
            }
        }

        false
    }

    fn generate_tl_steps(&self, src: POS) -> Vec<MOVE> {
        let mut basic_steps = self.generate_basic_steps(src, false);
        let src_ = get_pos(src);
        if Self::check_at_bank(src) {
            // [2, 6]
            if (src_.0 + 2) % 4 == 0 {
                basic_steps.push(to_move(&(src_, ((src_.0 + 4) % 8, src_.1))));
            } else {
                if src_.1 % 6 == 0 {
                    basic_steps.push(to_move(&(src_, (src_.0, 3))));
                } else {
                    basic_steps.push(to_move(&(src_, (src_.0, 0))));
                    basic_steps.push(to_move(&(src_, (src_.0, 6))));
                }
            }
        }
        basic_steps.into_iter().filter(|&mv| {
            let (src, dst) = (get_src_pos(mv), get_dst_pos(mv));
            self.check_movable(src, dst) && !self.check_rat(src, dst)
        }).collect()
    }

    fn generate_basic_steps(&self, src: POS, to_water: bool) -> Vec<MOVE> {
        const DX: [i32; 4] = [1, 0, -1, 0];
        const DY: [i32; 4] = [0, 1, 0, -1];
        let src_ = get_pos(src);
        let (x, y) = (src_.0 as i32, src_.1 as i32);
        let mut result = Vec::new();
        result.reserve(8);
        for i in 0..4 {
            let (xx, yy) = (x + DX[i], y + DY[i]);
            if xx < 0 || xx >= ROW_NUM as i32 ||
                yy < 0 || yy >= COL_NUM as i32 {
                    continue;
            }
            let dst = (xx as usize, yy as usize);
            if self.check_movable(src, to_pos(&dst)) {
                if ! Self::check_in_water(to_pos(&dst)) || to_water {
                    result.push(to_move(&(src_, dst)))
                }
            }
        }
        result
    }

    pub fn generate_steps(&self, pos: POS) -> Vec<MOVE> {
        let pos_ = get_pos(pos);
        match get_chess_type(self.chesses[pos_.0][pos_.1]) {
            RAT =>          { self.generate_basic_steps(pos, true)  }
            TIGER | LION => { self.generate_tl_steps(pos)           }
            _ =>            { self.generate_basic_steps(pos, false) }
        }
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
                    else { RED };
        self.compture_turn = ! self.compture_turn;
    }

    pub fn move_chess(&mut self, mv: MOVE) {
        let (src, dst) = get_move(mv);

        let eated = self.chesses[dst.0][dst.1];
        self.chesses[dst.0][dst.1] = self.chesses[src.0][src.1];
        self.chesses[src.0][src.1] = EMPTY;

        self.ctx.push_back(Context::new(eated, mv));
        self.switch_player()
    }

    pub fn undo_move(&mut self) {
        if let Some(context) = self.ctx.pop_back() {
            let (src, dst) = get_move(context.mv);

            self.chesses[src.0][src.1] = self.chesses[dst.0][dst.1];
            self.chesses[dst.0][dst.1] = context.eated;

            self.switch_player()
        }
    }

    fn process_click(&mut self, pos: (i32, i32)) {
        if let Some(dst) = self.get_click_rect(pos) {
            if get_chess_role(self.chesses[dst.0][dst.1]) != self.role {
                // may be move
                if let Some(_) = self.movable_pos.iter().find(|&&mv| { return get_dst_pos(mv) == to_pos(&dst) }) {
                    let src = self.selected_chess.unwrap();
                    self.move_chess(to_move(&(get_pos(src), dst)));
                }
                self.selected_chess = None;
            } else { // must be selected, because role is same as chess
                println!("selected_chess: {:?}", to_pos(&dst));
                self.selected_chess = Some(to_pos(&dst));
                return;
            }

            self.movable_pos.clear();
        }
    }

    pub fn check_win(&self) -> Role {
        let (mut red_chess_num, mut black_chess_num) = (0, 0);

        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let chess_id = self.chesses[i][j];
                if chess_id == EMPTY { continue; }

                if self.check_in_den(to_pos(&(i, j))) { return get_chess_role(chess_id); }

                if get_chess_role(chess_id) == RED {
                    red_chess_num += 1;
                } else {
                    black_chess_num += 1;
                }
            }
        }
        if red_chess_num * black_chess_num == 0 {
            if red_chess_num > 0 { return RED; }
            else { return BLACK; }
        }
        EMPTY
    }

    pub fn run(&mut self) -> Result<(), String> {
        'running: loop {
            // handle event
            let mut click_pos = (0, 0);
            let mut undo = false;

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => { break 'running }
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        match keycode {
                            Keycode::Escape => { break 'running }
                            Keycode::U      => { undo = true; }
                            _ => {}
                        }
                    }
                    Event::MouseButtonDown {x, y, ..} => { click_pos = (x, y); }
                    _ => {}
                }
            }

            if undo {
                self.undo_move();
                if self.compture_turn { self.undo_move(); }
            }

            let win_status = self.check_win();
            if win_status == EMPTY {
                self.process_click(click_pos);
                // update
                self.render()?;
                self.search_main();
            } else {
                self.render()?;
                println!("{} wins!", if win_status == RED { "RED" } else { "BLACK" });
            }

            // time management
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }
}
