use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use crate::cpu::{Display, Input};

pub struct SdlDisplay {
    display_tx: Sender<[[bool; 32]; 64]>,
}

pub struct SdlInput {
    input_rx: Receiver<u8>,
}

fn to_sdl_rect(display: [[bool; 32]; 64], width: u32, height: u32) -> Vec<Rect> {
    let mut rects: Vec<Rect> = Vec::new();
    let ratio_x: u32 = width / display.len() as u32;
    let ratio_y: u32 = height / display[0].len() as u32;

    for y in 0..(display[0].len() as u32) {
        for x in 0..(display.len() as u32) {
            let pixel = display[(x as usize)][(y as usize)];
            if pixel {
                rects.push(Rect::new((x * ratio_x) as i32, (y * ratio_y) as i32, ratio_x, ratio_y));
            }
        }
    }

    return rects;
}

impl SdlInput {
    pub fn new() -> (SdlInput, Sender<u8>) {
        let (input_tx, input_rx): (Sender<u8>, Receiver<u8>) = mpsc::channel();
        return (SdlInput {
            input_rx
        }, input_tx);
    }
}

impl SdlDisplay {
    pub fn new() -> (SdlDisplay, Receiver<[[bool; 32]; 64]>) {
        let (display_tx, display_rx): (Sender<[[bool; 32]; 64]>, Receiver<[[bool; 32]; 64]>) = mpsc::channel();

        return (SdlDisplay {
            display_tx
        }, display_rx);
    }

    pub fn run(title: String, width: u32, height: u32, tx: Sender<u8>, rx: Receiver<[[bool; 32]; 64]>) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window(title.as_str(), width, height)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            let result = rx.try_recv();
            if result.is_ok() {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
                canvas.clear();

                canvas.set_draw_color(Color::RGB(255, 255, 255));
                let rects = to_sdl_rect(result.unwrap(), width, height);
                for r in rects {
                    canvas.draw_rect(r).unwrap();
                    canvas.fill_rect(r).unwrap();
                }

                canvas.present();
            }

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    }
                    Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                        tx.send(0xA).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                        tx.send(0xB).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::Num4), .. } => {
                        tx.send(0xC).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                        tx.send(0xD).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                        tx.send(0xE).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::V), .. } => {
                        tx.send(0xF).unwrap();
                    }

                    Event::KeyDown { keycode: Some(Keycode::X), .. } => {
                        tx.send(0x0).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                        tx.send(0x1).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                        tx.send(0x2).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::Num3), .. } => {
                        tx.send(0x3).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                        tx.send(0x4).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                        tx.send(0x5).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                        tx.send(0x6).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                        tx.send(0x7).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                        tx.send(0x8).unwrap();
                    }
                    Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                        tx.send(0x9).unwrap();
                    }
                    _ => {
                    }
                }
            }

            thread::sleep(Duration::from_millis(1_000 / 60));
        }
    }
}

impl Display for SdlDisplay {
    fn draw(&self, display: [[bool; 32]; 64]) {
        self.display_tx.send(display).unwrap();
    }
}

impl Input for SdlInput {
    fn wait(&self) -> u8 {
        return self.input_rx.recv().unwrap();
    }

    fn current_value(&self) -> Option<u8> {
        return match self.input_rx.try_recv() {
            Ok(key) => {
                println!("current_key={}", key);
                Some(key)
            }
            Err(_) => {
                None
            }
        };
    }
}
