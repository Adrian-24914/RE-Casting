use crate::maze::Maze;
use crate::player::Player;

pub fn cast_ray(
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    max_distance: f32,
    explored: &mut [Vec<bool>],
) -> f32 {
    let mut d = 0.0;

    while d <= max_distance {
        let ray_x = player.pos.x + d * a.cos();
        let ray_y = player.pos.y + d * a.sin();

        if ray_x < 0.0 || ray_y < 0.0 {
            return d;
        }

        let x = ray_x as usize;
        let y = ray_y as usize;

        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return d;
        }

        explored[j][i] = true;

        if maze[j][i] != ' ' {
            return d;
        }

        d += 1.0;
    }

    max_distance
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
