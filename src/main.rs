#[macro_use]
extern crate array_macro;

mod game;
mod chess;

use game::*;

const WINDOW_WIDTH: u32 = 521;
const WINDOW_HEIGHT: u32 = 577;

fn main() -> Result<(), String> {
    let sdl_ctx = sdl2::init()?;
    let video_sys = sdl_ctx.video()?;

    let windows = video_sys.window("ChinessChess", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .allow_highdpi()
        .build().expect("could not initialize video subsystem");

    let mut game = Game::new(windows, sdl_ctx.event_pump()?);

    game.run()?;

    Ok(())
}
