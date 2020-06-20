/*************************************************************************
    > File Name: main.rs
    > Author: Netcan
    > Descripton: AnimalChess main
    > Blog: http://www.netcan666.com
    > Mail: 1469709759@qq.com
    > Created Time: 2020-06-20 19:24
************************************************************************/

use animal_chess_core::*;
mod gui;
use gui::*;

const WINDOW_WIDTH: u32 = 500;
const WINDOW_HEIGHT: u32 = 636;

fn main() -> Result<(), String> {
    let sdl_ctx = sdl2::init()?;
    let video_sys = sdl_ctx.video()?;

    let windows = video_sys.window("AnimalChess", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        // .resizable()
        .allow_highdpi()
        .build().expect("could not initialize video subsystem");

    let mut game = Game::new(windows, sdl_ctx.event_pump()?);

    game.run()?;

    Ok(())
}
