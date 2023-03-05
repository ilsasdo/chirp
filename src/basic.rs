use crate::cpu::{Display, Input};

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

    fn current_value(&self) -> Option<u8> {
        todo!()
    }
}
