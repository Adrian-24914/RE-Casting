use crate::maze::Maze;
use crate::player::Player;

pub struct RayHit {
    pub distance: f32,
    pub cell: char,
    pub x: f32,
    pub y: f32,
}

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

pub fn cast_ray_3d(maze: &Maze, player: &Player, a: f32, block_size: usize) -> Option<RayHit> {
    const RAY_STEP: f32 = 5.0;
    let mut distance = 0.0;

    loop {
        let x = player.pos.x + distance * a.cos();
        let y = player.pos.y + distance * a.sin();

        if x < 0.0 || y < 0.0 {
            return None;
        }

        let column = x as usize / block_size;
        let row = y as usize / block_size;

        if row >= maze.len() || column >= maze[row].len() {
            return None;
        }

        let cell = maze[row][column];
        if !matches!(cell, ' ' | 'K') {
            // El salto de 5 píxeles es rápido, pero aproximado. Se refina
            // solamente el último salto para fijar la textura a la pared.
            let mut before_wall = (distance - RAY_STEP).max(0.0);
            let mut inside_wall = distance;

            for _ in 0..6 {
                let middle = (before_wall + inside_wall) / 2.0;
                let middle_x = player.pos.x + middle * a.cos();
                let middle_y = player.pos.y + middle * a.sin();
                let middle_column = middle_x as usize / block_size;
                let middle_row = middle_y as usize / block_size;

                if middle_row == row && middle_column == column {
                    inside_wall = middle;
                } else {
                    before_wall = middle;
                }
            }

            let exact_distance = inside_wall;
            return Some(RayHit {
                distance: exact_distance,
                cell,
                x: player.pos.x + exact_distance * a.cos(),
                y: player.pos.y + exact_distance * a.sin(),
            });
        }

        distance += RAY_STEP;
    }
}
