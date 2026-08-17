use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

const MAP_PADDING: f32 = 20.0;
const NUM_RAYS: usize = 5;
const VIEW_DISTANCE_IN_CELLS: f32 = 3.0;

const BACKGROUND_COLOR: u32 = 0x08080D;
const UNEXPLORED_COLOR: u32 = 0x161622;
const PLAYER_COLOR: u32 = 0xFFFF00;
const RAY_COLOR: u32 = 0xFFDDDD;

#[derive(Clone, Copy, Default)]
struct Ray {
    angle: f32,
    distance: f32,
}

/// Estado y dibujo del mapa 2D.
///
/// Las posiciones del juego siguen expresadas con `block_size`, mientras que
/// `MapLayout` las convierte a píxeles del framebuffer. 
pub struct MapRenderer {
    explored: Vec<Vec<bool>>,
    rays: [Ray; NUM_RAYS],
}

impl MapRenderer {
    pub fn new(maze: &Maze) -> Self {
        Self {
            explored: exploration_grid(maze),
            rays: [Ray::default(); NUM_RAYS],
        }
    }

    pub fn update_exploration(
        &mut self,
        maze: &Maze,
        player: &Player,
        block_size: usize,
        fov: f32,
    ) {
        // Permite reutilizar el renderer si el laberinto cambia de dimensiones.
        if !same_shape(&self.explored, maze) {
            self.explored = exploration_grid(maze);
        }

        let max_distance = block_size as f32 * VIEW_DISTANCE_IN_CELLS;

        for (index, ray) in self.rays.iter_mut().enumerate() {
            let fraction = index as f32 / (NUM_RAYS - 1) as f32;
            let angle = player.a - fov / 2.0 + fov * fraction;
            let distance = cast_ray(
                maze,
                player,
                angle,
                block_size,
                max_distance,
                &mut self.explored,
            );

            *ray = Ray { angle, distance };
        }
    }

    pub fn render(
        &self,
        framebuffer: &mut Framebuffer,
        maze: &Maze,
        player: &Player,
        block_size: usize,
    ) {
        framebuffer.set_background_color(BACKGROUND_COLOR);
        framebuffer.clear();

        let Some(layout) = MapLayout::new(framebuffer.width, framebuffer.height, maze) else {
            return;
        };

        self.draw_cells(framebuffer, maze, &layout);
        self.draw_rays(framebuffer, player, block_size, &layout);
        self.draw_player(framebuffer, player, block_size, &layout);
    }

    fn draw_cells(&self, framebuffer: &mut Framebuffer, maze: &Maze, layout: &MapLayout) {
        for (row, line) in maze.iter().enumerate() {
            for (column, &cell) in line.iter().enumerate() {
                let is_explored = self
                    .explored
                    .get(row)
                    .and_then(|line| line.get(column))
                    .copied()
                    .unwrap_or(false);
                let color = if is_explored {
                    explored_cell_color(cell)
                } else {
                    UNEXPLORED_COLOR
                };
                let (left, top, right, bottom) = layout.cell_bounds(column, row);

                draw_rectangle(framebuffer, left, top, right, bottom, color);
            }
        }
    }

    fn draw_rays(
        &self,
        framebuffer: &mut Framebuffer,
        player: &Player,
        block_size: usize,
        layout: &MapLayout,
    ) {
        let start = layout.world_to_screen(player.pos.x, player.pos.y, block_size);

        for ray in self.rays {
            let end_x = player.pos.x + ray.distance * ray.angle.cos();
            let end_y = player.pos.y + ray.distance * ray.angle.sin();
            let end = layout.world_to_screen(end_x, end_y, block_size);

            draw_line(framebuffer, start, end, RAY_COLOR);
        }
    }

    fn draw_player(
        &self,
        framebuffer: &mut Framebuffer,
        player: &Player,
        block_size: usize,
        layout: &MapLayout,
    ) {
        let (center_x, center_y) = layout.world_to_screen(player.pos.x, player.pos.y, block_size);
        let radius = (layout.scale * 0.12).round().clamp(2.0, 6.0) as isize;

        draw_rectangle(
            framebuffer,
            center_x - radius,
            center_y - radius,
            center_x + radius + 1,
            center_y + radius + 1,
            PLAYER_COLOR,
        );
    }
}

struct MapLayout {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
}

impl MapLayout {
    fn new(frame_width: usize, frame_height: usize, maze: &Maze) -> Option<Self> {
        let rows = maze.len();
        let columns = maze.iter().map(Vec::len).max().unwrap_or(0);

        if rows == 0 || columns == 0 || frame_width == 0 || frame_height == 0 {
            return None;
        }

        let horizontal_padding = MAP_PADDING.min(frame_width.saturating_sub(1) as f32 / 2.0);
        let vertical_padding = MAP_PADDING.min(frame_height.saturating_sub(1) as f32 / 2.0);
        let available_width = frame_width as f32 - horizontal_padding * 2.0;
        let available_height = frame_height as f32 - vertical_padding * 2.0;
        let scale = (available_width / columns as f32).min(available_height / rows as f32);
        let map_width = columns as f32 * scale;
        let map_height = rows as f32 * scale;

        Some(Self {
            scale,
            offset_x: (frame_width as f32 - map_width) / 2.0,
            offset_y: (frame_height as f32 - map_height) / 2.0,
        })
    }

    fn cell_bounds(&self, column: usize, row: usize) -> (isize, isize, isize, isize) {
        let left = (self.offset_x + column as f32 * self.scale).floor() as isize;
        let top = (self.offset_y + row as f32 * self.scale).floor() as isize;
        let right = (self.offset_x + (column + 1) as f32 * self.scale).ceil() as isize;
        let bottom = (self.offset_y + (row + 1) as f32 * self.scale).ceil() as isize;

        (left, top, right, bottom)
    }

    fn world_to_screen(&self, world_x: f32, world_y: f32, block_size: usize) -> (isize, isize) {
        let scale_from_world = self.scale / block_size as f32;
        let screen_x = self.offset_x + world_x * scale_from_world;
        let screen_y = self.offset_y + world_y * scale_from_world;

        (screen_x.round() as isize, screen_y.round() as isize)
    }
}

fn exploration_grid(maze: &Maze) -> Vec<Vec<bool>> {
    maze.iter().map(|row| vec![false; row.len()]).collect()
}

fn same_shape(explored: &[Vec<bool>], maze: &Maze) -> bool {
    explored.len() == maze.len()
        && explored
            .iter()
            .zip(maze)
            .all(|(explored_row, maze_row)| explored_row.len() == maze_row.len())
}

fn explored_cell_color(cell: char) -> u32 {
    match cell {
        ' ' => 0x333355,
        '+' => 0x00AAFF,
        '-' | '|' => 0xFF5555,
        'g' | 'G' => 0x00FF00,
        _ => 0xFFDDDD,
    }
}

fn draw_rectangle(
    framebuffer: &mut Framebuffer,
    left: isize,
    top: isize,
    right: isize,
    bottom: isize,
    color: u32,
) {
    let left = left.clamp(0, framebuffer.width as isize) as usize;
    let top = top.clamp(0, framebuffer.height as isize) as usize;
    let right = right.clamp(0, framebuffer.width as isize) as usize;
    let bottom = bottom.clamp(0, framebuffer.height as isize) as usize;

    framebuffer.set_current_color(color);
    for y in top..bottom {
        for x in left..right {
            framebuffer.point(x, y);
        }
    }
}

fn draw_line(
    framebuffer: &mut Framebuffer,
    (mut x0, mut y0): (isize, isize),
    (x1, y1): (isize, isize),
    color: u32,
) {
    let dx = (x1 - x0).abs();
    let step_x = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let step_y = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    framebuffer.set_current_color(color);
    loop {
        if x0 >= 0 && y0 >= 0 {
            framebuffer.point(x0 as usize, y0 as usize);
        }

        if x0 == x1 && y0 == y1 {
            break;
        }

        let doubled_error = error * 2;
        if doubled_error >= dy {
            error += dy;
            x0 += step_x;
        }
        if doubled_error <= dx {
            error += dx;
            y0 += step_y;
        }
    }
}
