use std::f32::consts::PI;

use crate::caster::{cast_ray, cast_ray_3d};
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub const BLOCK_SIZE: usize = 100;
pub const WIDTH: usize = 1300;
pub const HEIGHT: usize = 900;
const FOV: f32 = PI / 3.0;
const NUM_RAYS: usize = 5;
const MAP_VIEW_DISTANCE: f32 = BLOCK_SIZE as f32 * 3.0;
const MAP_BACKGROUND_COLOR: u32 = 0x000000;
const VIEW_3D_BACKGROUND_COLOR: u32 = 0x333355;

#[derive(Clone, Copy)]
pub enum RenderMode {
    Map2D,
    View3D,
}

pub struct Renderer {
    mode: RenderMode,
    explored: Vec<Vec<bool>>,
}

impl Renderer {
    pub fn new(maze: &Maze) -> Self {
        let explored = maze.iter().map(|row| vec![false; row.len()]).collect();

        Self {
            mode: RenderMode::View3D,
            explored,
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            RenderMode::Map2D => RenderMode::View3D,
            RenderMode::View3D => RenderMode::Map2D,
        };
    }

    pub fn render(&mut self, framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
        update_explored_map(maze, player, &mut self.explored);

        match self.mode {
            RenderMode::Map2D => {
                framebuffer.set_background_color(MAP_BACKGROUND_COLOR);
                framebuffer.clear();
                render_2d(framebuffer, maze, player, &mut self.explored);
            }
            RenderMode::View3D => {
                framebuffer.set_background_color(VIEW_3D_BACKGROUND_COLOR);
                framebuffer.clear();
                render_3d(framebuffer, maze, player);
            }
        }
    }
}

fn cell_color(cell: char) -> u32 {
    match cell {
        ' ' => 0x333355,       // suelo explorado
        '+' => 0x00AAFF,       // columnas
        '-' => 0xFF5555,       // paredes horizontales
        '|' => 0xFF5555,       // paredes verticales
        'g' | 'G' => 0x00FF00, // meta
        _ => 0xFFDDDD,         // cualquier otra cosa
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, cell: char) {
    framebuffer.set_current_color(cell_color(cell));

    for x in xo..xo + BLOCK_SIZE {
        for y in yo..yo + BLOCK_SIZE {
            framebuffer.point(x, y);
        }
    }
}

fn update_explored_map(maze: &Maze, player: &Player, explored: &mut [Vec<bool>]) {
    for i in 0..NUM_RAYS {
        let ray_fraction = i as f32 / (NUM_RAYS - 1) as f32;
        let angle = player.a - FOV / 2.0 + FOV * ray_fraction;
        cast_ray(
            None,
            maze,
            player,
            angle,
            BLOCK_SIZE,
            MAP_VIEW_DISTANCE,
            explored,
        );
    }
}

fn render_2d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    explored: &mut [Vec<bool>],
) {
    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            if explored[row][col] {
                draw_cell(framebuffer, col * BLOCK_SIZE, row * BLOCK_SIZE, cell);
            }
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
        cast_ray(
            Some(framebuffer),
            maze,
            player,
            angle,
            BLOCK_SIZE,
            MAP_VIEW_DISTANCE,
            explored,
        );
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
