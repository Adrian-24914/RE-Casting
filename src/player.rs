use minifb::{Key, Window};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;

use crate::maze::Maze;

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
}

fn can_move_to(maze: &Maze, position: &Vec2, block_size: usize, radius: f32) -> bool {
    let collision_points = [
        (position.x - radius, position.y - radius),
        (position.x + radius, position.y - radius),
        (position.x - radius, position.y + radius),
        (position.x + radius, position.y + radius),
    ];

    collision_points.iter().all(|&(x, y)| {
        if x < 0.0 || y < 0.0 {
            return false;
        }

        let column = x as usize / block_size;
        let row = y as usize / block_size;

        maze.get(row)
            .and_then(|line| line.get(column))
            .is_some_and(|cell| matches!(*cell, ' ' | 'g' | 'G'))
    })
}

pub fn process_events(window: &Window, player: &mut Player, maze: &Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 7.0;
    const ROTATION_SPEED: f32 = PI / 40.0;
    const PLAYER_RADIUS: f32 = 15.0;

    if window.is_key_down(Key::A) {
        player.a -= ROTATION_SPEED;
    }

    if window.is_key_down(Key::D) {
        player.a += ROTATION_SPEED;
    }

    let mut movement = 0.0;

    if window.is_key_down(Key::W) {
        movement += MOVE_SPEED;
    }

    if window.is_key_down(Key::S) {
        movement -= MOVE_SPEED;
    }

    let dx = movement * player.a.cos();
    let dy = movement * player.a.sin();

    let next_x = Vec2::new(player.pos.x + dx, player.pos.y);
    if can_move_to(maze, &next_x, block_size, PLAYER_RADIUS) {
        player.pos.x = next_x.x;
    }

    let next_y = Vec2::new(player.pos.x, player.pos.y + dy);
    if can_move_to(maze, &next_y, block_size, PLAYER_RADIUS) {
        player.pos.y = next_y.y;
    }
}
