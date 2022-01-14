use rand::{prelude::ThreadRng, thread_rng, Rng};
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

// (Almost) every CHIP-8 program starts on 0x200 memory address.
const PROGRAM_START: u16 = 0x200;

const OPCODE_TABLE: [fn(&mut CPU); 17] = [
    CPU::ret_or_clear,
    CPU::jump,
    CPU::call,
    CPU::if_equal,
    CPU::if_nequal,
    CPU::if_equal_xy,
    CPU::assign,
    CPU::add,
    CPU::arithmetic,
    CPU::if_nequal_xy,
    CPU::index_assign,
    CPU::offset_jump,
    CPU::rng,
    CPU::draw_sprite,
    CPU::is_pressed_or_not,
    CPU::os,
    CPU::null,
];

const ARITHMETIC_TABLE: [fn(&mut CPU); 16] = [
    CPU::assign_xy,
    CPU::or,
    CPU::and,
    CPU::xor,
    CPU::add_carry,
    CPU::substract,
    CPU::right_shift,
    CPU::y_sub_x,
    CPU::null,
    CPU::null,
    CPU::null,
    CPU::null,
    CPU::null,
    CPU::left_shift,
    CPU::null,
    CPU::null,
];

// The OS table is initialized in a different way because it's too large to initialize manually.
static mut OS_TABLE: [fn(&mut CPU); 102] = [CPU::null; 102];

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
        println!("{}", rom_data.len());

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

    pub fn initialize_os_table() {
        unsafe {
            OS_TABLE[7] = CPU::get_delay;
            OS_TABLE[10] = CPU::wait_for_input;
            OS_TABLE[21] = CPU::set_delay;
            OS_TABLE[24] = CPU::set_sound;
            OS_TABLE[41] = CPU::get_sprite;
            OS_TABLE[51] = CPU::get_bcd;
            OS_TABLE[85] = CPU::reg_dump;
            OS_TABLE[101] = CPU::reg_load;
        }
    }

    pub fn cycle(&mut self) {
        self.opcode = ((self.memory[self.pc as usize] as u16) << 8u16
            | self.memory[self.pc as usize + 1] as u16) as u16;

        println!("{:#X}", self.opcode);

        OPCODE_TABLE[(self.opcode & 0xF000) as usize >> 12](self);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    /////////////////////////////////////////////////////////////////////////////
    ///                         MAIN OPCODES                                 ///
    ///////////////////////////////////////////////////////////////////////////
    fn null(&mut self) {}

    fn ret_or_clear(&mut self) {
        if self.opcode & 0x000F == 0x0000 {
            self.gfx = [0; 2048];
            self.draw_flag = true;
        } else if self.opcode & 0x000F == 0x000E {
            self.stack[self.sp as usize] = 0;
            self.sp -= 1;
            self.pc = self.stack[self.sp as usize];
        }
        self.pc += 2;
    }

    fn jump(&mut self) {
        self.pc = self.opcode & 0x0FFF;
    }

    fn call(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFF;
    }

    fn if_equal(&mut self) {
        if self.register[(self.opcode & 0x0F00) as usize >> 8] == (self.opcode & 0x00FF) as u8 {
            self.pc += 4;
        } else {
            self.pc += 2;
        };
    }

    fn if_nequal(&mut self) {
        if self.register[(self.opcode & 0x0F00) as usize >> 8] != (self.opcode & 0x00FF) as u8 {
            self.pc += 4;
        } else {
            self.pc += 2;
        };
    }

    fn if_equal_xy(&mut self) {
        if self.register[(self.opcode & 0x0F00) as usize >> 8]
            == self.register[(self.opcode & 0x0F0) as usize >> 4]
        {
            self.pc += 4;
        } else {
            self.pc += 2;
        };
    }

    fn if_nequal_xy(&mut self) {
        if self.register[(self.opcode & 0x0F00) as usize >> 8]
            != self.register[(self.opcode & 0x0F0) as usize >> 4]
        {
            self.pc += 4;
        } else {
            self.pc += 2;
        };
    }

    fn assign(&mut self) {
        self.register[(self.opcode & 0x0F00) as usize >> 8] = (self.opcode & 0x00FF) as u8;
        self.pc += 2;
    }

    fn add(&mut self) {
        self.register[(self.opcode & 0x0F00) as usize >> 8] = self.register
            [(self.opcode & 0x0F00) as usize >> 8]
            .wrapping_add((self.opcode & 0x00FF) as u8);
        self.pc += 2;
    }

    fn arithmetic(&mut self) {
        ARITHMETIC_TABLE[(self.opcode & 0x000F) as usize](self);
        self.pc += 2;
    }

    fn index_assign(&mut self) {
        self.index_register = self.opcode & 0x0FFF;
        self.pc += 2;
    }

    fn offset_jump(&mut self) {
        self.pc = self.opcode & 0x0FFF + self.register[0] as u16;
    }

    fn rng(&mut self) {
        let number: u8 = self.rng.gen();
        self.register[(self.opcode & 0x0F00) as usize >> 8] = number & (self.opcode & 0x00FF) as u8;
        self.pc += 2;
    }

    fn draw_sprite(&mut self) {
        let x = self.register[(self.opcode & 0x0F00) as usize >> 8] as u16;
        let y = self.register[(self.opcode & 0x0F0) as usize >> 4] as u16;
        let height = self.opcode & 0x000F;

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

    fn os(&mut self) {
        unsafe {
            OS_TABLE[(self.opcode & 0x00FF) as usize](self);
        }
    }

    fn is_pressed_or_not(&mut self) {
        if self.opcode & 0x00FF == 0x009E {
            if self.keypad[self.register[(self.opcode & 0x0F00) as usize >> 8] as usize] != 0 {
                self.pc += 4;
            } else {
                self.pc += 2;
            }
        } else if self.opcode & 0x00FF == 0x00A1 {
            if self.keypad[self.register[(self.opcode & 0x0F00) as usize >> 8] as usize] == 0 {
                self.pc += 4;
            } else {
                self.pc += 2;
            }
        }
    }

    /////////////////////////////////////////////////////////////////////////////
    ///                         ARITHMETIC OPCODES                           ///
    ///////////////////////////////////////////////////////////////////////////

    fn assign_xy(&mut self) {
        self.register[(self.opcode & 0x0F00) as usize >> 8] =
            self.register[(self.opcode & 0x0F0) as usize >> 4];
    }

    fn or(&mut self) {
        self.register[(self.opcode & 0x0F00) as usize >> 8] |=
            self.register[(self.opcode & 0x0F0) as usize >> 4];
    }

    fn and(&mut self) {
        self.register[(self.opcode & 0x0F00) as usize >> 8] &=
            self.register[(self.opcode & 0x0F0) as usize >> 4];
    }

    fn xor(&mut self) {
        self.register[(self.opcode & 0x0F00) as usize >> 8] ^=
            self.register[(self.opcode & 0x0F0) as usize >> 4];
    }

    fn add_carry(&mut self) {
        if self.register[(self.opcode & 0x0F0) as usize >> 4]
            > (0xFF - self.register[(self.opcode & 0x0F00) as usize >> 8])
        {
            self.register[0xF] = 1;
        } else {
            self.register[0xF] = 0;
        }
        self.register[(self.opcode & 0x0F00) as usize >> 8] = self.register
            [(self.opcode & 0x0F00) as usize >> 8]
            .wrapping_add(self.register[(self.opcode & 0x0F0) as usize >> 4]);
    }

    fn substract(&mut self) {
        if self.register[(self.opcode & 0x0F0) as usize >> 4]
            > (0xFF - self.register[(self.opcode & 0x0F00) as usize >> 8])
        {
            self.register[0xF] = 0;
        } else {
            self.register[0xF] = 1;
        }
        self.register[(self.opcode & 0x0F00) as usize >> 8] = self.register
            [(self.opcode & 0x0F00) as usize >> 8]
            .wrapping_sub(self.register[(self.opcode & 0x0F0) as usize >> 4]);
    }

    fn right_shift(&mut self) {
        self.register[0xF] = self.register[(self.opcode & 0x0F00) as usize >> 8] & 0x1;
        self.register[(self.opcode & 0x0F00) as usize >> 8] >>= 1;
    }

    fn y_sub_x(&mut self) {
        if self.register[(self.opcode & 0x0F0) as usize >> 4]
            > (0xFF - self.register[(self.opcode & 0x0F00) as usize])
        {
            self.register[0xF] = 1;
        } else {
            self.register[0xF] = 0;
        }
        self.register[(self.opcode & 0x0F00) as usize >> 8] = self.register
            [(self.opcode & 0x0F0) as usize >> 4]
            - self.register[(self.opcode & 0x0F00) as usize >> 8];
    }

    fn left_shift(&mut self) {
        self.register[0xF] = self.register[(self.opcode & 0x0F00) as usize >> 8] & 7;
        self.register[(self.opcode & 0x0F00) as usize >> 8] <<= 1;
    }

    /////////////////////////////////////////////////////////////////////////////
    ///                         SYSTEM OPCODES                               ///
    ///////////////////////////////////////////////////////////////////////////

    fn get_delay(&mut self) {
        self.register[(self.opcode & 0x0F00) as usize >> 8] = self.delay_timer;
        self.pc += 2;
    }

    fn wait_for_input(&mut self) {
        if !self.input_flag {
            return;
        }

        self.register[(self.opcode & 0x0F00) as usize >> 8] = self.last_key;
        self.pc += 2
    }

    fn set_delay(&mut self) {
        self.delay_timer = self.register[(self.opcode & 0x0F00) as usize >> 8];
        self.pc += 2;
    }

    fn set_sound(&mut self) {
        self.sound_timer = self.register[(self.opcode & 0x0F00) as usize >> 8];
        self.pc += 2;
    }

    fn get_sprite(&mut self) {
        self.index_register = self.register[(self.opcode & 0x0F00) as usize >> 8] as u16 * 0x5;
        self.pc += 2;
    }

    fn get_bcd(&mut self) {
        self.memory[self.index_register as usize] =
            self.register[(self.opcode & 0x0F00) as usize >> 8] / 100;
        self.memory[self.index_register as usize + 1] =
            (self.register[(self.opcode & 0x0F00) as usize >> 8] / 10) % 10;
        self.memory[self.index_register as usize + 2] =
            (self.register[(self.opcode & 0x0F00) as usize >> 8] % 100) % 10;
        self.pc += 2;
    }

    fn reg_dump(&mut self) {
        for offset in 0..((self.opcode & 0x0F00) as usize >> 8) + 1 {
            self.memory[self.index_register as usize + offset] = self.register[offset];
        }
        self.index_register += ((self.opcode & 0x0F00) >> 8) + 1;
        self.pc += 2;
    }

    fn reg_load(&mut self) {
        for offset in 0..((self.opcode & 0x0F00) as usize >> 8) + 1 {
            self.register[offset] = self.memory[self.index_register as usize + offset];
        }
        self.index_register += ((self.opcode & 0x0F00) >> 8) + 1;
        self.pc += 2;
    }
}
