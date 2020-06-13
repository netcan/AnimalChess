use sdl2::render::{Texture, WindowCanvas};
use sdl2::rect::Rect;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::image::LoadTexture;
use std::time::Duration;
use crate::chess::*;
use crate::board::*;
use crate::player::*;
use std::cell::RefCell;
use std::rc::Rc;

const BOARD_WIDTH: u32 = 500;
const BOARD_HEIGHT: u32 = 636;
const CELL_WIDTH: u32 = 70;
const CELL_HEIGHT: u32 = 70;

const CHESS_WIDTH: u32 = 64;
const CHESS_HEIGHT: u32 = 64;

pub struct Game {
    chesses_textures: Vec<Texture>,
    board: Rc<RefCell<Board>>,
    board_texture: Texture,
    canvas: WindowCanvas,
    event_pump: EventPump,
    computer: Box<dyn Player>,
    computer_turn: bool,
    selected_chess: Option<POS>,
    selected_frame: Texture,
    movable_pos: Vec<MOVE>,
}

impl Game {
    const CHESS_OFFSET: (i32, i32) = (5, 3);

    pub fn new(window: Window, event_pump: EventPump) -> Self {
        let canvas = window.into_canvas()
            .present_vsync()
            .build().expect("could not make a canvas");

        let texture_creator = canvas.texture_creator();

        let board = Rc::new(RefCell::new(Board::new()));
        let computer = Box::new(AlphaBeta::new(board.clone()));
        let mut game = Game {
            chesses_textures: Vec::new(),
            board,
            computer,
            computer_turn: false,
            board_texture: texture_creator.load_texture("assets/board.png").unwrap(),
            selected_frame: texture_creator.load_texture("assets/oos.gif").unwrap(),
            selected_chess: None,
            movable_pos: Vec::new(),
            canvas,
            event_pump,
        };

        for role in RoleType::iter() {
            for kind in ChessKind::iter() {
                game.chesses_textures.push(
                    ChessId { role: *role, kind: *kind }.get_chess_texture(&texture_creator)
                );
            }
        }

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
            self.draw_frame(&vec![pos])?;

            self.movable_pos = self.board.borrow().generate_steps(pos);
            self.draw_frame(&self.movable_pos.iter().map(|&mv| { get_dst_pos(mv) }).collect())?;
        }
        Ok(())
    }

    fn render(&mut self) -> Result<(), String> {
        self.canvas.clear();
        self.canvas.copy(&self.board_texture, None, None)?;
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                let chess = self.board.borrow().chesses[i][j];
                if chess != EMPTY_CHESS {
                    self.canvas.copy(&self.chesses_textures[chess.get_chess_idx()],
                        None, self.get_dst_rect(to_pos(&(i, j))))?;
                }
            }
        }

        self.process_selected_chess()?;

        self.canvas.present();
        Ok(())
    }

    fn get_click_rect(&self, mut pos: (i32, i32)) -> Option<(usize, usize)> {
        pos.0 -= Self::CHESS_OFFSET.0;
        pos.1 -= Self::CHESS_OFFSET.1;
        if pos.0 < 0 || pos.1 < 0 { return None; }

        let row = (pos.1 / CELL_HEIGHT as i32) as usize;
        let col = (pos.0 / CELL_WIDTH as i32) as usize;

        if row >= ROW_NUM || col >= COL_NUM { return None; }

        Some((row, col))
    }

    fn process_click(&mut self, pos: (i32, i32)) {
        if let Some(dst) = self.get_click_rect(pos) {
            let mut board = self.board.borrow_mut();
            if board.chesses[dst.0][dst.1].role != board.role {
                // may be move
                if let Some(_) = self.movable_pos.iter().find(|&&mv| { return get_dst_pos(mv) == to_pos(&dst) }) {
                    let src = self.selected_chess.unwrap();
                    board.move_chess(to_move(&(get_pos(src), dst)));
                    self.computer_turn = ! self.computer_turn;
                }
                self.selected_chess = None;
            } else { // must be selected, because role is same as chess
                println!("selected_chess: {:?}", to_pos(&dst));
                self.selected_chess = Some(to_pos(&dst));
            }

            self.movable_pos.clear();
        }
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

            {
                let mut board = self.board.borrow_mut();
                if undo {
                    board.undo_move();
                    board.undo_move();
                    self.selected_chess = None;
                }
            }

            let win_status = self.board.borrow().check_win();
            if win_status == RoleType::EMPTY {
                self.process_click(click_pos);
                // update
                self.render()?;
                if self.computer_turn && self.board.borrow().check_win() == RoleType::EMPTY {
                    let mv = self.computer.get_move();
                    self.board.borrow_mut().move_chess(mv);
                    self.computer_turn = ! self.computer_turn;
                }
            } else {
                self.render()?;
                println!("{:?} wins!", win_status);
                break;
            }

            // time management
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }

        Ok(())
    }
}
