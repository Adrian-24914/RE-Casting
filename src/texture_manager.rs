use std::path::Path;

const FLOOR_COLOR: u32 = 0x2B2523;

#[derive(Clone, Copy)]
pub struct Texel {
    pub color: u32,
    pub alpha: u8,
}

pub struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<Texel>,
}

impl Texture {
    fn load(path: &Path) -> Result<Self, image::ImageError> {
        let image = image::open(path)?.into_rgba8();
        let (width, height) = image.dimensions();
        let pixels = image
            .pixels()
            .map(|pixel| Texel {
                color: ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | pixel[2] as u32,
                alpha: pixel[3],
            })
            .collect();

        Ok(Self {
            width: width as usize,
            height: height as usize,
            pixels,
        })
    }

    /// Obtiene un píxel usando coordenadas normalizadas entre 0.0 y 1.0.
    pub fn sample(&self, u: f32, v: f32) -> Texel {
        let u = u.clamp(0.0, 0.999_999);
        let v = v.clamp(0.0, 0.999_999);
        let x = (u * self.width as f32) as usize;
        let y = (v * self.height as f32) as usize;

        self.pixels[y * self.width + x]
    }
}

/// Carga las texturas una sola vez y decide qué imagen corresponde a cada
/// tipo de celda de la vista 3D.
pub struct TextureManager {
    ceiling: Texture,
    door: Texture,
    floor: Texture,
    key: Texture,
    stone: Texture,
    wall: Texture,
    wall_left: Texture,
    wall_right: Texture,
}

impl TextureManager {
    pub fn load(asset_directory: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        let directory = asset_directory.as_ref();

        Ok(Self {
            ceiling: Texture::load(&directory.join("Ceiling.png"))?,
            door: Texture::load(&directory.join("Door.png"))?,
            floor: Texture::load(&directory.join("Floor.png"))?,
            key: Texture::load(&directory.join("Key.png"))?,
            stone: Texture::load(&directory.join("Stone_P.png"))?,
            wall: Texture::load(&directory.join("Wall.png"))?,
            wall_left: Texture::load(&directory.join("Wall_L.png"))?,
            wall_right: Texture::load(&directory.join("Wall_R.png"))?,
        })
    }

    pub fn texture_for_cell(&self, cell: char) -> &Texture {
        match cell {
            '+' => &self.stone,
            'D' => &self.door,
            'g' | 'G' => &self.floor,
            'L' => &self.wall_left,
            'R' => &self.wall_right,
            '-' | '|' => &self.wall,
            _ => &self.wall,
        }
    }

    pub fn key(&self) -> &Texture {
        &self.key
    }

    /// Crea una sola imagen de fondo: techo texturizado y suelo sólido.
    pub fn static_background(&self, width: usize, height: usize) -> Vec<u32> {
        let mut background = vec![0; width * height];
        let half_height = height / 2;

        for y in 0..half_height {
            let v = y as f32 / half_height.max(1) as f32;
            for x in 0..width {
                let u = x as f32 / width.max(1) as f32;
                background[y * width + x] = self.ceiling.sample(u, v).color;
            }
        }

        background[half_height * width..].fill(FLOOR_COLOR);

        background
    }
}
