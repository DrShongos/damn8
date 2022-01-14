use std::fs;
use rand::{thread_rng, prelude::ThreadRng, Rng};

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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

// (Almost) every CHIP-8 program starts on 0x200 memory address.
const PROGRAM_START: u16 = 0x200;

pub struct CPU {
    opcode: u16,
    memory: [u8; 4096],

    rng: ThreadRng,

    // CHIP-8's registers. Even though there are 16 of them, only 0-E can be used by a program.
    register: [u8; 16],

    index_register: u16,
    pc: u16,

    pub gfx: [u8; 2048],

    // Timers
    delay_timer: u8,
    sound_timer: u8,

    stack: [u16; 16],
    sp: u16,

    pub keypad: [u8; 16],

    pub draw_flag: bool,
    pub input_flag: bool,
    pub last_key: u8,
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
            rng: thread_rng(),
            register: [0; 16],
            index_register: 0,
            pc: PROGRAM_START,
            gfx: [0; 2048],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keypad: [0; 16],
            draw_flag: false,
            input_flag: false,
            last_key: 0,
        }
    }

    pub fn cycle(&mut self) {
        self.opcode = ((self.memory[self.pc as usize] as u16) << 8u16
            | self.memory[self.pc as usize + 1] as u16) as u16;

        //println!("OPCODE: {:#X}", self.opcode);
        // TODO: Consider using funcion pointers instead.
        let v_x = (self.opcode & 0x0F00) as usize >> 8;
        let v_y = (self.opcode & 0x00F0) as usize >> 4;

        let nnn = self.opcode & 0x0FFF;
        let nn = self.opcode & 0x00FF;
        let n = self.opcode & 0x000F;

        match self.opcode & 0xF000 {
            0x0000 => match n {
                0x0000 => {
                    self.gfx = [0; 2048];

                    self.draw_flag = true;
                    self.pc += 2;
                }
                0x000E => {
                    self.stack[self.sp as usize] = 0;
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode detected: {:#x}. Ignoring.", self.opcode);
                }
            },
            0x1000 => {
                self.pc = nnn;
            }
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            0x3000 => {
                if self.register[v_x] == (nn) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0x4000 => {
                if self.register[v_x] != (nn) as u8 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0x5000 => {
                if self.register[v_x] == self.register[v_y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0x6000 => {
                self.register[v_x] = (nn) as u8;
                self.pc += 2;
            }
            0x7000 => {
                self.register[v_x] = self.register[v_x].wrapping_add((nn) as u8);
                self.pc += 2;
            }
            0x8000 => match n {
                0x0000 => {
                    self.register[v_x] = self.register[v_y];
                    self.pc += 2;
                }
                0x0001 => {
                    self.register[v_x] |= self.register[v_y];
                    self.pc += 2;
                }
                0x0002 => {
                    self.register[v_x] &= self.register[v_y];
                    self.pc += 2;
                }
                0x0003 => {
                    self.register[v_x] ^= self.register[v_y];
                    self.pc += 2;
                }
                0x0004 => {
                    if self.register[v_y] > (0xFF - self.register[v_x]) {
                        self.register[0xF] = 1;
                    } else {
                        self.register[0xF] = 0;
                    }
                    self.register[v_x] = self.register[v_x].wrapping_add(self.register[v_y]);
                    self.pc += 2;
                }
                0x0005 => {
                    if self.register[v_y] > self.register[v_x] {
                        self.register[0xF] = 0;
                    } else {
                        self.register[0xF] = 1;
                    }
                    self.register[v_x] = self.register[v_x].wrapping_sub(self.register[v_y]);
                    self.pc += 2;
                }
                0x0006 => {
                    self.register[0xF] = self.register[v_x] & 0x1;
                    self.register[v_x] >>= 1;
                    self.pc += 2;
                }
                0x0007 => {
                    if self.register[v_y] > (0xFF - self.register[(self.opcode & 0x0F00) as usize]) {
                        self.register[0xF] = 1;
                    } else {
                        self.register[0xF] = 0;
                    }
                    self.register[v_x] = self.register[v_y] - self.register[v_x];
                    self.pc += 2;
                }
                0x000E => {
                    self.register[0xF] = self.register[v_x] & 7;
                    self.register[v_x] <<= 1;
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode detected: {:#x}. Ignoring.", self.opcode);
                }
            },
            0x9000 => {
                if self.register[v_x] != self.register[v_y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0xA000 => {
                self.index_register = nnn;
                self.pc += 2;
            }
            0xB000 => {
                self.pc = (nnn) + self.register[0] as u16;
            }
            0xC000 => {
                let number: u8 = self.rng.gen();
                self.register[v_x] = number & nn as u8;
                self.pc += 2;
            }
            0xD000 => {
                let x = self.register[v_x] as u16;
                let y = self.register[v_y] as u16;
                let height = n;

                self.register[0xf] = 0;
                for y_line in 0..height {
                    let pixel = self.memory[(self.index_register + y_line) as usize] as u16;

                    for x_line in 0..8 {
                        let x_pos = (x + x_line) % 64;
                        let y_pos = (y + y_line) % 32;

                        let pixel_pos = x_pos + (y_pos * 64);

                        if (pixel & (0x80 >> x_line)) != 0 {
                            if self.gfx[pixel_pos as usize] == 1 {
                                self.register[0xf] = 1;
                            }
                            self.gfx[pixel_pos as usize] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            }
            0xE000 => match nn {
                0x009E => {
                    if self.keypad[self.register[v_x] as usize] != 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                }
                0x00A1 => {
                    if self.keypad[self.register[v_x] as usize] == 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                }
                _ => {
                    println!("Unknown opcode detected: {:#x}. Ignoring", self.opcode);
                }
            }
            0xF000 => match nn {
                0x0007 => {
                    self.register[v_x] = self.delay_timer;
                    self.pc += 2;
                }
                0x000A => {
                    if !self.input_flag {
                        return;
                    }

                    //println!("KEY PRESSED: {:#x}", self.last_key);
                    self.register[v_x] = self.last_key;
                    self.pc += 2;   
                }
                0x0015 => {
                    self.delay_timer = self.register[v_x];
                    self.pc += 2;
                }
                0x0018 => {
                    self.sound_timer = self.register[v_x];
                    self.pc += 2;
                }
                0x0029 => {
                    self.index_register = self.register[v_x] as u16 * 0x5;
                    self.pc += 2;
                }
                0x0033 => {
                    self.memory[self.index_register as usize] = self.register[v_x] / 100;
                    self.memory[self.index_register as usize + 1] = (self.register[v_x] / 10) % 10;
                    self.memory[self.index_register as usize + 2] = (self.register[v_x] % 100) % 10;
                    self.pc += 2;
                }
                0x0055 => {
                    for offset in 0..v_x + 1 {
                        self.memory[self.index_register as usize + offset] = self.register[offset];
                    }
                    self.index_register += ((self.opcode & 0x0F00) >> 8) + 1;
                    self.pc += 2;
                }
                0x0065 => {
                    for offset in 0..v_x + 1 {
                        self.register[offset] = self.memory[self.index_register as usize + offset];
                    }
                    self.index_register += ((self.opcode & 0x0F00) >> 8) + 1;
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode detected: {:#x}. Ignoring", self.opcode);
                }
            },
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
