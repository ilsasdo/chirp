use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

struct Chip8 {
    ram: [u8; 4096],
    display: [bool; 64 * 32],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],
}

impl Chip8 {
    fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            ram: [0x0; 4096],
            display: [false; 64 * 32],
            pc: 0,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: [0x0; 16],
        };

        let fonts: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70,   // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0,   // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0,   // 3
            0x90, 0x90, 0xF0, 0x10, 0x10,   // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0,   // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0,   // 6
            0xF0, 0x10, 0x20, 0x40, 0x40,   // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0,   // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0,   // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90,   // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0,   // B
            0xF0, 0x80, 0x80, 0x80, 0xF0,   // C
            0xE0, 0x90, 0x90, 0x90, 0xE0,   // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0,   // E
            0xF0, 0x80, 0xF0, 0x80, 0x80];  // F

        for (i, e) in fonts.iter().enumerate() {
            chip8.ram[i + 50] = *e;
        }

        return chip8;
    }

    fn clear_screen(&mut self) {
        for i in 0..(self.display.len())  {
            self.display[i] = false;
        }
    }

    fn load_rom(&mut self, rom: String) -> io::Result<()> {
        let file = File::open(rom)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        // Read file into vector.
        reader.read_to_end(&mut buffer).unwrap();

        // load to ram
        let mut i = 200;
        for value in buffer {
            self.ram[i] = value;
            i = i + 1;
        }

        self.pc = 200;
        Ok(())
    }

    fn subroutine_return(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    fn jump(&mut self, location: u16) {
        self.pc = location;
    }

    fn execute(&mut self) -> Result<(), String> {
        loop {
            // read the instruction pointed from the pc:
            let (first_nibble, second_nibble) = self.fetch_instruction();
            let (third_nibble, fourth_nibble) = self.fetch_instruction();

            match first_nibble {
                0x0 => {
                    match second_nibble {
                        0x0 => {
                            match third_nibble {
                                0xE => {
                                    match fourth_nibble {
                                        0x0 => {
                                            self.clear_screen()
                                        }
                                        0xE => {
                                            self.subroutine_return()
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }

                        _ => {

                        }
                    }
                }

                0x1 => {
                    self.jump((u16::from(second_nibble) << 8) + (u16::from(third_nibble) << 4) + u16::from(fourth_nibble));
                }

                0x6 => {

                }

                _ => {
                    return Err(String::from("Unknown instruction"));
                }
            }
        }
    }

    fn fetch_instruction(&mut self) -> (u8, u8) {
        let instruction = self.ram[self.pc as usize];
        self.pc = self.pc + 1;

        let first_nibble = instruction << 4;
        let second_nibble = instruction >> 4;

        return (first_nibble, second_nibble);
    }
}


fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_rom(String::from("roms/IBM Logo.ch8"));

    chip8.execute();
}

#[cfg(test)]
mod tests {
    #[test]
    fn shift_left() {
        let a: u8 = 255;
        assert_eq!(15, (a >> 4));
    }

    #[test]
    fn shift_right() {
        let a: u8 = 255;
        assert_eq!(240, (a << 4));
    }
}
