use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyTriggerEvent {
    pub sound_id: String,
    pub key_combination: String,
    pub timestamp_ms: u64,
}

pub struct HotkeyRegistry {
    sound_to_key: Mutex<HashMap<String, String>>,
    key_to_sound: Mutex<HashMap<String, String>>,
}

impl HotkeyRegistry {
    pub fn new(initial_bindings: Vec<(String, String)>) -> Arc<Self> {
        let mut s2k = HashMap::new();
        let mut k2s = HashMap::new();
        for (sound_id, combo) in initial_bindings {
            let normalized = Self::normalize_combo(&combo);
            if !normalized.is_empty() {
                s2k.insert(sound_id.clone(), normalized.clone());
                k2s.insert(normalized, sound_id);
            }
        }
        Arc::new(Self {
            sound_to_key: Mutex::new(s2k),
            key_to_sound: Mutex::new(k2s),
        })
    }

    pub fn normalize_combo(raw: &str) -> String {
        raw.split('+')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| {
                let lower = part.to_lowercase();
                match lower.as_str() {
                    "control" | "ctrl" => "Ctrl".to_string(),
                    "alt" | "option" => "Alt".to_string(),
                    "shift" => "Shift".to_string(),
                    "super" | "meta" | "win" | "cmd" => "Super".to_string(),
                    "space" | "spacebar" => "Space".to_string(),
                    _ if lower.starts_with("numpad") => {
                        let suffix = lower.trim_start_matches("numpad").trim();
                        format!("Num {}", suffix.to_uppercase())
                    }
                    _ if lower.starts_with("num ") => {
                        let suffix = lower.trim_start_matches("num ").trim();
                        format!("Num {}", suffix.to_uppercase())
                    }
                    _ => part.to_uppercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" + ")
    }

    pub fn register_hotkey(&self, sound_id: &str, key_combination: &str) -> Result<String, String> {
        let normalized = Self::normalize_combo(key_combination);
        if normalized.is_empty() {
            return Err("Kombinasi tombol hotkey tidak valid".to_string());
        }

        let mut s2k = self.sound_to_key.lock();
        let mut k2s = self.key_to_sound.lock();

        if let Some(existing_sound) = k2s.get(&normalized) {
            if existing_sound != sound_id {
                s2k.remove(existing_sound);
            }
        }
        if let Some(old_key) = s2k.insert(sound_id.to_string(), normalized.clone()) {
            k2s.remove(&old_key);
        }
        k2s.insert(normalized.clone(), sound_id.to_string());
        Ok(normalized)
    }

    pub fn unregister_hotkey(&self, sound_id: &str) -> Result<(), String> {
        let mut s2k = self.sound_to_key.lock();
        let mut k2s = self.key_to_sound.lock();
        if let Some(old_key) = s2k.remove(sound_id) {
            k2s.remove(&old_key);
        }
        Ok(())
    }

    pub fn lookup_sound_by_combo(&self, combo: &str) -> Option<String> {
        let normalized = Self::normalize_combo(combo);
        self.key_to_sound.lock().get(&normalized).cloned()
    }

    pub fn get_all_bindings(&self) -> HashMap<String, String> {
        self.sound_to_key.lock().clone()
    }
}

pub fn format_rdev_key(key: rdev::Key) -> Option<&'static str> {
    use rdev::Key::*;
    match key {
        Num1 => Some("1"),
        Num2 => Some("2"),
        Num3 => Some("3"),
        Num4 => Some("4"),
        Num5 => Some("5"),
        Num6 => Some("6"),
        Num7 => Some("7"),
        Num8 => Some("8"),
        Num9 => Some("9"),
        Num0 => Some("0"),
        Kp1 => Some("Num 1"),
        Kp2 => Some("Num 2"),
        Kp3 => Some("Num 3"),
        Kp4 => Some("Num 4"),
        Kp5 => Some("Num 5"),
        Kp6 => Some("Num 6"),
        Kp7 => Some("Num 7"),
        Kp8 => Some("Num 8"),
        Kp9 => Some("Num 9"),
        Kp0 => Some("Num 0"),
        F1 => Some("F1"),
        F2 => Some("F2"),
        F3 => Some("F3"),
        F4 => Some("F4"),
        F5 => Some("F5"),
        F6 => Some("F6"),
        F7 => Some("F7"),
        F8 => Some("F8"),
        F9 => Some("F9"),
        F10 => Some("F10"),
        F11 => Some("F11"),
        F12 => Some("F12"),
        KeyA => Some("A"),
        KeyB => Some("B"),
        KeyC => Some("C"),
        KeyD => Some("D"),
        KeyE => Some("E"),
        KeyF => Some("F"),
        KeyG => Some("G"),
        KeyH => Some("H"),
        KeyI => Some("I"),
        KeyJ => Some("J"),
        KeyK => Some("K"),
        KeyL => Some("L"),
        KeyM => Some("M"),
        KeyN => Some("N"),
        KeyO => Some("O"),
        KeyP => Some("P"),
        KeyQ => Some("Q"),
        KeyR => Some("R"),
        KeyS => Some("S"),
        KeyT => Some("T"),
        KeyU => Some("U"),
        KeyV => Some("V"),
        KeyW => Some("W"),
        KeyX => Some("X"),
        KeyY => Some("Y"),
        KeyZ => Some("Z"),
        Space => Some("Space"),
        _ => None,
    }
}

