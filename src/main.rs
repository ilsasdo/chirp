use std::thread;
use crate::basic::{DummyInput};
use crate::cpu::{Chip8};
use crate::sdl::{SdlDisplay, SdlInput};

pub mod cpu;
mod sdl;
mod basic;

fn main() {
    let (sdl_display, display_rx) = SdlDisplay::new();
    let (sdl_input, input_tx) = SdlInput::new();

    let mut chip8 = Chip8::new(sdl_input, sdl_display);
    thread::spawn(move || {
        chip8.load_rom_file(String::from("roms/Delay Timer Test [Matthew Mikolay, 2010].ch8")).expect("File to exists.");
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
