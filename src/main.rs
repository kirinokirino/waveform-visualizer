
use macroquad::prelude::*;
use macroquad::rand::rand;

const WINDOW_WIDTH: u16 = 600;
const WINDOW_HEIGHT: u16 = 480;

fn window_conf() -> Conf {
    Conf {
        window_title: "FLOATING".to_string(),
        window_width: WINDOW_WIDTH.into(),
        window_height: WINDOW_HEIGHT.into(),
        fullscreen: false,
        ..Default::default()
    }
}


#[macroquad::main(window_conf)]
async fn main() {
	let ww = WINDOW_WIDTH as f32;
	let wh = WINDOW_HEIGHT as f32;
	let points = 36 * 4;
	
    loop {
    	println!("{}", get_fps());
		let mut wave: Vec<Vec2> = Vec::with_capacity(points);
		for point in 0..points {
			wave.push(Vec2::new((ww / points as f32 * point as f32), wh / 2.0 + (rand() % 100) as f32 - 50.0));
		}
        clear_background(LIGHTGRAY);

		for pair in wave.as_slice().windows(2) {
			let (from, to) = (pair[0], pair[1]);
			let (x1, y1, x2, y2) = (from.x, from.y, to.x, to.y);
			draw_line(x1, y1, x2, y2, 1.0, BLACK);
		}
		
        draw_text("HELLO", 20.0, 20.0, 30.0, DARKGRAY);

        std::thread::sleep(std::time::Duration::from_millis(16));
        next_frame().await
    }
}
