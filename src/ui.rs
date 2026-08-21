use crate::framebuffer::Framebuffer;

const MENU_TOP: u32 = 0x101426;
const MENU_BOTTOM: u32 = 0x3A1420;
const SUCCESS_TOP: u32 = 0x071A16;
const SUCCESS_BOTTOM: u32 = 0x164E3D;
const GOLD: u32 = 0xE9C46A;
const WHITE: u32 = 0xF5F1E8;
const MUTED: u32 = 0xB9BBC6;
const SELECTED: u32 = 0xE76F51;

/// Pantalla de bienvenida y selector. Se dibuja con formas y una fuente 5x7
/// para no agregar dependencias ni requerir imágenes externas.
pub fn draw_menu(
    framebuffer: &mut Framebuffer,
    selected_level: usize,
    level_names: &[&str],
    music_label: &str,
) {
    draw_gradient(framebuffer, MENU_TOP, MENU_BOTTOM);
    draw_border(
        framebuffer,
        38,
        34,
        framebuffer.width - 38,
        framebuffer.height - 34,
        GOLD,
        3,
    );

    draw_centered_text(framebuffer, "RESIDENT EVIL CASTING", 78, 5, GOLD);
    draw_centered_text(framebuffer, "BIENVENIDO", 132, 3, WHITE);
    draw_centered_text(framebuffer, "SELECCIONA UN NIVEL", 190, 2, MUTED);

    for (index, name) in level_names.iter().enumerate() {
        let marker = if index == selected_level { ">" } else { " " };
        let line = format!("{marker} NIVEL {} - {name}", index + 1);
        let color = if index == selected_level {
            SELECTED
        } else {
            WHITE
        };
        draw_centered_text(framebuffer, &line, 235 + index * 42, 3, color);
    }

    draw_centered_text(framebuffer, &format!("MUSICA: {music_label}"), 345, 3, GOLD);
    draw_centered_text(
        framebuffer,
        "FLECHAS: ELEGIR  T: MUSICA  ENTER: JUGAR",
        425,
        2,
        WHITE,
    );
    draw_centered_text(framebuffer, "Q: SALIR", 468, 2, MUTED);
    draw_centered_text(
        framebuffer,
        "W/S MOVER  A/D O MOUSE GIRAR  M MAPA  ESC MENU",
        530,
        2,
        MUTED,
    );
}

pub fn draw_success(framebuffer: &mut Framebuffer, level_name: &str) {
    draw_gradient(framebuffer, SUCCESS_TOP, SUCCESS_BOTTOM);
    draw_border(
        framebuffer,
        56,
        48,
        framebuffer.width - 56,
        framebuffer.height - 48,
        GOLD,
        4,
    );

    // Estrella central: un detalle animado no es necesario aquí porque la
    // llave del nivel ya posee su propia animación vertical continua.
    draw_centered_text(framebuffer, "NIVEL COMPLETADO", 150, 5, GOLD);
    draw_centered_text(framebuffer, level_name, 245, 4, WHITE);
    draw_centered_text(framebuffer, "ENCONTRASTE LA SALIDA", 325, 3, WHITE);
    draw_centered_text(framebuffer, "ENTER: VOLVER AL MENU", 430, 2, MUTED);
    draw_centered_text(framebuffer, "Q: SALIR", 472, 2, MUTED);
}

fn draw_gradient(framebuffer: &mut Framebuffer, top: u32, bottom: u32) {
    let denominator = framebuffer.height.saturating_sub(1).max(1) as f32;
    for y in 0..framebuffer.height {
        let amount = y as f32 / denominator;
        let color = blend(top, bottom, amount);
        let row = y * framebuffer.width;
        framebuffer.buffer[row..row + framebuffer.width].fill(color);
    }
}

fn blend(start: u32, end: u32, amount: f32) -> u32 {
    let channel = |shift: u32| {
        let a = ((start >> shift) & 0xFFu32) as f32;
        let b = ((end >> shift) & 0xFFu32) as f32;
        (a + (b - a) * amount) as u32
    };
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn draw_border(
    framebuffer: &mut Framebuffer,
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    color: u32,
    thickness: usize,
) {
    fill_rect(framebuffer, left, top, right, top + thickness, color);
    fill_rect(framebuffer, left, bottom - thickness, right, bottom, color);
    fill_rect(framebuffer, left, top, left + thickness, bottom, color);
    fill_rect(framebuffer, right - thickness, top, right, bottom, color);
}

fn fill_rect(
    framebuffer: &mut Framebuffer,
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    color: u32,
) {
    let right = right.min(framebuffer.width);
    let bottom = bottom.min(framebuffer.height);
    for y in top.min(bottom)..bottom {
        let start = y * framebuffer.width + left.min(right);
        let end = y * framebuffer.width + right;
        framebuffer.buffer[start..end].fill(color);
    }
}

fn draw_centered_text(
    framebuffer: &mut Framebuffer,
    text: &str,
    y: usize,
    scale: usize,
    color: u32,
) {
    let width = text_width(text, scale);
    let x = framebuffer.width.saturating_sub(width) / 2;
    draw_text(framebuffer, text, x, y, scale, color);
}

fn text_width(text: &str, scale: usize) -> usize {
    text.chars()
        .count()
        .saturating_mul(6 * scale)
        .saturating_sub(scale)
}

fn draw_text(
    framebuffer: &mut Framebuffer,
    text: &str,
    mut x: usize,
    y: usize,
    scale: usize,
    color: u32,
) {
    for character in text.chars() {
        let glyph = glyph(character.to_ascii_uppercase());
        for (row, bits) in glyph.into_iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) != 0 {
                    fill_rect(
                        framebuffer,
                        x + column * scale,
                        y + row * scale,
                        x + (column + 1) * scale,
                        y + (row + 1) * scale,
                        color,
                    );
                }
            }
        }
        x += 6 * scale;
    }
}

/// Patrones de una fuente monoespaciada 5x7. Cada bit encendido representa
/// un píxel que luego se escala para seguir siendo legible.
fn glyph(character: char) -> [u8; 7] {
    match character {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 14],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        '>' => [16, 8, 4, 2, 4, 8, 16],
        _ => [0; 7],
    }
}
