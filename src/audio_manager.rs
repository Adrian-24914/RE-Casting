use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};

const BACKGROUND_MUSIC: &str = "Background.mp3";
const TAYLOR_MUSIC: &str = "Taylor.mp3";
const KEY_SOUND: &str = "KeyPickup.mp3";
const DOOR_SOUND: &str = "DoorOpen.mp3";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicChoice {
    Normal,
    TaylorSwift,
}

impl MusicChoice {
    pub fn toggle(self) -> Self {
        match self {
            Self::Normal => Self::TaylorSwift,
            Self::TaylorSwift => Self::Normal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::TaylorSwift => "TAYLOR SWIFT",
        }
    }
}

pub struct AudioManager {
    device: Option<MixerDeviceSink>,
    music: Option<Player>,
    assets_directory: PathBuf,
}

impl AudioManager {
    pub fn new(assets_directory: impl AsRef<Path>) -> Self {
        let device = DeviceSinkBuilder::open_default_sink()
            .map_err(|error| eprintln!("Audio desactivado: {error}"))
            .ok();

        Self {
            device,
            music: None,
            assets_directory: assets_directory.as_ref().to_path_buf(),
        }
    }

    /// Cambia inmediatamente entre los dos archivos de música disponibles.
    pub fn play_background_music(&mut self, choice: MusicChoice) {
        let Some(device) = &self.device else {
            return;
        };

        let file_name = match choice {
            MusicChoice::Normal => BACKGROUND_MUSIC,
            MusicChoice::TaylorSwift => TAYLOR_MUSIC,
        };
        let Some(source) = self.load_audio(file_name) else {
            return;
        };

        // Al descartar el Player anterior, su pista se detiene antes de
        // conectar la nueva al mezclador.
        self.music = None;
        let music = Player::connect_new(device.mixer());
        music.set_volume(0.30);
        music.append(source.repeat_infinite());
        self.music = Some(music);
    }

    pub fn play_key_pickup(&self) {
        self.play_effect(KEY_SOUND);
    }

    pub fn play_door_open(&self) {
        self.play_effect(DOOR_SOUND);
    }

    fn play_effect(&self, file_name: &str) {
        let Some(device) = &self.device else {
            return;
        };
        let Some(source) = self.load_audio(file_name) else {
            return;
        };

        let effect = Player::connect_new(device.mixer());
        effect.set_volume(0.75);
        effect.append(source);
        effect.detach();
    }

    fn load_audio(&self, file_name: &str) -> Option<Decoder<BufReader<File>>> {
        let path = self.assets_directory.join(file_name);
        let file = File::open(&path)
            .map_err(|error| eprintln!("No se pudo cargar {}: {error}", path.display()))
            .ok()?;

        Decoder::try_from(file)
            .map_err(|error| eprintln!("No se pudo leer {}: {error}", path.display()))
            .ok()
    }
}
