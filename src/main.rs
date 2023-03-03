use std::thread;
use crate::basic::{DummyInput};
use crate::cpu::{Chip8};
use crate::sdl::SdlDisplay;

pub mod cpu;
mod sdl;
mod basic;

fn main() {
    let (sdl, input_tx, display_rx) = SdlDisplay::new();
    let mut chip8 = Chip8::new(DummyInput {}, sdl);
    thread::spawn(move || {
        chip8.load_rom(String::from("roms/Clock Program [Bill Fisher, 1981].ch8")).expect("File to exists.");
        chip8.execute().expect("OH NO!");
    });

    SdlDisplay::run(String::from("Chip8 Emulator"), 800, 600, input_tx, display_rx);
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_overflow() {
        let a: u8 = 244u8;
        let b: u8 = 244u8;

        if ((a as u16) + (b as u16)) > 255u16 {
            assert!(true)
        } else {
            assert!(false)
        }
    }
}
