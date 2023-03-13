use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use crate::cpu::{Display, Input};

pub struct SdlDisplay {
    display_tx: SyncSender<[[bool; 32]; 64]>,
}

pub struct SdlInput {
    pub keypad: Arc<Mutex<u16>>
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
            keypad: Arc::new(Mutex::new(0x0))
        }, input_tx);
    }
}

fn press_key(key:u16, keypad: &u16) -> u16 {
    return keypad | (1 << key);
}

impl SdlDisplay {
    pub fn new() -> (SdlDisplay, Receiver<[[bool; 32]; 64]>) {
        let (display_tx, display_rx): (SyncSender<[[bool; 32]; 64]>, Receiver<[[bool; 32]; 64]>) = mpsc::sync_channel(1);

        return (SdlDisplay {
            display_tx
        }, display_rx);
    }

    pub fn run(title: String, width: u32, height: u32, keypad: Arc<Mutex<u16>>, rx: Receiver<[[bool; 32]; 64]>) {
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
                    _ => {}
                }
            }

            let mut keys : u16 = 0;
            for pressed_key in event_pump.keyboard_state().pressed_scancodes() {
                match pressed_key {
                    Scancode::Num1 => keys = press_key(0x1, &keys),
                    Scancode::Num2 => keys = press_key(0x2, &keys),
                    Scancode::Num3 => keys = press_key(0x3, &keys),
                    Scancode::Num4 => keys = press_key(0xC, &keys),
                    Scancode::Q => keys = press_key(0x4, &keys),
                    Scancode::W => keys = press_key(0x5, &keys),
                    Scancode::E => keys = press_key(0x6, &keys),
                    Scancode::R => keys = press_key(0xD, &keys),
                    Scancode::A => keys = press_key(0x7, &keys),
                    Scancode::S => keys = press_key(0x8, &keys),
                    Scancode::D => keys = press_key(0x9, &keys),
                    Scancode::F => keys = press_key(0xE, &keys),
                    Scancode::Z => keys = press_key(0xA, &keys),
                    Scancode::X => keys = press_key(0x0, &keys),
                    Scancode::C => keys = press_key(0xB, &keys),
                    Scancode::V => keys = press_key(0xF, &keys),

                    _ => {}
                }
            }
            *(keypad.lock().unwrap()) = keys;

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
    fn wait_for_key(&self) -> u8 {
        return 0x0;
    }

    fn is_key_pressed(&self, key: u8) -> bool {
        let pressed_keys = *self.keypad.lock().unwrap();
        println!("pressed_keys: {pressed_keys} {key}");
        return (pressed_keys & (1 << key)) > 0;
    }
}
