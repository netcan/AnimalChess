use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::rect::Rect;
use crate::chess::*;

const ROW_NUM: usize = 10;
const COL_NUM: usize = 9;

pub struct Game {
    board: [[Box<Chess>; COL_NUM]; ROW_NUM],
}

impl Game {
    pub fn new<T>(texture_creator: &TextureCreator<T>) -> Self {
        let game = Game {
            board: array![
                array![Box::new(Chess::new(get_chess_id(RED, KING), texture_creator)); COL_NUM];
            ROW_NUM]
        };
        game
    }

    pub fn render_chess(&self, canvas: &mut WindowCanvas) {
        for i in 0..ROW_NUM {
            for j in 0..COL_NUM {
                canvas.copy((self.board[i][j].texture.as_ref().unwrap()), None, Rect::new(
                    j as i32 * 57, i as i32 * 57, 57, 57
                ));
            }
        }
    }
}
