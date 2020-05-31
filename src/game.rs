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
    board: Texture,
    board_size: (u32, u32),
    chess_size: (u32, u32),
    canvas: WindowCanvas,
    event_pump: EventPump,
}

impl Game {
    const CHESS_OFFSET: (i32, i32) = (4, 3);
    pub fn new(window: Window, event_pump: EventPump) -> Self {
        let canvas = window.into_canvas()
            .present_vsync()
            .build().expect("could not make a canvas");

        let texture_creator = canvas.texture_creator();

        let mut game = Game {
            chesses: array![
                array![Box::new(Chess::new(get_chess_id(RED, KING), &texture_creator)); COL_NUM];
            ROW_NUM],
            board: texture_creator.load_texture("assets/board.jpg").unwrap(),
            board_size: (0, 0),
            chess_size: (0, 0),
            canvas,
            event_pump
        };

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
