// https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/
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
        for i in 0..FONTSET.len() {
            memory[i] = FONTSET[i];
        }

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
    pub fn increment_pc(&mut self) {
        self.pc += 2; // every instruction is 2 bytes hence why we inc by 2
    }

    pub fn cycle(&mut self) {
        self.opcode = (self.memory[self.pc as usize] << 8 | self.memory[(self.pc + 1) as usize]) as u16;
        
        //X000
        let first = self.opcode >> 12;
    }
}