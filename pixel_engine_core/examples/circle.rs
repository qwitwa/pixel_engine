extern crate pixel_engine_core as engine;
use engine::traits::*;
fn main() {
    let mut game = engine::Engine::new("Circle".to_owned(), (51, 51, 10));
    game.run(|game: &mut engine::Engine| {
        game.screen.clear(engine::Color::WHITE);
        game.screen.draw_circle(25, 25, 25, engine::Color::BLACK);
        game.screen.fill_circle(25, 25, 12, engine::Color::BLUE);
        Ok(true)
    });
}