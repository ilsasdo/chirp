use std::thread;
use simple_logger::SimpleLogger;
use crate::cpu::{Chip8};
use crate::sdl::{SdlDisplay, SdlInput};

pub mod cpu;
mod sdl;
mod basic;

fn main() {
    // SimpleLogger::new().init().unwrap();
    let (sdl_display, display_rx) = SdlDisplay::new();
    let (sdl_input, input_tx) = SdlInput::new();
    let keypad = sdl_input.keypad.clone();

    thread::spawn(move || {
        let mut chip8 = Chip8::new(&sdl_input, &sdl_display);
        chip8.load_rom_file(String::from("roms/Space Invaders [David Winter].ch8")).expect("File to exists.");
        chip8.execute().expect("OH NO!");
    });

    SdlDisplay::run(String::from("Chip8 Emulator"), 800, 600, keypad, display_rx);
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
