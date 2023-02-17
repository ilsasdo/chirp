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

fn init_chip8() -> Chip8 {
    return Chip8 {
        ram: [0x0; 4096],
        display: [false; 64 * 32],
        pc: 0,
        i: 0,
        stack: Vec::new(),
        delay_timer: 0,
        sound_timer: 0,
        registers: [0x0; 16],
    };
}

fn load_rom(chip8: &mut Chip8, rom: String) -> io::Result<()> {
    let file = File::open(rom)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer).unwrap();

    // load to ram
    let mut i = 200;
    for value in buffer {
        chip8.ram[i] = value;
        i = i + 1;
    }

    chip8.pc = 200;
    Ok(())
}

fn execute(chip8: &mut Chip8) -> Result<(), String> {
    loop {
        // read the instruction pointed from the pc:
        let (first_nibble, second_nibble) = fetch_instruction(chip8);
        let (third_nibble, fourth_nibble) = fetch_instruction(chip8);

        match first_nibble {

            0 => {

            }

            _ => {
                return Err(String::from("Unknown instruction"))
            }
        }
    }
}

fn fetch_instruction(chip8: &mut Chip8) -> (u8, u8) {
    let instruction = chip8.ram[chip8.pc as usize];
    chip8.pc = chip8.pc + 1;

    let first_nibble = instruction << 4;
    let second_nibble = instruction >> 4;

    return (first_nibble, second_nibble);
}

fn main() {
    let chip8 = &mut init_chip8();
    load_rom(chip8, String::from("roms/IBM Logo.ch8"));

    execute(chip8);
}

#[cfg(test)]
mod tests {
    #[test]
    fn shift_left() {
        let a : u8 = 255;
        assert_eq!(15, (a >> 4));
    }
    #[test]
    fn shift_right() {
        let a : u8 = 255;
        assert_eq!(240, (a << 4));
    }
}
