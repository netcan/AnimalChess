use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::image::{self, LoadTexture, InitFlag};
use std::time::Duration;

const WINDOW_WIDTH: u32 = 521;
const WINDOW_HEIGHT: u32 = 577;

fn render(canvas: &mut WindowCanvas, texture: &Texture) -> Result<(), String> {
    canvas.clear();
    canvas.copy(texture, None, None)?;
    canvas.present();
    Ok(())
}

fn main() -> Result<(), String> {
    let sdl_ctx = sdl2::init()?;
    let video_sys = sdl_ctx.video()?;
    let windows = video_sys.window("ChinessChess", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build().expect("could not initialize video subsystem");
    let mut canvas = windows.into_canvas()
        .build().expect("could not make a canvas");
    let _image_ctx = image::init(InitFlag::JPG)?;
    let texture_creator = canvas.texture_creator();
    let board = texture_creator.load_texture("assets/board.jpg")?;


    let mut event_pump = sdl_ctx.event_pump()?;
    'running: loop {
        // handle event
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // update
        render(&mut canvas, &board)?;

        // time management
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
