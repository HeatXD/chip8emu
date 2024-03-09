mod chip8;

use chip8::CHIP8;
use macroquad::prelude::*;

#[macroquad::main("CHIP-8 Emulator By HeatXD")]
async fn main() {
    let mut cpu = CHIP8::default();
    // todo load rom
    loop {
        cpu.cycle();

        clear_background(BLACK);

        draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}