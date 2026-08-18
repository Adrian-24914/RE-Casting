use std::f32::consts::PI;

use crate::caster::cast_ray_3d;
use crate::framebuffer::Framebuffer;
use crate::map_renderer::MapRenderer;
use crate::maze::Maze;
use crate::player::Player;

pub const BLOCK_SIZE: usize = 100;
pub const WIDTH: usize = 1200;
pub const HEIGHT: usize = 600;
const FOV: f32 = PI / 3.0;
const VIEW_3D_BACKGROUND_COLOR: u32 = 0x333355;

#[derive(Clone, Copy)]
pub enum RenderMode {
    Map2D,
    View3D,
}

pub struct Renderer {
    mode: RenderMode,
    map_renderer: MapRenderer,
}

impl Renderer {
    pub fn new(maze: &Maze) -> Self {
        Self {
            mode: RenderMode::View3D,
            map_renderer: MapRenderer::new(maze),
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            RenderMode::Map2D => RenderMode::View3D,
            RenderMode::View3D => RenderMode::Map2D,
        };
    }

    pub fn render(&mut self, framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
        self.map_renderer
            .update_exploration(maze, player, BLOCK_SIZE, FOV);

        match self.mode {
            RenderMode::Map2D => {
                self.map_renderer
                    .render(framebuffer, maze, player, BLOCK_SIZE);
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
        'D' => 0x8B4513,       // puerta
        'K' => 0xFFFF00,       // llave
        'L' => 0x00FF00,       // pared izquierda
        'R' => 0x000000,       // pared derecha
        'g' | 'G' => 0x00FF00, // meta
        _ => 0xFFDDDD,         // cualquier otra cosa
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
