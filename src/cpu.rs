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

        if (4096 - PROGRAM_START) < rom_data.len() as u16 {
            eprintln!("ERROR: The ROM is too big for the system to handle!");
            std::process::exit(1);
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
        //println!("Current opcode: {:#x}", self.opcode);
        // TODO: Consider using funcion pointers instead.
        let v_x = (self.opcode & 0x0F00) as usize >> 8;
        let v_y = (self.opcode & 0x00F0) as usize >> 4;
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x000F {
                0x0000 => {
                    self.gfx = [0; 2048];

                    self.draw_flag = true;
                    self.pc += 2;
                },
                0x000E => {
                    self.stack[self.sp as usize] = 0;
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode detected: {:#x}. Ignoring.", self.opcode);
                }
            }
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            }
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }
            0x3000 => {
                if self.V[v_x]== (self.opcode & 0x00FF) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0x4000 => {
                if self.V[v_x]!= (self.opcode & 0x00FF) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0x5000 => {
                if self.V[v_x]== self.V[v_y]{
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0x6000 => {
                self.V[v_x]= (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            0x7000 => {
                self.V[v_x]= self.V[(self.opcode & 0x0F00) as usize >> 8].wrapping_add((self.opcode & 0x00FF) as u8);
                self.pc += 2;
            }
            0x8000 => match self.opcode & 0x000F {
                0x0000 => {
                    self.V[v_x]= self.V[v_y];
                    self.pc += 2;
                }
                0x0001 => {
                    self.V[v_x]|= self.V[v_y];
                    self.pc += 2;
                }
                0x0002 => {
                    self.V[v_x]&= self.V[v_y];
                    self.pc += 2;
                }
                0x0003 => {
                    self.V[v_x]^= self.V[v_y];
                    self.pc += 2;
                }
                0x0004 => {
                    if self.V[v_y]> (0xFF - self.V[v_x]) {
                        self.V[0xF] = 1;
                    } else {
                        self.V[0xF] = 0;
                    }
                    self.V[v_x]= self.V[v_x].wrapping_add(self.V[v_y]);
                    self.pc += 2;
                }
                0x0005 => {
                    if self.V[v_y]> self.V[v_x]{
                        self.V[0xF] = 0;
                    } else {
                        self.V[0xF] = 1;
                    }
                    self.V[v_x] = self.V[v_x].wrapping_sub(self.V[v_y]);
                    self.pc += 2;
                }
                0x0006 => {
                    self.V[0xF] = self.V[v_x]& 0x1;
                    self.V[v_x]>>= 1;
                    self.pc += 2;
                }
                0x0007 => {
                    if self.V[v_y]> (0xFF - self.V[(self.opcode & 0x0F00) as usize]) {
                        self.V[0xF] = 1;
                    } else {
                        self.V[0xF] = 0;
                    }
                    self.V[v_x]= self.V[v_y]- self.V[v_x];
                    self.pc += 2;
                }
                0x000E => {
                    self.V[0xF] = self.V[v_x]& 7;
                    self.V[v_x]<<= 1;
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode detected: {:#x}. Ignoring.", self.opcode);
                }
            }
            0x9000 => {
                if self.V[v_x]!= self.V[v_y]{
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0xA000 => {
                self.I = self.opcode & 0x0FFF;
                self.pc += 2;
            }
            0xB000 => {
                self.pc = (self.opcode & 0x0FFF) + self.V[0] as u16;
            }
            0xD000 => {
                let x = self.V[v_x]as u16;
                let y = self.V[v_y]as u16;
                let height = self.opcode & 0x000F;

                self.V[0xf] = 0;
                for y_line in 0..height {
                    let pixel = self.memory[(self.I + y_line) as usize] as u16;
                    
                    for x_line in 0..8 {
                        if (pixel & (0x80 >> x_line)) != 0 {
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
            0xF000 => match self.opcode & 0x00FF {
                0x0007 => {
                    self.V[v_x]= self.delay_timer;
                    self.pc += 2;
                }
                0x0029 => {
                    self.I = self.V[v_x]as u16 * 0x5;
                    self.pc += 2;
                }
                0x0033 => {
                    self.memory[self.I as usize] = self.V[v_x]/ 100;
                    self.memory[self.I as usize+1] = (self.V[v_x]/ 10) % 10;
                    self.memory[self.I as usize+2] = (self.V[v_x]% 100) % 10;
                    self.pc += 2;
                }
                0x0055 => {
                    for offset in 0..v_x + 1{
                        self.memory[self.I as usize + offset] = self.V[offset];
                    }
                    //self.I += ((self.opcode & 0x0F00) >> 8) + 1;
                    self.pc += 2;
                }
                0x0065 => {
                    for offset in 0..v_x + 1 {
                        self.V[offset] = self.memory[self.I as usize + offset];
                    }
                    //self.I += ((self.opcode & 0x0F00) >> 8) + 1;
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode detected: {:#x}. Ignoring", self.opcode);
                }
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