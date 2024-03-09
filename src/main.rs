mod chip8;

use chip8::CHIP8;
use macroquad::prelude::*;

#[macroquad::main("CHIP-8 Emulator By HeatXD")]
async fn main() {
    let mut cpu = CHIP8::default();
    // todo load rom
    loop {
        cpu.cycle();
        
        // todo poll input
        clear_background(BLACK);

        //todo draw gfx
        next_frame().await
    }
}