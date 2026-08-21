use std::{f32::consts::PI, time::Instant};

use crate::caster::{cast_ray_3d, RayHit};
use crate::framebuffer::Framebuffer;
use crate::map_renderer::MapRenderer;
use crate::maze::Maze;
use crate::player::Player;
use crate::texture_manager::{Texel, TextureManager};

pub const BLOCK_SIZE: usize = 100;
pub const WIDTH: usize = 1200;
pub const HEIGHT: usize = 600;
const FOV: f32 = PI / 3.0;
const ATTENUATION_DISTANCE: f32 = 1800.0;
const MIN_BRIGHTNESS: f32 = 0.4;
const KEY_BOB_SPEED: f32 = 3.0;
const KEY_BOB_AMOUNT: f32 = 0.08;

#[derive(Clone, Copy)]
pub enum RenderMode {
    Map2D,
    View3D,
}

pub struct Renderer {
    mode: RenderMode,
    map_renderer: MapRenderer,
    textures: TextureManager,
    background_3d: Vec<u32>,
    animation_start: Instant,
}

impl Renderer {
    pub fn new(maze: &Maze) -> Self {
        let textures = TextureManager::load("assets")
            .expect("no se pudieron cargar las texturas 3D de assets");
        let background_3d = textures.static_background(WIDTH, HEIGHT);

        Self {
            mode: RenderMode::View3D,
            map_renderer: MapRenderer::new(maze),
            textures,
            background_3d,
            animation_start: Instant::now(),
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            RenderMode::Map2D => RenderMode::View3D,
            RenderMode::View3D => RenderMode::Map2D,
        };
    }

    pub fn is_map_visible(&self) -> bool {
        matches!(self.mode, RenderMode::Map2D)
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
                render_3d(
                    framebuffer,
                    maze,
                    player,
                    &self.textures,
                    &self.background_3d,
                    self.animation_start.elapsed().as_secs_f32(),
                );
            }
        }
    }
}

fn render_3d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    textures: &TextureManager,
    background: &[u32],
    animation_time: f32,
) {
    framebuffer.buffer.copy_from_slice(background);
    let wall_depths = render_walls(framebuffer, maze, player, textures);
    render_key_sprites(
        framebuffer,
        maze,
        player,
        textures,
        &wall_depths,
        animation_time,
    );
}

fn render_walls(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    textures: &TextureManager,
) -> Vec<f32> {
    let horizon = HEIGHT as f32 / 2.0;
    let distance_to_plane = (WIDTH as f32 / 2.0) / (FOV / 2.0).tan();
    let db = FOV / (WIDTH - 1) as f32;
    let mut wall_depths = vec![f32::INFINITY; WIDTH];

    for (i, wall_depth) in wall_depths.iter_mut().enumerate() {
        let relative_angle = -FOV / 2.0 + db * i as f32;
        let ray_angle = player.a + relative_angle;

        let Some(hit) = cast_ray_3d(maze, player, ray_angle, BLOCK_SIZE) else {
            continue;
        };

        let corrected_distance = (hit.distance * relative_angle.cos()).max(0.001);
        *wall_depth = corrected_distance;
        let wall_height = BLOCK_SIZE as f32 / corrected_distance * distance_to_plane;
        let projected_top = horizon - wall_height / 2.0;
        let projected_bottom = horizon + wall_height / 2.0;
        let top = projected_top.clamp(0.0, HEIGHT as f32) as usize;
        let bottom = projected_bottom.clamp(0.0, HEIGHT as f32) as usize;
        let texture = textures.texture_for_cell(hit.cell);
        let texture_u = wall_texture_u(&hit);
        let brightness = wall_brightness(corrected_distance);

        for y in top..bottom {
            let texture_v = (y as f32 - projected_top) / wall_height;
            let texel = attenuate_texel(texture.sample(texture_u, texture_v), brightness);
            draw_texel(framebuffer, i, y, texel);
        }
    }

    wall_depths
}

fn wall_texture_u(hit: &RayHit) -> f32 {
    let block_size = BLOCK_SIZE as f32;
    let local_x = hit.x.rem_euclid(block_size);
    let local_y = hit.y.rem_euclid(block_size);
    let distance_to_vertical_edge = local_x.min(block_size - local_x);
    let distance_to_horizontal_edge = local_y.min(block_size - local_y);

    if distance_to_vertical_edge < distance_to_horizontal_edge {
        local_y / block_size
    } else {
        local_x / block_size
    }
}

fn wall_brightness(distance: f32) -> f32 {
    let brightness = 1.0 / (1.0 + distance / ATTENUATION_DISTANCE);
    brightness.max(MIN_BRIGHTNESS)
}

fn attenuate_texel(texel: Texel, brightness: f32) -> Texel {
    let red = (((texel.color >> 16) & 0xFF) as f32 * brightness) as u32;
    let green = (((texel.color >> 8) & 0xFF) as f32 * brightness) as u32;
    let blue = ((texel.color & 0xFF) as f32 * brightness) as u32;

    Texel {
        color: (red << 16) | (green << 8) | blue,
        alpha: texel.alpha,
    }
}

fn render_key_sprites(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    textures: &TextureManager,
    wall_depths: &[f32],
    animation_time: f32,
) {
    let horizon = HEIGHT as f32 / 2.0;
    let distance_to_plane = (WIDTH as f32 / 2.0) / (FOV / 2.0).tan();
    let key_texture = textures.key();

    for (row, cells) in maze.iter().enumerate() {
        for (column, &cell) in cells.iter().enumerate() {
            if cell != 'K' {
                continue;
            }

            let key_x = (column as f32 + 0.5) * BLOCK_SIZE as f32;
            let key_y = (row as f32 + 0.5) * BLOCK_SIZE as f32;
            let difference_x = key_x - player.pos.x;
            let difference_y = key_y - player.pos.y;
            let distance = difference_x.hypot(difference_y);
            let angle_to_key = difference_y.atan2(difference_x);
            let relative_angle = normalize_angle(angle_to_key - player.a);

            if relative_angle.abs() > FOV / 2.0 {
                continue;
            }

            let depth = distance * relative_angle.cos();
            if depth <= 0.001 {
                continue;
            }
            let projected_cell_height = BLOCK_SIZE as f32 / depth * distance_to_plane;
            let sprite_size = projected_cell_height * 0.7;
            let center_x = WIDTH as f32 / 2.0 + relative_angle.tan() * distance_to_plane;
            let bob = (animation_time * KEY_BOB_SPEED).sin() * sprite_size * KEY_BOB_AMOUNT;
            let floor_y = horizon + projected_cell_height / 2.0 - bob;
            let left = (center_x - sprite_size / 2.0) as isize;
            let top = (floor_y - sprite_size) as isize;
            let right = (center_x + sprite_size / 2.0) as isize;
            let bottom = floor_y as isize;

            for screen_x in left.max(0)..right.min(WIDTH as isize) {
                let x = screen_x as usize;
                if depth >= wall_depths[x] {
                    continue;
                }

                let texture_u = (screen_x - left) as f32 / sprite_size;
                for screen_y in top.max(0)..bottom.min(HEIGHT as isize) {
                    let texture_v = (screen_y - top) as f32 / sprite_size;
                    let texel = key_texture.sample(texture_u, texture_v);
                    draw_texel(framebuffer, x, screen_y as usize, texel);
                }
            }
        }
    }
}

fn normalize_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(2.0 * PI) - PI
}

fn draw_texel(framebuffer: &mut Framebuffer, x: usize, y: usize, texel: Texel) {
    if texel.alpha > 128 {
        framebuffer.buffer[y * framebuffer.width + x] = texel.color;
    }
}
