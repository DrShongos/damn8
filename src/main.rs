use macroquad::prelude::*;
use std::env;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

mod cpu;

use cpu::CPU;

const PIXEL_SIZE: f32 = 10.0;

const KEYMAP: [KeyCode; 16] = [
    KeyCode::X,    // 0
    KeyCode::Key1, // 1
    KeyCode::Key2, // 2
    KeyCode::Key3, // 3
    KeyCode::Q,    // 4
    KeyCode::W,    // 5
    KeyCode::E,    // 6
    KeyCode::A,    // 7
    KeyCode::S,    // 8
    KeyCode::D,    // 9
    KeyCode::Z,    // A
    KeyCode::C,    // B
    KeyCode::Key4, // C
    KeyCode::R,    // D
    KeyCode::F,    // E
    KeyCode::V,    // F
];

#[derive(Deserialize, Serialize)]
struct Config {
    color_on: (u8, u8, u8, u8),
    color_off: (u8, u8, u8, u8),
}

impl Default for Config {
    fn default() -> Self {
        Self {
            color_on: (255, 255, 255, 255),
            color_off: (0, 0, 0, 255),
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please specify a path.");
        std::process::exit(1);
    }
    let path = &args[1];

    let config = load_config();
    let color_off = Color::from_rgba(
        config.color_off.0,
        config.color_off.1,
        config.color_off.2,
        config.color_off.3,
    );
    let color_on = Color::from_rgba(
        config.color_on.0,
        config.color_on.1,
        config.color_on.2,
        config.color_on.3,
    );

    let mut cpu = CPU::new(path.as_str());
    CPU::initialize_os_table();

    clear_background(BLACK);

    loop {
        for i in 0..16 {
            if !is_key_down(KEYMAP[i]) {
                cpu.keypad[i] = 0;
                cpu.input_flag = false;
            }
        }

        get_input(&mut cpu);

        cpu.cycle();

        if cpu.draw_flag {
            draw_screen(&cpu, &color_off, &color_on);
        }

        next_frame().await
    }
}

fn draw_screen(cpu: &CPU, color_off: &Color, color_on: &Color) {
    clear_background(*color_off);

    for y in 0..32 {
        for x in 0..64 {
            if cpu.gfx[(y * 64 + x) as usize] != 0 {
                draw_rectangle(
                    x as f32 * PIXEL_SIZE,
                    y as f32 * PIXEL_SIZE,
                    PIXEL_SIZE,
                    PIXEL_SIZE,
                    *color_on,
                );
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Damn-8".to_string(),
        window_width: 640,
        window_height: 320,
        window_resizable: false,
        ..Default::default()
    }
}

fn load_config() -> Config {
    let path = Path::new("config.ron");
    if !path.exists() {
        let config = Config::default();
        fs::write(path, ron::to_string(&config).unwrap()).unwrap();
        return config;
    }

    let file = fs::File::open(path).unwrap();
    let config: Config = ron::de::from_reader(file).unwrap();

    config
}

fn get_input(cpu: &mut CPU) {
    if let Some(key) = get_last_key_pressed() {
        if KEYMAP.contains(&key) {
            let code = KEYMAP.iter().position(|k| k == &key).unwrap();
            cpu.input_flag = true;
            cpu.last_key = code as u8;
            cpu.keypad[code] = 1;
        }
    }
}
