use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub fn cast_ray(
    mut framebuffer: Option<&mut Framebuffer>,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    max_distance: f32,
    explored: &mut [Vec<bool>],
) {
    let mut d = 0.0;

    if let Some(framebuffer) = framebuffer.as_deref_mut() {
        framebuffer.set_current_color(0xFFDDDD);
    }

    while d <= max_distance {
        let ray_x = player.pos.x + d * a.cos();
        let ray_y = player.pos.y + d * a.sin();

        if ray_x < 0.0 || ray_y < 0.0 {
            return;
        }

        let x = ray_x as usize;
        let y = ray_y as usize;

        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return;
        }

        explored[j][i] = true;

        if maze[j][i] != ' ' {
            return;
        }

        if let Some(framebuffer) = framebuffer.as_deref_mut() {
            framebuffer.point(x, y);
        }

        d += 1.0;
    }
}

pub fn cast_ray_3d(maze: &Maze, player: &Player, a: f32, block_size: usize) -> Option<(f32, char)> {
    let mut d = 0.0;

    loop {
        let x = player.pos.x + d * a.cos();
        let y = player.pos.y + d * a.sin();

        if x < 0.0 || y < 0.0 {
            return None;
        }

        let i = x as usize / block_size;
        let j = y as usize / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return None;
        }

        if maze[j][i] != ' ' {
            return Some((d, maze[j][i]));
        }

        d += 1.0;
    }
}
