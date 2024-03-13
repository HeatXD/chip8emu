mod chip8;

use std::env;

use chip8::CHIP8;
use macroquad::{
    audio::{
        load_sound, play_sound, stop_sound, PlaySoundParams,
    },
    prelude::*,
};

const KEYS: [KeyCode; 16] = [
    KeyCode::X,
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::Z,
    KeyCode::C,
    KeyCode::Key4,
    KeyCode::R,
    KeyCode::F,
    KeyCode::V,
];

#[macroquad::main("CHIP-8 Emulator By HeatXD")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("NO ROM GIVEN");
    }

    // load beep sound
    set_pc_assets_folder("assets");
    let beep = load_sound("beep.wav").await.expect("Couldn't load audio");

    // set window size following 64x32 ratio
    request_new_screen_size(1024., 512.);

    // drawing prep
    let mut image = Image::gen_image_color(64, 32, BLACK);
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    // setup chip8 and load rom
    let mut cpu = CHIP8::default();
    cpu.load_rom(&args[1]);

    while !is_quit_requested() {
        
        cpu.cycle();
        // update timers
        cpu.advance_frame();
        // update keys
        for (idx, key) in KEYS.iter().enumerate() {
            cpu.set_key(idx, is_key_down(*key));
        }
        
        if cpu.did_beep() {
            // stop if its still playing
            stop_sound(&beep);
            play_sound(
                &beep,
                PlaySoundParams {
                    volume: 0.2,
                    looped: false,
                },
            );
        }

        clear_background(WHITE);

        if cpu.did_draw() {
            update_image(&cpu, &mut image, &texture);
        }
        
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
