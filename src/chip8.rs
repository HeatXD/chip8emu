// https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
// https://www.youtube.com/watch?v=YHkBgR6yvbY

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
    opcode: u16,        // every opcode is 2 bytes
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
}

impl Default for CHIP8 {
    fn default() -> Self {
        let mut memory: [u8; 4096] = [0; 4096];
        // load the fontset into memory
        memory[0..80].copy_from_slice(&FONTSET);
        println!("{:?}", &memory[0..100]);

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
        }
    }
}

impl CHIP8 {
    fn inc_pc(&mut self) {
        self.pc += 2; // every instruction is 2 bytes hence why we inc by 2
    }

    pub fn cycle(&mut self) {
        self.opcode =
            (self.memory[self.pc as usize] << 8 | self.memory[(self.pc + 1) as usize]) as u16;

        //X000
        let first = self.opcode >> 12;

        match first {
            Opcode::SYS => {
                match self.opcode {
                    Opcode::SYS_CLS => self.gfx[0..64 * 32].fill(0),
                    Opcode::SYS_RET => {
                        // decrement stack pointer
                        self.sp -= 1;
                        // get the addr needed to return
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => (),
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
                // Set Vx += kk
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
                let x = (self.opcode & 0x0F00) >> 8;
                let kk = self.opcode & 0x00FF;
            }
            _ => (),
        }
    }
}

struct Opcode;
impl Opcode {
    pub const SYS: u16 = 0x0;
    pub const SYS_CLS: u16 = 0x00E; // clear screen
    pub const SYS_RET: u16 = 0x0EE; // return from subroutine
    pub const JMP_1NNN: u16 = 0x1; // Jump addr
    pub const CALL_2NNN: u16 = 0x2; // Call addr
    pub const SKIP_3XKK: u16 = 0x3;
    pub const SKIP_4XKK: u16 = 0x4;
    pub const SKIP_5YX0: u16 = 0x5;
    pub const SET_6XKK: u16 = 0x6;
    pub const SET_7XKK: u16 = 0x7;
    pub const SET_8XY: u16 = 0x8;
    pub const SKIP_9YX0: u16 = 0x9;
    pub const SET_ANNN: u16 = 0xA;
    pub const JMP_BNNN: u16 = 0xB;
    pub const SET_CXKK: u16 = 0xC;
}
