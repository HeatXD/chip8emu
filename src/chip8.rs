// https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
// https://www.youtube.com/watch?v=YHkBgR6yvbY

use core::panic;
use std::fs;

use macroquad::rand::{rand, srand};

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

pub struct CHIP8 {
    opcode: u16,        // every current_opcode is 2 bytes
    memory: [u8; 4096], // 4k memory
    regs: [u8; 16],     // 15 general puporse registers + 1 carry flag
    index: u16,         // index register
    pc: u16,            // program counter

    gfx: [u8; 64 * 32], // graphics

    d_timer: u8, // delay timer
    s_timer: u8, // sound timer

    stack: [u16; 16], // 16 2 byte long adressess
    sp: u16,          // stack pointer

    keys: [u8; 16], // pressed keyboard keys

    is_rom_loaded: bool,
    draw_happend: bool,
    did_beep: bool,
}

impl Default for CHIP8 {
    fn default() -> Self {
        // seed rand
        srand(1);

        let mut memory: [u8; 4096] = [0; 4096];
        // load the fontset into memory
        memory[0..80].copy_from_slice(&FONTSET);
        // println!("{:?}", &memory[0..100]);
        Self {
            opcode: 0,
            memory,
            regs: [0; 16],
            index: 0,
            pc: 0x200, // most chip8 programs start at 0x200
            gfx: [0; 64 * 32],
            d_timer: 0,
            s_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
            is_rom_loaded: false,
            draw_happend: false,
            did_beep: false,
        }
    }
}

impl CHIP8 {
    fn inc_pc(&mut self) {
        self.pc += 2; // every instruction is 2 bytes hence why we inc by 2
    }

    pub fn cycle(&mut self) {
        if self.pc > 0xFFF {
            panic!("Opcode out of range! Program Error!");
        }

        if !self.is_rom_loaded {
            return;
        }

        self.draw_happend = false;
        self.did_beep = false;

        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | self.memory[(self.pc + 1) as usize] as u16;

        let first = self.opcode >> 12;

        println!("0x{:X} ,, 0x{:X} ,, 0x{:X}", first, self.opcode, self.pc);

        match first {
            Opcode::SYS => {
                match self.opcode {
                    0x0E0 => {
                        // clear the screen
                        self.gfx = [0; 64 * 32];
                        // set draw flag for the frame to true
                        self.draw_happend = true;
                    }
                    0x0EE => {
                        // return from subroutine
                        // decrement stack pointer
                        self.sp -= 1;
                        // get the addr needed to return
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => panic!("Unknown Opcode: 0x{:X}", self.opcode),
                }
                self.inc_pc();
            }
            Opcode::JMP_1NNN => self.pc = self.opcode & 0x0FFF,
            Opcode::CALL_2NNN => {
                // store pc on the stack.
                self.stack[self.sp as usize] = self.pc;
                // inc stack pointer
                self.sp += 1;
                // set pc to the address
                self.pc = self.opcode & 0x0FFF
            }
            Opcode::SKIP_3XKK => {
                let x = (self.opcode & 0x0F00) >> 8;
                // check if values are Vx == KK
                if self.regs[x as usize] == (self.opcode & 0x00FF) as u8 {
                    self.inc_pc();
                }
                self.inc_pc();
            }
            Opcode::SKIP_4XKK => {
                let x = (self.opcode & 0x0F00) >> 8;
                // check if values are Vx != KK
                if self.regs[x as usize] != (self.opcode & 0x00FF) as u8 {
                    self.inc_pc();
                }
                self.inc_pc();
            }
            Opcode::SKIP_5YX0 => {
                let x = (self.opcode & 0x0F00) >> 8;
                let y = (self.opcode & 0x00F0) >> 4;
                // check if values are Vx == Vy
                if self.regs[x as usize] == self.regs[y as usize] {
                    self.inc_pc();
                }
                self.inc_pc();
            }
            Opcode::SET_6XKK => {
                let x = (self.opcode & 0x0F00) >> 8;
                // Set Vx = kk
                self.regs[x as usize] = (self.opcode & 0x00FF) as u8;
                self.inc_pc();
            }
            Opcode::SET_7XKK => {
                let x = (self.opcode & 0x0F00) >> 8;
                self.regs[x as usize] += (self.opcode & 0x00FF) as u8;
                self.inc_pc();
            }
            Opcode::SET_8XY => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let y = ((self.opcode & 0x00F0) >> 4) as usize;
                let mode = self.opcode & 0x000F;

                match mode {
                    0 => self.regs[x] = self.regs[y],  // Set Vx = Vy.
                    1 => self.regs[x] |= self.regs[y], // Set Vx = Vx OR Vy.
                    2 => self.regs[x] &= self.regs[y], // Set Vx = Vx AND Vy.
                    3 => self.regs[x] ^= self.regs[y], // Set Vx = Vx XOR Vy.
                    4 => {
                        // Set Vx = Vx + Vy, set VF = carry.
                        let mut sum = self.regs[x] as u16;
                        sum += self.regs[y] as u16;
                        // set flag register
                        self.regs[0xF] = if sum > 255 { 1 } else { 0 };
                        self.regs[x] = (sum & 0x00FF) as u8;
                    }
                    5 => {
                        // Set Vx = Vx - Vy, set VF = NOT borrow.
                        // set flag register
                        self.regs[0xF] = if self.regs[x] > self.regs[y] { 1 } else { 0 };
                        self.regs[x] -= self.regs[y];
                    }
                    6 => {
                        // Set Vx = Vx SHR 1.
                        self.regs[0xF] = self.regs[x] & 0x1;
                        self.regs[x] >>= 1;
                    }
                    7 => {
                        // Set Vx = Vy - Vx, set VF = NOT borrow.
                        self.regs[0xF] = if self.regs[y] > self.regs[x] { 1 } else { 0 };
                        self.regs[x] = self.regs[y] - self.regs[x];
                    }
                    14 => {
                        // Set Vx = Vx SHL 1.
                        self.regs[0xF] = if self.regs[x] & 0x80 != 0 { 1 } else { 0 };
                        self.regs[x] <<= 1;
                    }
                    _ => (),
                }

                self.inc_pc();
            }
            Opcode::SKIP_9YX0 => {
                // Skip next instruction if Vx != Vy.
                let x = (self.opcode & 0x0F00) >> 8;
                let y = (self.opcode & 0x00F0) >> 4;
                // check if values are Vx != Vy
                if self.regs[x as usize] != self.regs[y as usize] {
                    self.inc_pc();
                }
                self.inc_pc();
            }
            Opcode::SET_ANNN => {
                // Set I = nnn.
                self.index = self.opcode & 0x0FFF;
                self.inc_pc();
            }
            Opcode::JMP_BNNN => {
                // Jump to location nnn + V0.
                self.pc = (self.opcode & 0x0FFF) + self.regs[0] as u16;
            }
            Opcode::SET_CXKK => {
                // Set Vx = random byte AND kk.
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let kk = self.opcode & 0x00FF;

                self.regs[x] = (rand() & kk as u32) as u8;
                self.inc_pc();
            }
            Opcode::DISPLAY_DXYN => {
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                let reg_x: u16 = self.regs[((self.opcode & 0x0F00) >> 8) as usize] as u16;
                let reg_y: u16 = self.regs[((self.opcode & 0x00F0) >> 4) as usize] as u16;
                let height = self.opcode & 0x000F;

                let mut y = 0;
                while y < height {
                    let pixel = self.memory[(self.index + y) as usize];
                    let mut x = 0;
                    while x < 8 {
                        let msb = 0x80;
                        if pixel & (msb >> x) != 0 {
                            let tx = (reg_x + x) % 64;
                            let ty = (reg_y + y) % 32;

                            let idx = (tx + ty * 64) as usize;
                            self.gfx[idx] ^= 1;

                            if self.gfx[idx] == 0 {
                                self.regs[0xF] = 1;
                            }
                        }
                        x += 1;
                    }
                    y += 1;
                }
                // set draw flag for the frame to true
                self.draw_happend = true;

                self.inc_pc();
            }
            Opcode::SKIP_EX => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let mode = self.opcode & 0x00FF;
                match mode {
                    // Skip next instruction if key with the value of Vx is pressed.
                    0x9E => {
                        if self.keys[self.regs[x] as usize] == 1 {
                            self.inc_pc();
                        }
                    }
                    // Skip next instruction if key with the value of Vx is not pressed.
                    0xA1 => {
                        if self.keys[self.regs[x] as usize] != 1 {
                            self.inc_pc();
                        }
                    }
                    _ => (),
                }
                self.inc_pc();
            }
            Opcode::MISC_FX => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let mode = self.opcode & 0x00FF;

                match mode {
                    0x07 => self.regs[x] = self.d_timer, // Set Vx = delay timer value.
                    0x0A => {
                        let mut key_pressed = false;

                        for (i, v) in self.keys.iter().enumerate() {
                            if *v != 0 {
                                self.regs[x] = i as u8;
                                key_pressed = true;
                                break;
                            }
                        }

                        if !key_pressed {
                            return;
                        }
                    }
                    0x15 => self.d_timer = self.regs[x], // Set delay timer = Vx.
                    0x18 => self.s_timer = self.regs[x], // Set sound timer = Vx.
                    0x1E => self.index += self.regs[x] as u16, // Set I = I + Vx.
                    0x29 => {
                        // Set I = location of sprite for digit Vx.
                        if self.regs[x] < 16 {
                            self.index = (self.regs[x] * 0x5) as u16;
                        }
                    }
                    0x33 => {
                        // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                        let idx = self.index as usize;
                        self.memory[idx] = self.regs[x] / 100;
                        self.memory[idx + 1] = (self.regs[x] / 10) % 10;
                        self.memory[idx + 2] = self.regs[x] % 10;
                    }
                    0x55 => {
                        // Store registers V0 through Vx in memory starting at location I.
                        let mut i: usize = 0;
                        while i <= x {
                            self.memory[i + self.index as usize] = self.regs[i];
                            i += 1;
                        }
                    }
                    0x65 => {
                        // Read registers V0 through Vx from memory starting at location I.
                        let mut i: usize = 0;
                        while i <= x {
                            self.regs[i] = self.memory[i + self.index as usize];
                            i += 1;
                        }
                    }
                    _ => (),
                }
                self.inc_pc();
            }
            _ => (),
        }

        if self.d_timer > 0 {
            self.d_timer -= 1;
        }

        if self.s_timer > 0 {
            if self.s_timer == 1 {
                self.did_beep = true;
            }
            self.s_timer -= 1;
        }
    }

    pub fn load_rom(&mut self, filepath: &str) {
        if let Ok(rom) = fs::read(filepath) {
            //println!("{}", rom.len());
            for (i, byte) in rom.iter().enumerate() {
                self.memory[i + 0x200] = *byte;
            }
            self.is_rom_loaded = true;
        } else {
            panic!("Failed to read file! {}", filepath);
        }
    }

    pub fn get_gfx(&self) -> &[u8; 2048] {
        &self.gfx
    }

    pub fn did_draw(&self) -> bool {
        self.draw_happend
    }

    pub fn did_beep(&self) -> bool {
        self.did_beep
    }

    pub fn set_key(&mut self, idx: usize, state: bool) {
        self.keys[idx] = state as u8;
    }
}

struct Opcode;
impl Opcode {
    const SYS: u16 = 0x0;
    const JMP_1NNN: u16 = 0x1;
    const CALL_2NNN: u16 = 0x2;
    const SKIP_3XKK: u16 = 0x3;
    const SKIP_4XKK: u16 = 0x4;
    const SKIP_5YX0: u16 = 0x5;
    const SET_6XKK: u16 = 0x6;
    const SET_7XKK: u16 = 0x7;
    const SET_8XY: u16 = 0x8;
    const SKIP_9YX0: u16 = 0x9;
    const SET_ANNN: u16 = 0xA;
    const JMP_BNNN: u16 = 0xB;
    const SET_CXKK: u16 = 0xC;
    const DISPLAY_DXYN: u16 = 0xD;
    const SKIP_EX: u16 = 0xE;
    const MISC_FX: u16 = 0xF;
}
