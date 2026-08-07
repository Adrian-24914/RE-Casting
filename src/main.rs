mod caster;
mod framebuffer;
mod maze;
mod player;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::Duration;

use crate::caster::{cast_ray, cast_ray_3d};
use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, Maze};
use crate::player::{process_events, Player};

const BLOCK_SIZE: usize = 100;
const WIDTH: usize = 1300;
const HEIGHT: usize = 900;
const FOV: f32 = PI / 3.0;

const NUM_RAYS: usize = 5;

#[derive(Clone, Copy)]
enum RenderMode {
    Map2D,
    View3D,
}

fn cell_color(cell: char) -> u32 {
    match cell {
        '+' => 0x00AAFF,       // columnas
        '-' => 0xFF5555,       // paredes horizontales
        '|' => 0xFF5555,       // paredes verticales
        'g' | 'G' => 0x00FF00, // meta
        _ => 0xFFDDDD,         // cualquier otra cosa
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, cell: char) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(cell_color(cell));

    for x in xo..xo + BLOCK_SIZE {
        for y in yo..yo + BLOCK_SIZE {
            framebuffer.point(x, y);
        }
    }
}

fn render_2d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            draw_cell(framebuffer, col * BLOCK_SIZE, row * BLOCK_SIZE, cell);
        }
    }

    framebuffer.set_current_color(0xFFFF00);

    let px = player.pos.x as usize;
    let py = player.pos.y as usize;

    for x in px.saturating_sub(3)..=px + 5 {
        for y in py.saturating_sub(3)..=py + 5 {
            framebuffer.point(x, y);
        }
    }

    for i in 0..NUM_RAYS {
        let ray_fraction = i as f32 / (NUM_RAYS - 1) as f32; // de 0.0 a 1.0
        let angle = player.a - FOV / 2.0 + FOV * ray_fraction;
        cast_ray(framebuffer, maze, player, angle, BLOCK_SIZE);
    }
}

fn render_3d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    let horizon = HEIGHT as f32 / 2.0;
    let distance_to_plane = (WIDTH as f32 / 2.0) / (FOV / 2.0).tan();
    let db = FOV / (WIDTH - 1) as f32;

    for i in 0..WIDTH {
        let b = -FOV / 2.0 + db * i as f32;
        let ray_angle = player.a + b;

        let Some((raw_distance, wall)) = cast_ray_3d(maze, player, ray_angle, BLOCK_SIZE) else {
            continue;
        };

        let corrected_distance = raw_distance * b.cos();
        let wall_height = BLOCK_SIZE as f32 / corrected_distance * distance_to_plane;

        let top = (horizon - wall_height / 2.0).clamp(0.0, HEIGHT as f32) as usize;
        let bottom = (horizon + wall_height / 2.0).clamp(0.0, HEIGHT as f32) as usize;

        framebuffer.set_current_color(cell_color(wall));
        for y in top..bottom {
            framebuffer.point(i, y);
        }
    }
}

fn main() {
    let frame_delay = Duration::from_millis(16);

    let (maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(0x333355);

    let mut window = Window::new("Resident Evil Casting", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut render_mode = RenderMode::View3D;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        process_events(&window, &mut player, &maze, BLOCK_SIZE);

        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            render_mode = match render_mode {
                RenderMode::Map2D => RenderMode::View3D,
                RenderMode::View3D => RenderMode::Map2D,
            };
        }

        // ¿el jugador llegó a la meta? Se traduce su posición en píxeles a la
        // celda que ocupa y se revisa si esa celda es la marca `g`.
        let i = player.pos.x as usize / BLOCK_SIZE;
        let j = player.pos.y as usize / BLOCK_SIZE;
        if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
            println!("¡Meta alcanzada! Fin del juego.");
            break;
        }

        framebuffer.clear();

        match render_mode {
            RenderMode::Map2D => render_2d(&mut framebuffer, &maze, &player),
            RenderMode::View3D => render_3d(&mut framebuffer, &maze, &player),
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
