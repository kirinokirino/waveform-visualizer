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
    next: bool,
}

impl App {
    pub fn new(window_size: UVec2) -> Self {
        let audio = Audio::new();
        Self {
            viewport: window_size,
            audio,
            next: true,
        }
    }
}

impl WindowHandler for App {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        if self.next {
            self.next = false;
            let buffer = self.audio.next();
            if let Some(buffer) = buffer {
                let points = 512_usize;
                let mut wave: Vec<Vec2> = vec![Vec2::ZERO; points];
                buffer
                    .first_channel
                    .iter()
                    .enumerate()
                    .map(|(i, sample)| {
                        (i, Vec2::new(
                            WINDOW_WIDTH as f32 / 512.0 * i as f32,
                            f32::from(*sample).mul_add((WINDOW_HEIGHT as f32 * 0.5) / f32::from(i16::MAX), WINDOW_HEIGHT as f32 / 2.0),
                        ))
                    })
                    .for_each(|(i, sample)| if let Some(point) = wave.get_mut(i) {
						*point = sample;
                    });
                graphics.clear_screen(Color::from_rgb(0.8, 0.8, 0.8));
                for pair in wave.as_slice().windows(2) {
                    let (from, to) = (pair[0], pair[1]);
                    graphics.draw_line(from, to, 2.0, Color::BLACK);
                }
            }
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
                VirtualKeyCode::Space => self.next = true,
                key => println!("Key: {key:?}, scancode: {scancode}"),
            }
        }
    }
}

struct Audio {
    counter: usize,
    buffers: Vec<Buffer>,
}

impl Audio {
    pub fn new() -> Self {
        let file_path = "0.ogg";
        println!("Opening file: {file_path}");
        let file = File::open(file_path).expect("Can't open file");
        let mut stream_reader =
            OggStreamReader::new(file).expect("Can't create oggstreamreader for file.");
        let sample_rate = stream_reader.ident_hdr.audio_sample_rate;
        println!("Sample rate: {sample_rate}");
        let audio_channels = stream_reader.ident_hdr.audio_channels;
        println!("There are {audio_channels} audio channels.");

        let mut buffers = Vec::new();
        let mut packet_idx = 0;
        let mut total_samples = 0;
        while let Some(samples) = stream_reader
            .read_dec_packet_itl()
            .expect("couldn't read ogg packet")
        {
            println!(
                "Decoded packet no {}, with {} samples.",
                packet_idx,
                samples.len()
            );
            packet_idx += 1;
            total_samples += samples.len();
            match audio_channels {
                1 => {
                    let mut buffer = Buffer::new();
                    buffer.mono_buffer(&samples);
                    buffers.push(buffer);
                }
                n => panic!("unsupported number of channels: {n}"),
            };
        }
        let length_in_seconds = total_samples as f32 / (f32::from(audio_channels) * sample_rate as f32);
        println!("The piece is {length_in_seconds} s long.");
        Self {
            buffers,
            counter: 0,
        }
    }
    pub fn next(&mut self) -> Option<&Buffer> {
        let result = if self.counter >= self.buffers.len() {
            None
        } else {
            self.buffers.get(self.counter)
        };
        self.counter += 1;
        result
    }
}

struct Buffer {
    pub first_channel: Vec<i16>,
}

impl Buffer {
    pub const fn new() -> Self {
        Self {
            first_channel: Vec::new(),
        }
    }

    fn mono_buffer(&mut self, samples: &[i16]) {
        self.first_channel = samples.to_vec();
    }
}
