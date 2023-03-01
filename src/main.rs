use crate::cpu::{Chip8, Display, Input};

pub mod cpu;

pub struct ConsoleDisplay {}

impl Display for ConsoleDisplay {
    fn draw(&self, display: [[bool; 32]; 64]) {
        for y in 0..display[0].len() {
            for x in 0..display.len() {
                let pixel = display[x][y];
                if pixel {
                    print!("#");
                } else {
                    print!("_");
                }
            }
            println!("_");
        }
    }
}

pub struct DummyInput {}

impl Input for DummyInput {
    fn wait(&self) -> u8 {
        todo!()
    }

    fn current_value(&self) -> u8 {
        todo!()
    }
}

fn main() {
    let mut chip8 = Chip8::new(DummyInput {}, ConsoleDisplay {});
    chip8.load_rom(String::from("roms/IBM Logo.ch8")).expect("File to exists.");
    chip8.execute().expect("OH NO!");
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
