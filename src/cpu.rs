#![allow(dead_code)] // Remove after testing

use std::fs;
use rand::{thread_rng, Rng};

const START_ADDRESS: usize = 0x200; // Where rom instructions start being stored
const FONT_START_ADDRESS: usize = 0x50; // Where rom instructions start being stored
const VIDEO_WIDTH: u8 = 64;
const VIDEO_HEIGHT: u8 = 32;

const FONT_SET: [u8; 80] = 
[
	0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8 {
    registers: [u8; 16], // Registers V0 to VF, where VF or registers[15] is the FLAG address
    memory: [u8; 4096], // Memmory, holds program instructions, font, and long-short term data
    index: u16, // Register used to hold memory addresses for use in OPCODES
    pc: u16, // Program Counter, holds the memory address for the next instruction to be executed
    stack: [u16; 16], // Stack that holds function calls
    sp: u8, // Stack Pointer, points to the most recent value in the Stack
    delay_timer: u8, // Simple timer, when set to > 0 it will decrement at cpu cycle rate
    sound_timer: u8, // Same logic as delay_timer, however emits a sound at 0
    keypad: [u8; 16], // Keypad with 0-16 mapped to specific keys, details on notes.txt
    video: [u32; VIDEO_WIDTH as usize * VIDEO_HEIGHT as usize], // 64 * 32 Video, rendered through SDL
    pub opcode: u16, // Current opcode
}

impl Chip8 {
    pub fn new() ->  Self {
        Chip8 {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 64*32],
            opcode: 0,
        }
    }

    pub fn get_display(&self) -> [u32; VIDEO_WIDTH as usize * VIDEO_HEIGHT as usize]{
        self.video
    }

    pub fn cycle(&mut self) {
        if self.pc == 4096 {
            self.pc = 0;
        }
        self.opcode = ((self.memory[self.pc as usize] as u16) << 8) | (self.memory[(self.pc + 1) as usize] as u16);
        let digit1 = (self.opcode & 0xF000) >> 12;
        let digit2 = (self.opcode & 0x0F00) >> 8;
        let digit3 = (self.opcode & 0x00F0) >> 4;
        let digit4 = self.opcode & 0x000F;

        
        self.pc += 2;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0xe, 0) => self.op_00e0(),
            (0, 0, 0xe, 0xe) => self.op_00ee(),
            // Unique first digit opcodes
            (1, ..) => self.op_1nnn(),
            (2, ..) => self.op_2nnn(),
            (3, ..) => self.op_3xkk(),
            (4, ..) => self.op_4xkk(),
            (5, ..) => self.op_5xy0(),
            (6, ..) => self.op_6xkk(),
            (7, ..) => self.op_7xkk(),
            (9, ..) => self.op_9xy0(),
            (0xA, ..) => self.op_annn(),
            (0xB, ..) => self.op_bnnn(),
            (0xC, ..) => self.op_cxkk(),
            (0xd, ..) => self.op_dxyn(),
            
            // 8 Table
            (8, .., 0) => self.op_8xy0(),
            (8, .., 1) => self.op_8xy1(),
            (8, .., 2) => self.op_8xy2(),
            (8, .., 3) => self.op_8xy3(),
            (8, .., 4) => self.op_8xy4(),
            (8, .., 5) => self.op_8xy5(),
            (8, .., 6) => self.op_8xy6(),
            (8, .., 7) => self.op_8xy7(),
            (8, .., 0xe) => self.op_8xye(),

            // E table
            (0xE, _, 0xA, 1) => self.op_exa1(),
            (0xE, _, 0x9, 0xE) => self.op_ex9e(),
            
            // F Table
            (0xF, _, 0, 7) => self.op_fx07(),
            (0xF, _, 0, 0xA) => self.op_fx0a(),
            (0xF, _, 1, 5) => self.op_fx15(),
            (0xF, _, 1, 8) => self.op_fx18(),
            (0xF, _, 1, 0xE) => self.op_fx1e(),
            (0xF, _, 2, 9) => self.op_fx29(),
            (0xF, _, 3, 3) => self.op_fx33(),
            (0xF, _, 5, 5) => self.op_fx55(),
            (0xF, _, 6, 5) => self.op_fx65(),

            _ => ()
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn update_key(&mut self, key: usize, value: usize) {
        self.keypad[key] = value as u8;
    }

    pub fn load_rom(&mut self, filename: &str) {
        let data: Vec<u8> = fs::read(filename).expect("Unable to read ROM");
        self.memory[START_ADDRESS..(START_ADDRESS+data.len())].copy_from_slice(&data);
        self.pc = START_ADDRESS as u16;
    }

    fn load_fonts(&mut self) {
        self.memory[FONT_START_ADDRESS..FONT_START_ADDRESS+FONT_SET.len()].copy_from_slice(&FONT_SET);
    }

    fn op_00e0(&mut self) {
        self.video.fill(0);   
    }

    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn op_1nnn(&mut self) {
        let addr: u16 = self.opcode & 0xFFF;
        self.pc = addr;
    }

    fn op_2nnn(&mut self) {
        let addr: u16 = self.opcode & 0xFFF;

        self.stack[self.sp as usize]  = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    fn op_3xkk(&mut self) {
        let vx = (self.opcode >> 8) & 0x00F;
        let kk = self.opcode & 0x00FF;
        if self.registers[vx as usize] as u16 == kk {
            self.pc += 2;
        }
    }
    
    fn op_4xkk(&mut self) {
        let vx = (self.opcode >> 8) & 0x00F;
        let kk = self.opcode & 0x00FF;
        if self.registers[vx as usize] as u16 != kk {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self) {
        let vx = (self.opcode >> 8) & 0x00F;
        let vy = (self.opcode >> 4) & 0x00F;

        if self.registers[vx as usize] == self.registers[vy as usize] {
            self.pc += 2
        }
    }

    fn op_6xkk(&mut self) {
        let vx = (self.opcode >> 8) & 0x00F;
        let kk = self.opcode & 0x00FF;
                
        self.registers[vx as usize] = kk as u8;
    }

    fn op_7xkk(&mut self) {
        let vx = (self.opcode >> 8) & 0x00F;
        let kk = self.opcode & 0x00FF;

        self.registers[vx as usize] = self.registers[vx as usize].wrapping_add(kk as u8);
    }

    fn op_8xy0(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let vy = ((self.opcode >> 4) & 0x00F) as usize;

        self.registers[vx] = self.registers[vy];
    }
    
    fn op_8xy1(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let vy = ((self.opcode >> 4) & 0x00F) as usize;

        self.registers[vx] |= self.registers[vy];
    }
    
    fn op_8xy2(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let vy = ((self.opcode >> 4) & 0x00F) as usize;

        self.registers[vx] &= self.registers[vy];
    }
    
    fn op_8xy3(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let vy = ((self.opcode >> 4) & 0x00F) as usize;

        self.registers[vx] ^= self.registers[vy];
    }

    fn op_8xy4(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let vy = ((self.opcode >> 4) & 0x00F) as usize;

        let sum: u16 = self.registers[vx].wrapping_add(self.registers[vy]).into();

        if sum > 255 {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }

        self.registers[vx] = (sum & 0xFF) as u8;
    }

    fn op_8xy5(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let vy = ((self.opcode >> 4) & 0x00F) as usize;
        
        if self.registers[vx] > self.registers[vy] {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }

        self.registers[vx] = self.registers[vx].wrapping_sub(self.registers[vy]);
    }


    fn op_8xy6(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let least = self.registers[vx] & 0x1;
        
        if least == 1 {
            self.registers[15] = 1
        } else {
            self.registers[15] = 0
        }

        self.registers[vx] >>= 1;
    }

    fn op_8xy7(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let vy = ((self.opcode >> 4) & 0x00F) as usize;
         
        if self.registers[vx] < self.registers[vy] {
            self.registers[15] = 1;
        } else {
            self.registers[15] = 0;
        }

        self.registers[vx] = self.registers[vy] - self.registers[vx];
    }
    
    fn op_8xye(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let most = (self.registers[vx] & 0x80) >> 7;
        
        if most == 1 {
            self.registers[15] = 1
        } else {
            self.registers[15] = 0
        }

        self.registers[vx] <<= 1;
    }

    fn op_9xy0(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let vy = ((self.opcode >> 4) & 0x00F) as usize;

        if self.registers[vx] != self.registers[vy] {
            self.pc += 2;
        }
    }

    fn op_annn(&mut self) {
        let addr = self.opcode & 0xFFF;
        self.index = addr;
    }

    fn op_bnnn(&mut self) {
        let addr = self.opcode & 0xFFF;
        self.pc = addr + self.registers[0] as u16;
    }

    fn op_cxkk(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let kk = self.opcode & 0x00FF;
        
        let mut rng = rand::thread_rng();
        let rand = rng.gen_range(0..=255);

        self.registers[vx] = (rand & kk) as u8;
    }

    fn op_dxyn(&mut self) {
        let vx = (self.opcode >> 8) & 0x00F;
        let vy = (self.opcode >> 4) & 0x00F;
        let height = self.opcode & 0x000F;
        
        // Setting cordinates of sprite as mod width/height in order to wrap
        let x_pos = self.registers[vx as usize] % VIDEO_WIDTH;
        let y_pos = self.registers[vy as usize] % VIDEO_HEIGHT;

        // Setting flag address to 0 to reset, will change to 1 in case of pixel collision
        self.registers[15] = 0;

        for row in 0..height {
            
            let sprite_byte = self.memory[(self.index + row) as usize];

            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0b10000000 >> col);
                let screen_pixel = &mut self.video[((y_pos as usize + row as usize) * VIDEO_WIDTH as usize + (x_pos as usize + col as usize)) % 2048];
                // on
                if sprite_pixel != 0 {
                    
                    if *screen_pixel == 0xFFFFFFF {
                        self.registers[15] = 1;
                        self.video[(y_pos as usize + row as usize) * VIDEO_WIDTH as usize + (x_pos as usize + col as usize)] = 0x0000000;
                    } else {
                        self.video[(y_pos as usize + row as usize) * VIDEO_WIDTH as usize + (x_pos as usize + col as usize)] = 0xFFFFFFF;
                    }
                    
                } 
            }
        }
    }

    
    fn op_ex9e(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let key = self.registers[vx];

        if self.keypad[key as usize] == 1 {
            self.pc += 2
        }
    }

    fn op_exa1(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let key = self.registers[vx];
        
        if self.keypad[key as usize] == 0 {
            self.pc += 2
        }
    }
    
    fn op_fx07(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        self.registers[vx] = self.delay_timer;
    }

    fn op_fx0a(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;

        let index = self.keypad.iter().position(|&x| x == 1);

        match index {
            Some(key) => self.registers[vx] = key as u8,
            None => self.pc += 2
        }
    }

    fn op_fx15(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        self.delay_timer = self.registers[vx];
    }

    fn op_fx18(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        self.sound_timer = self.registers[vx];
    }

    fn op_fx1e(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        self.index += self.registers[vx] as u16;
    }

    fn op_fx29(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let digit = self.registers[vx];

        self.index = 0x50 + (5 * digit) as u16;
    }

    fn op_fx33(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        let mut value = self.registers[vx];

        let ones = value % 10;
        value /= 10;
        let tens = value % 10;
        value /=10;
        let hundreds = value % 10;

        self.memory[self.index as usize] = hundreds;
        self.memory[(self.index + 1 ) as usize] = tens;
        self.memory[(self.index + 2 ) as usize] = ones;
    }
    
    fn op_fx55(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        for i in 0..vx+1 {
            self.memory[(self.index + i as u16) as usize] = self.registers[i];
        }
    }

    fn op_fx65(&mut self) {
        let vx = ((self.opcode >> 8) & 0x00F) as usize;
        for i in 0..vx+1 {
            self.registers[i]  = self.memory[(self.index + i as u16) as usize]
        }
    }
} 



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn loading_rom() {
        let mut cpu = Chip8::new();
        cpu.load_rom("IBM LOGO.ch8");
        let data = fs::read("IBM LOGO.ch8").expect("Cannot read rom in test");
        //println!("{:#0x?}", &cpu.memory[START_ADDRESS..START_ADDRESS+data.len()]);
        assert_eq!(cpu.memory[START_ADDRESS+data.len()-1],0xa)
    }

    #[test]
    fn loading_fonts() {
        let mut cpu = Chip8::new();
        cpu.load_fonts();
        assert_eq!(cpu.memory[0x50], 0xF0);
        assert_eq!(cpu.memory[FONT_START_ADDRESS+FONT_SET.len()-1], 0x80);

    }

    #[test]
    fn test_00e0() {
        let mut cpu = Chip8::new();
        cpu.video.fill(1);
        cpu.opcode = 0x00e0;
        cpu.op_00e0();
        assert_eq!([0;64*32], cpu.video);
    }

    #[test]
    fn test_1nnn() {
        let mut cpu = Chip8::new();
        cpu.opcode = 0x1228;
        cpu.op_1nnn();
        assert_eq!(cpu.pc, 0x228);
    }

    #[test]
    fn test_6xkk() {
        let mut cpu = Chip8::new();
        cpu.opcode = 0x6128;
        cpu.op_6xkk();
        assert_eq!(cpu.registers[1], 0x28)
    }
    
    #[test]
    fn test_7xkk() {
        let mut cpu = Chip8::new();
        cpu.registers[1] = 0x28;
        cpu.opcode = 0x7128;
        cpu.op_7xkk();
        assert_eq!(cpu.registers[1], 0x28 + 0x28)
    }
    
    #[test]
    fn test_annn() {
        let mut cpu = Chip8::new();
        cpu.opcode = 0xa22a;
        cpu.op_annn();
        assert_eq!(cpu.index, 0x22a)
    }

    #[test]
    fn test_dxyn() {
        let mut cpu = Chip8::new();
        cpu.opcode = 0xd25f;
        cpu.op_dxyn();
    }

    #[test]
    fn test_3xkk() {
        let mut cpu = Chip8::new();
        cpu.registers[0] = 0x0e;
        cpu.opcode = 0x300e;
        cpu.pc = 200;
        cpu.op_3xkk();
        assert_eq!(cpu.pc, 202);
    }

    #[test]
    fn test_fx0a() {
        let mut cpu = Chip8::new();
        cpu.keypad[12] = 1;
        cpu.opcode = 0xf00a;
        assert_eq!(cpu.registers[0], 0);
        cpu.op_fx0a();
        assert_eq!(cpu.registers[0], 12);
    }
}
