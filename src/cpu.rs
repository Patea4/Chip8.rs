#![allow(dead_code)] // Remove after testing

use std::fs;

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

        
        println!("{:x?}", self.opcode);

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0xe, 0) => self.op_00e0(),
            (1, ..) => self.op_1nnn(),
            (6, ..) => self.op_6xkk(),
            (7, ..) => self.op_7xkk(),
            (0xA, ..) => self.op_annn(),
            (0xd, ..) => self.op_dxyn(),
            _ => ()
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
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

    fn op_1nnn(&mut self) {
        let addr: u16 = self.opcode & 0xFFF;
        self.pc = addr;
    }

    fn op_6xkk(&mut self) {
        let vx = (self.opcode >> 8) & 0x00F;
        let kk = self.opcode & 0x00FF;
                
        self.registers[vx as usize] = kk as u8;
    }

    fn op_7xkk(&mut self) {
        let vx = (self.opcode >> 8) & 0x00F;
        let kk = self.opcode & 0x00FF;

        self.registers[vx as usize] += kk as u8;
    }

    fn op_annn(&mut self) {
        let addr = self.opcode & 0xFFF;
        self.index = addr;
    }

    pub fn op_dxyn(&mut self) {
        println!("dasopidsada");
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
                let screen_pixel = &mut self.video[(y_pos as usize + row as usize) * VIDEO_WIDTH as usize + (x_pos as usize + col as usize)];
                println!("row: {row}, col: {col}, pixel: {}", sprite_pixel);
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
}
