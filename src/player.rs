use minifb::{Key, Window};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;

use crate::maze::Maze;

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
    pub has_key: bool,
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
            .is_some_and(|cell| matches!(*cell, ' ' | 'K' | 'g' | 'G'))
    })
}

fn collect_key(maze: &mut Maze, player: &mut Player, block_size: usize) {
    let column = player.pos.x as usize / block_size;
    let row = player.pos.y as usize / block_size;

    if maze.get(row).and_then(|line| line.get(column)) == Some(&'K') {
        maze[row][column] = ' ';
        player.has_key = true;
        println!("¡Llave recogida! Ya puedes abrir la puerta.");
    }
}

fn open_touched_doors(maze: &mut Maze, position: &Vec2, block_size: usize, radius: f32) {
    let collision_points = [
        (position.x - radius, position.y - radius),
        (position.x + radius, position.y - radius),
        (position.x - radius, position.y + radius),
        (position.x + radius, position.y + radius),
    ];
    let mut opened_door = false;

    for (x, y) in collision_points {
        if x < 0.0 || y < 0.0 {
            continue;
        }

        let column = x as usize / block_size;
        let row = y as usize / block_size;
        if maze.get(row).and_then(|line| line.get(column)) == Some(&'D') {
            maze[row][column] = ' ';
            opened_door = true;
        }
    }

    if opened_door {
        println!("¡Puerta abierta con la llave!");
    }
}

pub fn process_events(window: &Window, player: &mut Player, maze: &mut Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 7.0;
    const ROTATION_SPEED: f32 = PI / 40.0;
    const PLAYER_RADIUS: f32 = 10.0;

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
    if player.has_key {
        open_touched_doors(maze, &next_x, block_size, PLAYER_RADIUS);
    }
    if can_move_to(maze, &next_x, block_size, PLAYER_RADIUS) {
        player.pos.x = next_x.x;
    }

    let next_y = Vec2::new(player.pos.x, player.pos.y + dy);
    if player.has_key {
        open_touched_doors(maze, &next_y, block_size, PLAYER_RADIUS);
    }
    if can_move_to(maze, &next_y, block_size, PLAYER_RADIUS) {
        player.pos.y = next_y.y;
    }

    collect_key(maze, player, block_size);
}
