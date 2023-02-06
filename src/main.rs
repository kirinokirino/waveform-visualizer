#![warn(clippy::nursery, clippy::pedantic)]
#![allow(clippy::cast_precision_loss)]
use speedy2d::{
    color::Color,
    dimen::{UVec2, Vec2},
    window::{
        KeyScancode, VirtualKeyCode, WindowCreationOptions, WindowHandler, WindowHelper,
        WindowPosition, WindowSize,
    },
    Graphics2D, Window,
};

const WINDOW_WIDTH: u32 = 600;
const WINDOW_HEIGHT: u32 = 480;

fn main() {
    let window_size = UVec2::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let window_pixels = WindowSize::PhysicalPixels(window_size);
    let window = Window::new_with_options(
        "FLOATING",
        WindowCreationOptions::new_windowed(window_pixels, Some(WindowPosition::Center))
            .with_decorations(false)
            .with_transparent(true),
    )
    .expect("Wasn't able to create a window!");
    window.run_loop(App::new(window_size));
}

struct App {
    viewport: UVec2,
}

impl App {
    pub const fn new(window_size: UVec2) -> Self {
        Self {
            viewport: window_size,
        }
    }
}

impl WindowHandler for App {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        let points = (WINDOW_WIDTH / 12) as usize;
        let mut wave: Vec<Vec2> = Vec::with_capacity(points);
        for point in 0..points {
            wave.push(Vec2::new(
                (WINDOW_WIDTH as f32) / points as f32 * point as f32,
                fastrand::f32().mul_add(100.0, (WINDOW_HEIGHT as f32) / 2.0) - 50.0,
            ));
        }

        graphics.clear_screen(Color::from_rgb(0.8, 0.8, 0.8));
        for pair in wave.as_slice().windows(2) {
            let (from, to) = (pair[0], pair[1]);
            graphics.draw_line(from, to, 2.0, Color::BLACK);
        }

        std::thread::sleep(std::time::Duration::from_millis(60));
        helper.request_redraw();
    }

    fn on_resize(&mut self, _helper: &mut WindowHelper<()>, size_pixels: UVec2) {
        self.viewport = size_pixels;
    }

    fn on_key_down(
        &mut self,
        helper: &mut WindowHelper<()>,
        virtual_key_code: Option<VirtualKeyCode>,
        scancode: KeyScancode,
    ) {
        if let Some(key_code) = virtual_key_code {
            match key_code {
                VirtualKeyCode::Escape => helper.terminate_loop(),
                key => println!("Key: {key:?}, scancode: {scancode}"),
            }
        }
    }
}
