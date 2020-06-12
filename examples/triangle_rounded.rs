//extern crate pixel_engine_gl as engine;
use pixel_engine_gl as engine;
fn main() -> Result<(), String> {
    let mut engine = engine::Engine::new("Triangle".to_string(), (550, 550, 1), &game_logic);
    engine.run();
    Ok(())
}

fn game_logic(game: &mut engine::Engine) /*-> Result<(), String> */
{
    fn dist(p1: (i32, i32), p2: (i32, i32)) -> f64 {
        return (((p2.0 - p1.0).pow(2) + (p2.1 - p1.1).pow(2)) as f64).sqrt();
    };
    let base = 0.95;
    let green = (
        (game.size.0 as f32 * (1f32 - base)) as i32,
        ((game.size.1 as f32 * base) - 100f32 * base) as i32,
    );
    let blue = (
        (game.size.0 as f32 * base) as i32,
        ((game.size.1 as f32 * base) - 100f32 * base) as i32,
    );
    let red: (i32, i32) = (
        (game.size.0 as f32 * 0.50f32) as i32,
        ((game.size.1 as f32 * base) - 100f32 * base) as i32
            - (dist(green, blue).powi(2) - (dist(green, blue) / 2f64).powi(2)).sqrt() as i32,
    );
    let max_dist = dist(red, blue);

    println!("Done calculations");
    for x in 0..(game.size.0) {
        for y in 0..(game.size.1) {
            let point = (x as i32, y as i32);
            if dist(point, red) < max_dist
                && dist(point, blue) < max_dist
                && dist(point, green) < max_dist
            {
                game.screen.draw(
                    x,
                    y,
                    engine::Color::new(
                        ((1f64 - dist((x as i32, y as i32), red) / dist(red, blue)) * 255f64) as u8,
                        ((1f64 - dist((x as i32, y as i32), green) / dist(red, blue)) * 255f64)
                            as u8,
                        ((1f64 - dist((x as i32, y as i32), blue) / dist(red, blue)) * 255f64)
                            as u8,
                    ),
                )
            } else {
                game.screen.draw(x, y, engine::Color::BLACK)
            }
        }
    }
    println!("mainloop");
    while game.new_frame() {}
}
