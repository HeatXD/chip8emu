mod chip8;

use std::env;

use chip8::CHIP8;
use macroquad::prelude::*;

#[macroquad::main("CHIP-8 Emulator By HeatXD")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("NO ROM GIVEN");
    }

    let mut image = Image::gen_image_color(64, 32, BLACK);

    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    request_new_screen_size(1024., 512.);

    let mut cpu = CHIP8::default();

    cpu.load_rom(&args[1]);

    while !is_quit_requested() {
        cpu.cycle();

        // todo poll input
        clear_background(WHITE);
        update_image(&cpu, &mut image, &texture);
        draw_texture_ex(
            &texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(1024., 512.)),
                ..Default::default()
            },
        );
        next_frame().await
    }
}

fn update_image(cpu: &CHIP8, image: &mut Image, texture: &Texture2D) {
    let gfx = cpu.get_gfx();
    let mut col: Color;
    for i in 0..gfx.len() {
        if gfx[i] == 1 {
            col = WHITE;
        } else {
            col = BLACK;
        }
        image.bytes[i * 4 + 0] = (col.r * 255.) as u8;
        image.bytes[i * 4 + 1] = (col.g * 255.) as u8;
        image.bytes[i * 4 + 2] = (col.b * 255.) as u8;
        image.bytes[i * 4 + 3] = (col.a * 255.) as u8;
    }

    texture.update(&image);
}
