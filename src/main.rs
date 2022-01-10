use macroquad::prelude::*;

mod cpu;

use cpu::CPU;

const PIXEL_SIZE: u16 = 10;

fn window_conf() -> Conf {
    Conf {
        window_title: "Damn-8".to_string(),
        window_width: 640,
        window_height: 320,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut cpu = CPU::new("./Breakout [Carmelo Cortez, 1979].ch8");

    clear_background(BLACK);

    loop {
        cpu.cycle();

        if cpu.draw_flag {
            draw_screen(&cpu);
        }

        next_frame().await
    }
}

fn draw_screen(cpu: &CPU) {
    clear_background(BLACK);

    for y in 0..32 {
        for x in 0..64 {
            if cpu.gfx[(x * y) as usize] != 0 {
                draw_rectangle((x * PIXEL_SIZE) as f32, (y * PIXEL_SIZE) as f32, PIXEL_SIZE as f32, PIXEL_SIZE as f32, WHITE);
            }
        }
    }
}
