use std::fmt;

pub struct Instruction {
    first_byte: u8,
    second_byte: u8,
    pub first_nibble: u8,
    pub second_nibble: u8,
    pub third_nibble: u8,
    pub fourth_nibble: u8,
}

impl Instruction {
    pub fn new(first_byte: u8, second_byte: u8) -> Instruction {
        return Instruction {
            first_byte,
            second_byte,
            first_nibble: first_byte >> 4,
            second_nibble: first_byte & 15,
            third_nibble: second_byte >> 4,
            fourth_nibble: second_byte & 15,
        };
    }

    pub fn byte_sum_3(second_nibble: u8, third_nibble: u8, fourth_nibble: u8) -> u16 {
        return ((second_nibble as u16) << 8) + ((third_nibble as u16) << 4) + (fourth_nibble as u16);
    }

    pub fn byte_sum_2(third_nibble: u8, fourth_nibble: u8) -> u8 {
        return (third_nibble << 4) + fourth_nibble;
    }

    pub fn to_string(&self) -> String {
        return fmt::format(format_args!("{:#04x} {:#04x}", self.first_byte, self.second_byte));
    }
}


#[cfg(test)]
mod tests {
    use crate::instruction::Instruction;

    #[test]
    fn new_instruction() {
        let instruction = Instruction::new(0x12, 0xE3);
        assert_eq!(0x1, instruction.first_nibble);
        assert_eq!(0x2, instruction.second_nibble);
        assert_eq!(0xE, instruction.third_nibble);
        assert_eq!(0x3, instruction.fourth_nibble);
    }

    #[test]
    fn build_instruction_from_nibbles() {
        // assert_eq!(3, Instruction::byte_sum_3(0x0, 0x0, 0x3));
        // assert_eq!(0x23, Instruction::byte_sum_3(0x0, 0x2, 0x3));
        assert_eq!(228, Instruction::byte_sum_3((2 as u8), (2 as u8), (8 as u8)));
    }
}
