mod audio_manager;
mod caster;
mod framebuffer;
mod map_renderer;
mod maze;
mod player;
mod renderer;
mod texture_manager;
mod ui;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Instant;

use crate::audio_manager::{AudioManager, MusicChoice};
use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, Maze};
use crate::player::{process_events, InputState, Player};
use crate::renderer::{Renderer, BLOCK_SIZE, HEIGHT, WIDTH};

const TARGET_FPS: usize = 15;

#[derive(Clone, Copy)]
struct Level {
    name: &'static str,
    file: &'static str,
}

const LEVELS: [Level; 2] = [
    Level {
        name: "MANSION",
        file: "maze.txt",
    },
    Level {
        name: "CRIPTA",
        file: "maze2.txt",
    },
];

#[derive(Clone, Copy)]
enum Screen {
    Menu,
    Playing,
    Success { level_index: usize },
}

impl Screen {
    fn title(self) -> &'static str {
        match self {
            Self::Menu => "Menu",
            Self::Playing => "Juego",
            Self::Success { .. } => "Exito",
        }
    }
}

struct Game {
    maze: Maze,
    player: Player,
    renderer: Renderer,
    input: InputState,
}

impl Game {
    fn load(level: Level) -> Self {
        let (maze, player) = load_maze(level.file, BLOCK_SIZE);
        let renderer = Renderer::new(&maze);

        Self {
            maze,
            player,
            renderer,
            input: InputState::default(),
        }
    }

    fn reached_goal(&self) -> bool {
        let column = self.player.pos.x as usize / BLOCK_SIZE;
        let row = self.player.pos.y as usize / BLOCK_SIZE;

        self.maze
            .get(row)
            .and_then(|line| line.get(column))
            .is_some_and(|cell| matches!(*cell, 'g' | 'G'))
    }
}

/// Promedia los cuadros durante aproximadamente un segundo. El resultado se
/// muestra en el título de la ventana y en consola, nunca en el framebuffer.
struct FpsCounter {
    interval_start: Instant,
    frames: u32,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            interval_start: Instant::now(),
            frames: 0,
        }
    }

    fn frame_presented(&mut self, window: &mut Window, screen: Screen) {
        self.frames += 1;
        let elapsed = self.interval_start.elapsed();

        if elapsed.as_secs_f32() >= 1.0 {
            let fps = self.frames as f32 / elapsed.as_secs_f32();
            let title = format!(
                "Resident Evil Casting | {} | FPS: {fps:.1} (meta: {TARGET_FPS})",
                screen.title()
            );
            window.set_title(&title);
            println!("FPS: {fps:.1}");

            self.interval_start = Instant::now();
            self.frames = 0;
        }
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new(
        "Resident Evil Casting | FPS: calculando...",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("no se pudo crear la ventana");

    // minifb espera dentro de update_with_buffer para sostener ~15 FPS.
    window.set_target_fps(TARGET_FPS);
    window.set_cursor_visibility(true);

    let mut audio = AudioManager::new("assets");
    let mut music = MusicChoice::Normal;
    audio.play_background_music(music);

    let level_names: Vec<&str> = LEVELS.iter().map(|level| level.name).collect();
    let mut selected_level = 0;
    let mut screen = Screen::Menu;
    let mut game: Option<Game> = None;
    let mut fps_counter = FpsCounter::new();

    while window.is_open() {
        match screen {
            Screen::Menu => {
                if window.is_key_pressed(Key::Q, KeyRepeat::No) {
                    break;
                }

                if window.is_key_pressed(Key::Up, KeyRepeat::No) {
                    selected_level = (selected_level + LEVELS.len() - 1) % LEVELS.len();
                }
                if window.is_key_pressed(Key::Down, KeyRepeat::No) {
                    selected_level = (selected_level + 1) % LEVELS.len();
                }
                if window.is_key_pressed(Key::T, KeyRepeat::No) {
                    music = music.toggle();
                    audio.play_background_music(music);
                }

                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    game = Some(Game::load(LEVELS[selected_level]));
                    screen = Screen::Playing;
                    window.set_cursor_visibility(false);
                } else {
                    ui::draw_menu(
                        &mut framebuffer,
                        selected_level,
                        &level_names,
                        music.label(),
                    );
                }
            }
            Screen::Playing => {
                if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
                    if let Some(active_game) = &mut game {
                        active_game.input.reset_mouse();
                    }
                    game = None;
                    screen = Screen::Menu;
                    window.set_cursor_visibility(true);
                    ui::draw_menu(
                        &mut framebuffer,
                        selected_level,
                        &level_names,
                        music.label(),
                    );
                } else {
                    let active_game = game.as_mut().expect("el juego debe estar cargado");
                    if window.is_key_pressed(Key::M, KeyRepeat::No) {
                        active_game.renderer.toggle_mode();
                        active_game.input.reset_mouse();
                    }

                    // El mapa funciona como una pausa: conserva la exploración
                    // sin aceptar movimiento ni rotación del jugador.
                    if !active_game.renderer.is_map_visible() {
                        let events = process_events(
                            &window,
                            &mut active_game.player,
                            &mut active_game.maze,
                            BLOCK_SIZE,
                            &mut active_game.input,
                        );

                        if events.key_collected {
                            audio.play_key_pickup();
                        }
                        if events.door_opened {
                            audio.play_door_open();
                        }
                    }

                    if active_game.reached_goal() {
                        println!("¡Meta alcanzada en {}!", LEVELS[selected_level].name);
                        screen = Screen::Success {
                            level_index: selected_level,
                        };
                        game = None;
                        window.set_cursor_visibility(true);
                        ui::draw_success(&mut framebuffer, LEVELS[selected_level].name);
                    } else {
                        active_game.renderer.render(
                            &mut framebuffer,
                            &active_game.maze,
                            &active_game.player,
                        );
                    }
                }
            }
            Screen::Success { level_index } => {
                if window.is_key_pressed(Key::Q, KeyRepeat::No) {
                    break;
                }

                if window.is_key_pressed(Key::Enter, KeyRepeat::No)
                    || window.is_key_pressed(Key::Escape, KeyRepeat::No)
                {
                    screen = Screen::Menu;
                    ui::draw_menu(
                        &mut framebuffer,
                        selected_level,
                        &level_names,
                        music.label(),
                    );
                } else {
                    ui::draw_success(&mut framebuffer, LEVELS[level_index].name);
                }
            }
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .expect("no se pudo actualizar la ventana");
        fps_counter.frame_presented(&mut window, screen);
    }
}
