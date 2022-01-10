use std::fs;

const FONTSET: [u8; 80] = [
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

// (Almost) every CHIP-8 program starts on 0x200 memory address.
const PROGRAM_START: u16 = 0x200;

pub struct CPU {
    opcode: u16,
    memory: [u8; 4096],
    
    // CHIP-8's registers. Even though there are 16 of them, only 0-E can be used by a program.
    V: [u8; 16],

    I: u16,
    pc: u16,

    pub gfx: [u8; 2048],

    // Timers
    delay_timer: u8,
    sound_timer: u8,

    stack: [u16; 16],
    sp: u16, 

    keypad: [u8; 16],

    pub draw_flag: bool,
}

impl CPU {
    pub fn new(rom_path: &str) -> CPU {
        let mut memory = [0; 4096];
        let rom_data = fs::read(rom_path).unwrap();

        for i in 0..80 {
            memory[i] = FONTSET[i];
        }

        for i in 0..rom_data.len() {
            memory[i + PROGRAM_START as usize] = rom_data[i];
        }
        
        CPU {
            opcode: 0,
            memory: memory,
            V: [0; 16],
            I: 0,
            pc: PROGRAM_START,
            gfx: [0; 2048],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keypad: [0; 16],
            draw_flag: false,
        }
    }

    pub fn cycle(&mut self) {
        self.opcode = ((self.memory[self.pc as usize] as u16) << 8u16 | self.memory[self.pc as usize + 1] as u16) as u16;

        // TODO: Consider using funcion pointers instead.
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x000F {
                0x0000 => {
                    for i in 0..2048 {
                        self.gfx[i] = 0;
                    }

                    self.draw_flag = false;
                    self.pc += 2;
                },
                0x000E => {

                }
                _ => {
                    println!("Unknown opcode detected: {:#x}. Ignoring.", self.opcode);
                }
            }
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }
            0x6000 => {
                self.V[(self.opcode & 0x0F00) as usize >> 8] = (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            0xA000 => {
                self.I = self.opcode & 0x0FFF;
                self.pc += 2;
            }
            0xD000 => {
                let x = self.V[(self.opcode & 0x0F00) as usize >> 8] as u16;
                let y = self.V[(self.opcode & 0x00F0) as usize >> 4] as u16;
                let height = self.opcode & 0x000F;

                self.V[0xf] = 0;
                for y_line in 0..height {
                    let pixel = self.memory[(self.I + y_line) as usize] as u16;
                    for x_line in 0..8 {
                        if (pixel & (0x80 >> x)) != 0 {
                            if self.gfx[(x + x_line + ((y + y_line) * 64)) as usize] == 1{
                                self.V[0xf] = 1;
                            }
                            self.gfx[(x + x_line + ((y + y_line) * 64)) as usize] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            }
            _ => {
                println!("Unknown opcode detected: {:#x}. Ignoring.", self.opcode);
            }
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}