use std::{io, thread};
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::debug;

use rand::Rng;

use crate::cpu::instruction::Instruction;

mod instruction;

const FONT_OFFSET: u8 = 50;
const MEM_OFFSET: u16 = 0x200;

pub trait Display {
    fn draw(&self, display: [[bool; 32]; 64]);
}

pub trait Input {
    fn wait(&self) -> u8;

    fn current_value(&self) -> Option<u8>;
}

pub struct Chip8<T: Input, D: Display> {
    ram: [u8; 4096],
    display: [[bool; 32]; 64],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: Arc<Mutex<u8>>,
    sound_timer: Arc<Mutex<u8>>,
    registers: [u8; 16],
    options: Chip8Options,
    input: T,
    display_output: D,
}

struct Chip8Options {
    super_chip: bool,
}

impl<T: Input, D: Display> Chip8<T, D> {
    pub fn new(input: T, display: D) -> Chip8<T, D> {
        let mut chip8 = Chip8 {
            ram: [0x0; 4096],
            display: [[false; 32]; 64],
            pc: 0,
            i: 0,
            stack: Vec::new(),
            delay_timer: Arc::new(Mutex::new(0)),
            sound_timer: Arc::new(Mutex::new(0)),
            registers: [0x0; 16],
            options: Chip8Options {
                super_chip: false
            },
            input,
            display_output: display,
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
            chip8.ram[i + (FONT_OFFSET as usize)] = *e;
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

    pub(crate) fn load_rom_file(&mut self, rom: String) -> io::Result<()> {
        let file = File::open(rom)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        // Read file into vector.
        reader.read_to_end(&mut buffer).unwrap();

        // load to ram
        self.load_rom_bytes(buffer);

        Ok(())
    }

    pub(crate) fn load_rom_bytes(&mut self, buffer: Vec<u8>) {
        let mut i = MEM_OFFSET as usize;
        for value in buffer {
            self.ram[i] = value;
            i = i + 1;
        }

        self.pc = MEM_OFFSET;
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
        debug!("JUMP {}", location);
        self.pc = location;
    }

    fn jump_with_offset(&mut self, location: u16) {
        debug!("JUMP_OFFSET REG[0]={} + {}", self.register_get_value(0x0), location);
        self.jump((self.register_get_value(0x0) as u16) + location);
    }

    fn register_set_value(&mut self, register: u8, value: u8) {
        debug!("REG_SET_VALUE REG[{}]={}", register, value);
        self.registers[usize::from(register)] = value;
    }

    fn set_index_register(&mut self, value: u16) {
        debug!("INDEX_SET_VALUE {}", value);
        self.i = value;
    }

    fn register_add_value(&mut self, register: u8, value: u8) {
        let a: u8 = self.registers[register as usize];
        let b: u8 = value;
        let (sum, _overflows) = a.overflowing_add(b);
        self.registers[usize::from(register)] = sum;
    }

    fn register_set(&mut self, register_a: u8, register_b: u8) {
        self.registers[register_a as usize] = self.registers[register_b as usize];
    }

    fn register_get_value(&self, register: u8) -> u8 {
        return self.registers[(register as usize) & 0xF];
    }

    fn register_or(&mut self, register_a: u8, register_b: u8) {
        self.registers[register_a as usize] = self.registers[register_a as usize] | self.registers[register_b as usize];
    }

    fn register_and(&mut self, register_a: u8, register_b: u8) {
        self.registers[register_a as usize] = self.registers[register_a as usize] & self.registers[register_b as usize];
    }

    fn register_xor(&mut self, register_a: u8, register_b: u8) {
        self.registers[register_a as usize] = self.registers[register_a as usize] ^ self.registers[register_b as usize];
    }

    fn register_add(&mut self, register_a: u8, register_b: u8) {
        let a = self.register_get_value(register_a);
        let b = self.register_get_value(register_b);
        let (result, overflows) = a.overflowing_add(b);
        self.register_set_value(0xF, if overflows { 1 } else { 0 });
        self.register_set_value(register_a, result);
    }

    fn register_subtract(&mut self, register_a: u8, register_b: u8) {
        let a = self.register_get_value(register_a);
        let b = self.register_get_value(register_b);
        let (result, overflows) = a.overflowing_sub(b);
        self.register_set_value(0xF, if overflows { 0 } else { 1 });
        self.register_set_value(register_a, result);
    }

    fn register_left_shift(&mut self, register_a: u8, register_b: u8) {
        if self.options.super_chip {
            self.register_set(register_a, register_b);
        }

        let a = self.register_get_value(register_a);
        self.register_set_value(register_a, self.register_get_value(register_a) << 1);
        if a >= 128 {
            self.register_set_value(0xF, 1);
        } else {
            self.register_set_value(0xF, 0);
        }
    }

    fn register_right_shift(&mut self, register_a: u8, register_b: u8) {
        if self.options.super_chip {
            self.register_set(register_a, register_b);
        }

        let a = self.register_get_value(register_a);
        self.register_set_value(register_a, self.register_get_value(register_a) >> 1);
        if a % 2 == 0 {
            self.register_set_value(0xF, 0);
        } else {
            self.register_set_value(0xF, 1);
        }
    }

    fn skip_if_key_pressed_is(&mut self, register: u8) {
        let value = self.register_get_value(register);
        match self.input.current_value() {
            Some(key) => {
                if key == value {
                    self.pc += 2;
                }
            }
            None => {
                // do nothing
            }
        }
    }

    fn skip_if_key_pressed_is_not(&mut self, register: u8) {
        let value = self.register_get_value(register);
        match self.input.current_value() {
            None => {
                // do nothing
            }
            Some(key) => {
                if key != value {
                    self.pc += 2;
                }
            }
        }
    }

    fn draw(&mut self, x_register: u8, y_register: u8, height: u8) {
        let x = (self.register_get_value(x_register) % 64) as usize;
        let y = (self.register_get_value(y_register) % 32) as usize;

        // set VF register to 0 until any pixel become 0
        self.register_set_value(0xF, 0);

        for h in 0..(height as usize) {
            let sprite_row = self.ram[usize::from(self.i + (h as u16))];
            let display_row = self.get_display_row(x, y + h);
            let (new_row, collision) = self.draw_sprite_row(sprite_row, display_row);
            if collision {
                self.register_set_value(0xF, 1);
            }
            self.set_display_row(x, y + h, new_row);
        }
    }

    fn draw_sprite_row(&self, sprite_row: u8, display_row: u8) -> (u8, bool) {
        let new_row = sprite_row ^ display_row;
        let turned_off_pixels = display_row & sprite_row;
        return (new_row, turned_off_pixels > 0);
    }

    fn get_display_row(&mut self, x: usize, y: usize) -> u8 {
        let mut row = [false; 8];
        for i in 0..(row.len()) {
            if (x + i) < self.display.len() {
                row[i] = self.display[(x + i)][y];
            } else {
                row[i] = false;
            }
        }
        return u8::from_bit_array(row);
    }

    fn set_display_row(&mut self, x: usize, y: usize, row: u8) {
        let bits = row.to_bit_array();
        for bit in 0..(bits.len()) {
            if (x + bit) < self.display.len() {
                self.display[(x + bit)][y] = bits[bit];
            }
        }
    }

    fn register_set_value_to_delay_timer(&mut self, register: u8) {
        let c = *(Arc::clone(&self.delay_timer).lock().unwrap());
        self.register_set_value(register, c);
    }

    fn random(&mut self, register: u8, value: u8) {
        let mut rng = rand::thread_rng();
        self.register_set_value(register, rng.gen::<u8>() & value);
    }

    fn set_delay_timer(&mut self, register: u8) {
        let mut dt = self.delay_timer.lock().unwrap();
        *dt = self.register_get_value(register);
    }

    fn set_sound_timer(&mut self, register: u8) {
        let mut st = self.sound_timer.lock().unwrap();
        *st = self.register_get_value(register);
    }

    fn add_to_index(&mut self, register: u8) {
        let value = self.register_get_value(register) as u16;
        let (i, overflow) = self.i.overflowing_add(value);
        self.i = i;
        if overflow {
            self.register_set_value(0xF, 1)
        } else {
            // TODO: it's unclear if I have to reset to zero in case of non-overflow.
            // self.register_set_value(0xF, 0)
        }
    }

    fn set_index_register_to_font(&mut self, register_font: u8) {
        let f = self.register_get_value(register_font) & 0xF;
        debug!("INDEX_SET_FONT {}", f);
        self.set_index_register(((f * 5) + FONT_OFFSET) as u16);
    }

    fn decimal_conversion(&mut self, register: u8) {
        let value = self.register_get_value(register);
        let units = value % 10;
        let tens = (value / 10) % 10;
        let hundreds = value / 100;
        self.ram[self.i as usize] = hundreds;
        self.ram[self.i as usize + 1] = tens;
        self.ram[self.i as usize + 2] = units;
    }

    fn ram_store(&mut self, value: u8) {
        for x in 0..=(value as u16) {
            self.ram[(self.i + x) as usize] = self.register_get_value(x as u8);
        }
    }

    fn ram_load(&mut self, value: u8) {
        for x in 0..=(value as u16) {
            self.register_set_value(x as u8, self.ram[(self.i + x) as usize]);
        }
    }

    fn get_key(&mut self, register: u8) {
        let key = self.input.wait();
        self.register_set_value(register, key);
    }

    pub fn execute(&mut self) -> Result<(), String> {
        timer(Arc::clone(&self.delay_timer));
        // timer(Arc::clone(&self.sound_timer));

        loop {
            thread::sleep(Duration::from_millis(1));
            // read the instruction pointed from the pc:
            let instruction = self.fetch_instruction()?;

            match instruction.first_nibble {
                0x0 => {
                    if instruction.byte_sum_3() == 0x0E0 {
                        self.clear_screen();
                        self.display_output.draw(self.display);
                    } else if instruction.byte_sum_3() == 0x0EE {
                        self.subroutine_return();
                    }
                }

                0x1 => {
                    self.jump(instruction.byte_sum_3());
                }

                0x2 => {
                    self.call_subroutine(instruction.byte_sum_3());
                }

                0x3 => {
                    self.skip_if_equals(self.register_get_value(instruction.second_nibble), instruction.byte_sum_2());
                }

                0x4 => {
                    self.skip_if_not_equals(self.register_get_value(instruction.second_nibble), instruction.byte_sum_2());
                }

                0x5 => {
                    self.skip_if_equals(self.register_get_value(instruction.second_nibble), self.register_get_value(instruction.third_nibble));
                }

                0x6 => {
                    self.register_set_value(instruction.second_nibble, instruction.byte_sum_2());
                }

                0x7 => {
                    self.register_add_value(instruction.second_nibble, instruction.byte_sum_2());
                }

                0x8 => {
                    match instruction.fourth_nibble {
                        0x0 => {
                            self.register_set(instruction.second_nibble, instruction.third_nibble);
                        }
                        0x1 => {
                            self.register_or(instruction.second_nibble, instruction.third_nibble);
                        }
                        0x2 => {
                            self.register_and(instruction.second_nibble, instruction.third_nibble);
                        }
                        0x3 => {
                            self.register_xor(instruction.second_nibble, instruction.third_nibble);
                        }
                        0x4 => {
                            self.register_add(instruction.second_nibble, instruction.third_nibble);
                        }
                        0x5 => {
                            self.register_subtract(instruction.second_nibble, instruction.third_nibble);
                        }
                        0x6 => {
                            self.register_right_shift(instruction.second_nibble, instruction.third_nibble);
                        }
                        0x7 => {
                            self.register_subtract(instruction.third_nibble, instruction.second_nibble);
                        }
                        0xE => {
                            self.register_left_shift(instruction.second_nibble, instruction.third_nibble);
                        }
                        _ => {}
                    }
                }

                0x9 => {
                    self.skip_if_not_equals(self.register_get_value(instruction.second_nibble), self.register_get_value(instruction.third_nibble));
                }

                0xA => {
                    self.set_index_register(instruction.byte_sum_3());
                }

                0xB => {
                    self.jump_with_offset(instruction.byte_sum_3());
                }

                0xC => {
                    self.random(instruction.second_nibble, instruction.byte_sum_2());
                }

                0xD => {
                    self.draw(instruction.second_nibble, instruction.third_nibble, instruction.fourth_nibble);
                    self.display_output.draw(self.display);
                }

                0xE => {
                    if instruction.byte_sum_2() == 0x9E {
                        self.skip_if_key_pressed_is(instruction.second_nibble)
                    } else if instruction.byte_sum_2() == 0xA1 {
                        self.skip_if_key_pressed_is_not(instruction.second_nibble)
                    }
                }

                0xF => {
                    if instruction.byte_sum_2() == 0x07 {
                        self.register_set_value_to_delay_timer(instruction.second_nibble);
                    } else if instruction.byte_sum_2() == 0x15 {
                        self.set_delay_timer(instruction.second_nibble);
                    } else if instruction.byte_sum_2() == 0x18 {
                        self.set_sound_timer(instruction.second_nibble);
                    } else if instruction.byte_sum_2() == 0x1E {
                        self.add_to_index(instruction.second_nibble);
                    } else if instruction.byte_sum_2() == 0x0A {
                        self.get_key(instruction.second_nibble);
                    } else if instruction.byte_sum_2() == 0x29 {
                        self.set_index_register_to_font(instruction.second_nibble);
                    } else if instruction.byte_sum_2() == 0x33 {
                        self.decimal_conversion(instruction.second_nibble);
                    } else if instruction.byte_sum_2() == 0x55 {
                        self.ram_store(instruction.second_nibble);
                    } else if instruction.byte_sum_2() == 0x65 {
                        self.ram_load(instruction.second_nibble);
                    }
                }

                _ => {
                    return Err(format!("Unknown instruction: {}", instruction.first_nibble));
                }
            }
        }
    }
}

fn timer(delay: Arc<Mutex<u8>>) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(1000 / 60));
            let mut d = delay.lock().unwrap();
            if *d > 0 {
                *d -= 1;
            }
        }
    });
}

trait BitArray<T> {
    fn to_bit_array(&self) -> [bool; 8];

    fn from_bit_array(bits: [bool; 8]) -> T;
}

impl BitArray<u8> for u8 {
    fn to_bit_array(&self) -> [bool; 8] {
        let mut result: [bool; 8] = [false; 8];
        for bit in 0..result.len() {
            result[result.len() - 1 - bit] = ((*self) & (1 << bit)) > 0;
        }
        return result;
    }

    fn from_bit_array(bits: [bool; 8]) -> u8 {
        let mut result: u8 = 0;
        for bit in 0..bits.len() {
            if bits[bits.len() - 1 - bit] {
                result += 1 << bit
            }
        }
        return result;
    }
}

#[cfg(test)]
mod test {
    use crate::basic::DummyInput;
    use crate::cpu::{BitArray, Chip8, Display};

    struct FakeDisplay {}

    impl Display for FakeDisplay {
        fn draw(&self, _display: [[bool; 32]; 64]) {}
    }

    #[test]
    fn draw_sprite_row() {
        let input = DummyInput {};
        let display = FakeDisplay {};
        let cpu = Chip8::new(input, display);
        let (row, collision) = cpu.draw_sprite_row(0x1, 0x0);
        assert_eq!(row, 0x1);
        assert_eq!(collision, false);

        let (row, collision) = cpu.draw_sprite_row(0x2, 0x6);
        assert_eq!(row, 0x4);
        assert_eq!(collision, true);
    }

    #[test]
    fn to_bit_array() {
        let v: u8 = 0x85; // 10000101
        let a = v.to_bit_array();

        assert_eq!(a, [true, false, false, false, false, true, false, true]);
    }

    #[test]
    fn from_bit_array() {
        let v: u8 = u8::from_bit_array([true, false, false, false, false, true, false, true]); // 0x85
        assert_eq!(v, 0x85);
    }

    #[test]
    fn get_display_row() {
        let mut cpu = Chip8::new(DummyInput {}, FakeDisplay {});
        // 10000101 -> 0x85 -> 113
        cpu.display[0][0] = true;
        cpu.display[1][0] = false;
        cpu.display[2][0] = false;
        cpu.display[3][0] = false;
        cpu.display[4][0] = false;
        cpu.display[5][0] = true;
        cpu.display[6][0] = false;
        cpu.display[7][0] = true;

        // 10000000 -> 128
        cpu.display[60][0] = true;
        cpu.display[61][0] = false;
        cpu.display[62][0] = false;
        cpu.display[63][0] = false;
        // cpu.display[4][0] = false;
        // cpu.display[5][0] = true;
        // cpu.display[6][0] = false;
        // cpu.display[7][0] = true;
        let row = cpu.get_display_row(60, 0);
        assert_eq!(128, row);
    }
}
