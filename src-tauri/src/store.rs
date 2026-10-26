use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::audio::SoundItem;

const CANONICAL_SAMPLE_RATE: u32 = 48_000;

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
            reverb: 0.0,
            distortion: 0.0,
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

pub fn sounds_storage_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("SonicBoard").join("sounds");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn canonical_sound_wav_path(sound_id: &str) -> PathBuf {
    let safe_id: String = sound_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    sounds_storage_dir().join(format!("{safe_id}.wav"))
}

// Write canonical 48 kHz mono 16-bit PCM WAV to disk so custom imported sounds survive restarts intact.
pub fn persist_canonical_wav_48k_mono(
    sound_id: &str,
    samples_48k_mono: &[f32],
) -> Result<PathBuf, String> {
    let out_path = canonical_sound_wav_path(sound_id);
    let num_samples = samples_48k_mono.len();
    let data_bytes = (num_samples * 2) as u32;
    let riff_size = 36u32.saturating_add(data_bytes);
    let byte_rate = CANONICAL_SAMPLE_RATE * 2;

    let mut buf = Vec::with_capacity(44 + num_samples * 2);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&riff_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&CANONICAL_SAMPLE_RATE.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_bytes.to_le_bytes());

    for &sample in samples_48k_mono {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm_i16 = (clamped * i16::MAX as f32) as i16;
        buf.extend_from_slice(&pcm_i16.to_le_bytes());
    }

    let mut file =
        fs::File::create(&out_path).map_err(|e| format!("Failed to create canonical WAV: {e}"))?;
    file.write_all(&buf)
        .map_err(|e| format!("Failed to write canonical WAV: {e}"))?;
    Ok(out_path)
}

pub fn decode_data_uri_bytes(data_uri: &str) -> Option<Vec<u8>> {
    let trimmed = data_uri.trim();
    if !trimmed.starts_with("data:") {
        return None;
    }
    let (_, b64_part) = trimmed.split_once(";base64,")?;
    decode_base64_standard(b64_part)
}

fn decode_base64_standard(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity((input.len() * 3) / 4);
    let mut accum = 0u32;
    let mut bits = 0u8;

    for byte in input.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        if byte == b'=' {
            break;
        }
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        accum = (accum << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accum >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

pub fn normalize_persisted_sound_path(sound_id: &str, raw_path: &str) -> String {
    let trimmed = raw_path.trim();
    if trimmed.starts_with("/samples/") {
        return trimmed.to_string();
    }
    let canonical = canonical_sound_wav_path(sound_id);
    if canonical.exists() {
        return canonical.to_string_lossy().to_string();
    }
    if !trimmed.starts_with("blob:") && !trimmed.starts_with("data:") && Path::new(trimmed).exists()
    {
        return trimmed.to_string();
    }
    trimmed.to_string()
}

pub fn load_config() -> PersistedConfig {
    let path = config_file_path();
    if let Ok(raw) = fs::read_to_string(path) {
        if let Ok(mut parsed) = serde_json::from_str::<PersistedConfig>(&raw) {
            // Reset Voice FX to neutral bypass (0.0) on startup so uncalibrated DSP never corrupts playback.
            parsed.pitch = 0.0;
            parsed.reverb = 0.0;
            parsed.distortion = 0.0;

            for sound in parsed.sounds.iter_mut() {
                sound.file_path = normalize_persisted_sound_path(&sound.id, &sound.file_path);
            }

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
