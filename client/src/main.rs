use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use macroquad::ui::*;

#[macroquad::main("Trivia Client")]
async fn main() {
    loop {
        clear_background(BLACK);

        test();
        next_frame().await
    }
}

fn test() {
    let size: Vec2 = screen_size().into();
}
