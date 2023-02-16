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

use lewton::inside_ogg::OggStreamReader;

use std::fs::File;

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
    audio: Audio,
    segment: i32,
}

impl App {
    pub fn new(window_size: UVec2) -> Self {
        let audio = Audio::new();
        Self {
            viewport: window_size,
            audio,
            segment: 0,
        }
    }
}

impl WindowHandler for App {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        let segment = if self.segment > 0 {
            self.segment as usize
        } else {
            0
        };
        let segment_size = self.audio.meta.sample_rate as usize / 4;
        let slice = segment_size * segment;
        let wave: Vec<Vec2> = self
            .audio
            .buffer
            .iter()
            .skip(slice)
            .take(segment_size)
            .enumerate()
            .map(|(i, sample)| {
                Vec2::new(
                    WINDOW_WIDTH as f32 / 512.0 * i as f32,
                    f32::from(*sample).mul_add(
                        (WINDOW_HEIGHT as f32 * 0.5) / f32::from(i16::MAX),
                        WINDOW_HEIGHT as f32 / 2.0,
                    ),
                )
            })
            .collect();

        graphics.clear_screen(Color::from_rgb(0.8, 0.8, 0.8));
        for pair in wave.as_slice().windows(2) {
            let (from, to) = (pair[0], pair[1]);
            graphics.draw_line(from, to, 2.0, Color::BLACK);
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
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
                VirtualKeyCode::Space => self.segment += 1,
                key => println!("Key: {key:?}, scancode: {scancode}"),
            }
        }
    }
}

struct Audio {
    buffer: Vec<i16>,
    meta: Meta,
}

impl Audio {
    pub fn new() -> Self {
        let file_path = "0.ogg";
        println!("Opening file: {file_path}");
        let file = File::open(file_path).expect("Can't open file");
        let mut stream_reader =
            OggStreamReader::new(file).expect("Can't create oggstreamreader for file.");
        let sample_rate = stream_reader.ident_hdr.audio_sample_rate;
        let audio_channels = stream_reader.ident_hdr.audio_channels;
        let mut buffer = Vec::new();
        while let Some(samples) = stream_reader
            .read_dec_packet_itl()
            .expect("couldn't read ogg packet")
        {
            buffer.extend_from_slice(&samples);
        }
        let length_in_seconds =
            buffer.len() as f32 / (f32::from(audio_channels) * sample_rate as f32);

        let meta = Meta {
            length: length_in_seconds,
            channels: audio_channels,
            sample_rate,
        };
        meta.print();
        Self { buffer, meta }
    }
}

struct Meta {
    length: f32,
    channels: u8,
    sample_rate: u32,
}

impl Meta {
    pub fn print(&self) {
        println!("The piece is {}s long.", self.length);
        println!("Sample rate: {}", self.sample_rate);
        println!("There are {} audio channels.", self.channels);
    }
}
