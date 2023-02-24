use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

use crate::instruction::Instruction;

pub mod instruction;

struct Chip8 {
    ram: [u8; 4096],
    display: [[bool; 32]; 64],
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
            display: [[false; 32]; 64],
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
        for x in 0..(self.display.len()) {
            for y in 0..(self.display[x].len()) {
                self.display[x][y] = false;
            }
        }
    }

    fn load_rom(&mut self, rom: String) -> io::Result<()> {
        let file = File::open(rom)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        // Read file into vector.
        reader.read_to_end(&mut buffer).unwrap();

        // load to ram
        let mut i = 0x200;
        for value in buffer {
            self.ram[i] = value;
            i = i + 1;
        }

        self.pc = 0x200;
        Ok(())
    }

    fn fetch_instruction(&mut self) -> Result<Instruction, String> {
        if (self.pc + 1) as usize >= self.ram.len() {
            return Err(String::from("Out of Memory!"));
        }

        let first_byte = self.ram[self.pc as usize];
        self.pc = self.pc + 1;
        let second_byte = self.ram[self.pc as usize];
        self.pc = self.pc + 1;

        let instruction = Instruction::new(first_byte, second_byte);
        return Ok(instruction);
    }

    fn subroutine_return(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    fn call_subroutine(&mut self, address: u16) {
        self.stack.push(self.pc);
        self.pc = address;
    }

    fn skip_if_equals(&mut self, a: u8, b: u8) {
        if a == b {
            self.pc += 2;
        }
    }

    fn skip_if_not_equals(&mut self, a: u8, b: u8) {
        if a != b {
            self.pc += 2;
        }
    }

    fn jump(&mut self, location: u16) {
        self.pc = location;
    }

    fn set_register(&mut self, register: u8, value: u8) {
        self.registers[usize::from(register)] = value;
    }

    fn set_index_register(&mut self, value: u16) {
        self.i = value;
    }

    fn add_to_register(&mut self, register: u8, value: u8) {
        self.registers[usize::from(register)] += value;
    }

    fn draw(&mut self, x_register: u8, y_register: u8, height: u8) {
        let x = (self.registers[usize::from(x_register)] % 64) as usize;
        let y = (self.registers[usize::from(y_register)] % 32) as usize;

        // set VF register to 0 until any pixel become 0
        self.registers[0xF] = 0;

        for h in 0..(height as usize) {
            let sprite_row = self.ram[usize::from(self.i + (h as u16))];
            let display_row = self.get_display_row(x, y + h);
            let new_row = sprite_row ^ display_row;
            let turned_off_pixels = display_row & sprite_row;
            if turned_off_pixels > 0 {
                self.registers[0xF] = 1;
            }
            self.set_display_row(x, y + h, new_row);
        }
    }

    fn set_display_row(&mut self, x: usize, y: usize, row: u8) {
        for bit in 0..8 {
            if (x + bit) < self.display.len() && y < self.display[(x + bit)].len() {
                self.display[(x + bit)][y] = (row & (1 << (7 - bit))) > 0;
            }
        }
    }

    fn get_display_row(&mut self, x: usize, y: usize) -> u8 {
        let mut result: u8 = 0;
        for bit in 0..8 {
            if (x + bit) < self.display.len() && y < self.display[(x + bit)].len() {
                result += (self.display[(x + bit)][y] as u8) << bit;
            }
        }
        return result;
    }

    fn display(display: [[bool; 32]; 64]) {
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

    fn execute(&mut self) -> Result<(), String> {
        loop {
            // read the instruction pointed from the pc:
            let instruction = self.fetch_instruction()?;

            // println!("DEBUG Instruction: {}, pc: {}", instruction.to_string(), self.pc);

            match instruction.first_nibble {
                0x0 => {
                    match instruction.second_nibble {
                        0x0 => {
                            match instruction.third_nibble {
                                0xE => {
                                    match instruction.fourth_nibble {
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

                        _ => {}
                    }
                }

                0x1 => {
                    self.jump(Instruction::byte_sum_3(instruction.second_nibble, instruction.third_nibble, instruction.fourth_nibble));
                }

                0x2 => {
                    self.call_subroutine(Instruction::byte_sum_3(instruction.second_nibble, instruction.third_nibble, instruction.fourth_nibble));
                }

                0x3 => {
                    self.skip_if_equals(self.registers[instruction.second_nibble as usize], Instruction::byte_sum_2(instruction.third_nibble, instruction.fourth_nibble));
                }

                0x4 => {
                    self.skip_if_not_equals(self.registers[instruction.second_nibble as usize], Instruction::byte_sum_2(instruction.third_nibble, instruction.fourth_nibble));
                }

                0x5 => {
                    self.skip_if_equals(self.registers[instruction.second_nibble as usize], self.registers[instruction.third_nibble as usize]);
                }

                0x6 => {
                    self.set_register(instruction.second_nibble, Instruction::byte_sum_2(instruction.third_nibble, instruction.fourth_nibble));
                }

                0x7 => {
                    self.add_to_register(instruction.second_nibble, Instruction::byte_sum_2(instruction.third_nibble, instruction.fourth_nibble));
                }

                0x9 => {
                    self.skip_if_not_equals(self.registers[instruction.second_nibble as usize], self.registers[instruction.third_nibble as usize]);
                }

                0xA => {
                    self.set_index_register(Instruction::byte_sum_3(instruction.second_nibble, instruction.third_nibble, instruction.fourth_nibble));
                }

                0xD => {
                    self.draw(instruction.second_nibble, instruction.third_nibble, instruction.fourth_nibble);
                    Chip8::display(self.display);
                }

                _ => {
                    return Err(format!("Unknown instruction: {}", instruction.first_nibble));
                }
            }
        }
    }
}

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_rom(String::from("roms/IBM Logo.ch8")).expect("File to exists.");
    chip8.execute().expect("OH NO!");
}
