use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::audio::SoundItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedConfig {
    pub categories: Vec<String>,
    pub sounds: Vec<SoundItem>,
    pub mic_device_id: Option<String>,
    pub virtual_mic_id: Option<String>,
    pub monitor_headphone_id: Option<String>,
    pub mic_vol: f32,
    pub soundboard_vol: f32,
    pub monitor_vol: f32,
    pub pitch: f32,
    pub reverb: f32,
    pub distortion: f32,
}

impl Default for PersistedConfig {
    fn default() -> Self {
        Self {
            categories: vec![
                "Favorit".to_string(),
                "Memes".to_string(),
                "Gaming".to_string(),
                "Anime".to_string(),
                "Sound FX".to_string(),
            ],
            sounds: default_starter_sounds(),
            mic_device_id: None,
            virtual_mic_id: None,
            monitor_headphone_id: None,
            mic_vol: 0.85,
            soundboard_vol: 0.90,
            monitor_vol: 0.80,
            pitch: 0.0,
            reverb: 0.15,
            distortion: 0.05,
        }
    }
}

pub fn default_starter_sounds() -> Vec<SoundItem> {
    vec![
        SoundItem {
            id: "snd-vine-boom".to_string(),
            name: "Bass Drop Impact".to_string(),
            file_path: "/samples/vine-boom.wav".to_string(),
            hotkey: Some("Ctrl + 1".to_string()),
            category: "Memes".to_string(),
            volume: 0.95,
            duration_secs: 1.1,
            favorite: true,
            icon: "speaker".to_string(),
            waveform: vec![
                1.0, 0.96, 0.91, 0.86, 0.8, 0.74, 0.68, 0.62, 0.57, 0.51, 0.46, 0.42, 0.37, 0.34,
                0.3, 0.27, 0.24, 0.21, 0.19, 0.17, 0.16, 0.15, 0.14, 0.13, 0.13, 0.12, 0.12, 0.12,
            ],
        },
        SoundItem {
            id: "snd-clutch-siren".to_string(),
            name: "Tactical Clutch Alert".to_string(),
            file_path: "/samples/clutch-siren.wav".to_string(),
            hotkey: Some("Ctrl + 2".to_string()),
            category: "Gaming".to_string(),
            volume: 0.88,
            duration_secs: 1.2,
            favorite: true,
            icon: "crosshair".to_string(),
            waveform: vec![
                0.78, 0.92, 0.98, 0.86, 0.25, 0.74, 0.89, 0.95, 0.82, 0.22, 0.68, 0.84, 0.9, 0.76,
                0.2, 0.62, 0.75, 0.81, 0.67, 0.18, 0.52, 0.64, 0.69, 0.54, 0.15, 0.38, 0.28, 0.16,
            ],
        },
        SoundItem {
            id: "snd-anime-wow".to_string(),
            name: "Synth Chime Spark".to_string(),
            file_path: "/samples/anime-wow.wav".to_string(),
            hotkey: Some("Ctrl + 3".to_string()),
            category: "Anime".to_string(),
            volume: 0.90,
            duration_secs: 1.0,
            favorite: true,
            icon: "wand".to_string(),
            waveform: vec![
                0.35, 0.58, 0.76, 0.92, 1.0, 0.94, 0.85, 0.77, 0.69, 0.62, 0.55, 0.48, 0.43, 0.38,
                0.33, 0.29, 0.25, 0.22, 0.19, 0.17, 0.15, 0.14, 0.13, 0.12, 0.12, 0.12, 0.12, 0.12,
            ],
        },
        SoundItem {
            id: "snd-airhorn-triple".to_string(),
            name: "Stadium Airhorn".to_string(),
            file_path: "/samples/airhorn-triple.wav".to_string(),
            hotkey: Some("Num 1".to_string()),
            category: "Memes".to_string(),
            volume: 0.82,
            duration_secs: 1.3,
            favorite: false,
            icon: "megaphone".to_string(),
            waveform: vec![
                0.96, 0.98, 0.94, 0.18, 0.95, 0.99, 0.92, 0.18, 0.97, 1.0, 0.99, 0.98, 0.96, 0.94,
                0.91, 0.86, 0.78, 0.68, 0.58, 0.49, 0.4, 0.32, 0.25, 0.2, 0.16, 0.14, 0.12, 0.12,
            ],
        },
        SoundItem {
            id: "snd-victory-fanfare".to_string(),
            name: "Victory Brass Chord".to_string(),
            file_path: "/samples/victory-fanfare.wav".to_string(),
            hotkey: Some("Num 2".to_string()),
            category: "Gaming".to_string(),
            volume: 0.90,
            duration_secs: 1.25,
            favorite: false,
            icon: "trophy".to_string(),
            waveform: vec![
                0.68, 0.72, 0.7, 0.65, 0.78, 0.82, 0.79, 0.74, 0.96, 1.0, 0.98, 0.94, 0.88, 0.81,
                0.73, 0.65, 0.58, 0.5, 0.43, 0.37, 0.31, 0.26, 0.22, 0.18, 0.15, 0.13, 0.12, 0.12,
            ],
        },
        SoundItem {
            id: "snd-power-charge".to_string(),
            name: "Mecha Laser Charge".to_string(),
            file_path: "/samples/power-charge.wav".to_string(),
            hotkey: Some("Num 3".to_string()),
            category: "Anime".to_string(),
            volume: 0.86,
            duration_secs: 1.1,
            favorite: false,
            icon: "zap".to_string(),
            waveform: vec![
                0.18, 0.28, 0.42, 0.56, 0.68, 0.79, 0.86, 0.91, 0.94, 0.96, 0.98, 0.99, 1.0, 1.0,
                0.99, 0.98, 0.96, 0.92, 0.84, 0.72, 0.58, 0.45, 0.34, 0.25, 0.18, 0.14, 0.12, 0.12,
            ],
        },
        SoundItem {
            id: "snd-censor-beep".to_string(),
            name: "Broadcast 1kHz Bleep".to_string(),
            file_path: "/samples/censor-beep.wav".to_string(),
            hotkey: Some("Num 4".to_string()),
            category: "Sound FX".to_string(),
            volume: 0.78,
            duration_secs: 0.65,
            favorite: true,
            icon: "radio".to_string(),
            waveform: vec![
                0.24, 0.95, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98,
                0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.95, 0.42, 0.22, 0.15,
            ],
        },
        SoundItem {
            id: "snd-glitch-transition".to_string(),
            name: "Cyber Stutter Sweep".to_string(),
            file_path: "/samples/glitch-transition.wav".to_string(),
            hotkey: Some("Num 5".to_string()),
            category: "Sound FX".to_string(),
            volume: 0.85,
            duration_secs: 0.85,
            favorite: false,
            icon: "sliders".to_string(),
            waveform: vec![
                1.0, 0.22, 0.92, 0.2, 0.85, 0.18, 0.78, 0.18, 0.71, 0.16, 0.64, 0.15, 0.58, 0.15,
                0.52, 0.14, 0.46, 0.14, 0.4, 0.13, 0.35, 0.13, 0.3, 0.12, 0.25, 0.12, 0.2, 0.12,
            ],
        },
    ]
}

fn config_file_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("SonicBoard");
    let _ = fs::create_dir_all(&dir);
    dir.join("sonicboard-config.json")
}

pub fn load_config() -> PersistedConfig {
    let path = config_file_path();
    if let Ok(raw) = fs::read_to_string(path) {
        if let Ok(parsed) = serde_json::from_str::<PersistedConfig>(&raw) {
            if !parsed.sounds.is_empty() {
                return parsed;
            }
        }
    }
    PersistedConfig::default()
}

pub fn save_config(cfg: &PersistedConfig) -> Result<(), String> {
    let path = config_file_path();
    let serialized = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    fs::write(path, serialized).map_err(|e| format!("Failed to write config file: {e}"))
}
