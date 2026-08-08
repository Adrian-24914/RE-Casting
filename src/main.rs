mod caster;
mod framebuffer;
mod maze;
mod player;
mod renderer;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Duration;

use crate::framebuffer::Framebuffer;
use crate::maze::load_maze;
use crate::player::process_events;
use crate::renderer::{Renderer, BLOCK_SIZE, HEIGHT, WIDTH};

fn main() {
    let frame_delay = Duration::from_millis(16);

    let (maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(0x333355);

    let mut window = Window::new(
        "Resident Evil Casting",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();
    let mut renderer = Renderer::new(&maze);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        process_events(&window, &mut player, &maze, BLOCK_SIZE);

        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            renderer.toggle_mode();
        }

        // ¿el jugador llegó a la meta? Se traduce su posición en píxeles a la
        // celda que ocupa y se revisa si esa celda es la marca `g`.
        let i = player.pos.x as usize / BLOCK_SIZE;
        let j = player.pos.y as usize / BLOCK_SIZE;
        if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
            println!("¡Meta alcanzada! Fin del juego.");
            break;
        }

        renderer.render(&mut framebuffer, &maze, &player);

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
