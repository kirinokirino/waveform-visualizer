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

use std::collections::VecDeque;
use std::fs::File;
use std::thread::sleep;
use std::time::{Duration, Instant};

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
    let audio = Audio::new();
    window.run_loop(App::new(window_size, audio));
}

struct App {
    viewport: UVec2,
    audio: Audio,
}

impl App {
    pub const fn new(window_size: UVec2, audio: Audio) -> Self {
        Self {
            viewport: window_size,
            audio,
        }
    }
}

impl WindowHandler for App {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        let buffer = self.audio.next();
        if let Some(buffer) = buffer {
            let points = 512 as usize;
            let mut wave: Vec<Vec2> = Vec::with_capacity(points);
            buffer
                .first_channel
                .into_iter()
                .enumerate()
                .map(|(i, sample)| {
                    Vec2::new(
                        WINDOW_WIDTH as f32 / 512.0 * i as f32,
                        (sample as f32) * ((WINDOW_HEIGHT as f32 * 0.5) / i16::MAX as f32)
                            + WINDOW_HEIGHT as f32 / 2.0,
                    )
                })
                .for_each(|sample| wave.push(sample));
            graphics.clear_screen(Color::from_rgb(0.8, 0.8, 0.8));
            for pair in wave.as_slice().windows(2) {
                let (from, to) = (pair[0], pair[1]);
                graphics.draw_line(from, to, 2.0, Color::BLACK);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
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

struct Audio {
    counter: usize,
    buffers: Vec<Buffer>,
}

impl Audio {
    pub fn new() -> Self {
        let mut buffers = Vec::new();
        let file_path = "0.ogg";
        println!("Opening file: {}", file_path);
        let file = File::open(file_path).expect("Can't open file");

        // Prepare the reading
        let mut stream_reader =
            OggStreamReader::new(file).expect("Can't create oggstreamreader for file.");

        let sample_rate = stream_reader.ident_hdr.audio_sample_rate as i32;

        println!(
            "There are {} audio channels.",
            stream_reader.ident_hdr.audio_channels
        );
        println!("Sample rate: {}", stream_reader.ident_hdr.audio_sample_rate);
        // Now the fun starts..
        let mut n = 0;
        let mut len_play = 0.0;
        let mut start_play_time = None;
        let start_decode_time = Instant::now();
        let sample_channels = stream_reader.ident_hdr.audio_channels as f32
            * stream_reader.ident_hdr.audio_sample_rate as f32;
        while let Some(pck_samples) = stream_reader
            .read_dec_packet_itl()
            .expect("couldn't read ogg packet")
        {
            println!(
                "Decoded packet no {}, with {} samples.",
                n,
                pck_samples.len()
            );
            n += 1;
            len_play += pck_samples.len() as f32 / sample_channels;
            let buf = match stream_reader.ident_hdr.audio_channels {
                1 => {
                    let mut buffer = Buffer::new();
                    buffer.mono_buffer(&pck_samples);
                    buffers.push(buffer);
                }
                2 => {
                    panic!();
                }
                n => panic!("unsupported number of channels: {}", n),
            };

            // If we are faster than realtime, we can already start playing now.
            if n == 100 {
                let cur = Instant::now();
                if cur - start_decode_time < Duration::from_millis((len_play * 1000.0) as u64) {
                    start_play_time = Some(cur);
                }
            }
        }
        let total_duration = Duration::from_millis((len_play * 1000.0) as u64);
        let sleep_duration = total_duration
            - match start_play_time {
                None => Duration::from_millis(0),
                Some(t) => Instant::now() - t,
            };
        println!("The piece is {} s long.", len_play);
        Self {
            buffers,
            counter: 0,
        }
    }
    pub fn next(&mut self) -> Option<Buffer> {
        if self.buffers.len() > 0 {
            self.buffers.pop()
        } else {
            None
        }
    }
}

struct Buffer {
    pub first_channel: VecDeque<i16>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            first_channel: VecDeque::new(),
        }
    }

    fn mono_buffer(&mut self, samples: &[i16]) {
        println!("Mono buffer len {}", samples.len());
        if samples.len() != 512 {
            return;
        }
        println!("{:?}", samples[0]);
        self.first_channel = VecDeque::from_iter(samples.into_iter().copied());
    }
}
