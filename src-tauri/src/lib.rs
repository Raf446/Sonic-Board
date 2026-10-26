pub mod audio;
pub mod hotkeys;
pub mod store;

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use parking_lot::Mutex;
use tauri::{Emitter, State};

use audio::engine::AudioEngine;
use audio::{
    AudioDevicesResponse, AudioLevelPayload, MixerState, RoutingStatus, SoundItem,
};
use hotkeys::{format_rdev_key, HotkeyRegistry, HotkeyTriggerEvent};
use store::{load_config, save_config, PersistedConfig};

pub struct AppState {
    pub engine: Arc<AudioEngine>,
    pub hotkeys: Arc<HotkeyRegistry>,
    pub config: Mutex<PersistedConfig>,
}

impl AppState {
    pub fn new() -> Self {
        let cfg = load_config();
        let initial_bindings: Vec<(String, String)> = cfg
            .sounds
            .iter()
            .filter_map(|s| s.hotkey.as_ref().map(|hk| (s.id.clone(), hk.clone())))
            .collect();

        let engine = AudioEngine::new(cfg.sounds.clone());
        let _ = engine.set_mixer_volumes(cfg.mic_vol, cfg.soundboard_vol, cfg.monitor_vol);
        let _ = engine.set_voice_fx(cfg.pitch, cfg.reverb, cfg.distortion);
        if let (Some(mic), Some(virt), Some(mon)) = (
            cfg.mic_device_id.clone(),
            cfg.virtual_mic_id.clone(),
            cfg.monitor_headphone_id.clone(),
        ) {
            let _ = engine.set_routing(mic, virt, mon);
        }

        let hotkeys = HotkeyRegistry::new(initial_bindings);
        Self {
            engine,
            hotkeys,
            config: Mutex::new(cfg),
        }
    }

    pub fn sync_and_persist(&self) {
        let mut cfg = self.config.lock();
        cfg.sounds = self.engine.get_sounds();
        let routing = self.engine.get_connection_status();
        cfg.mic_device_id = Some(routing.mic_device_id);
        cfg.virtual_mic_id = Some(routing.virtual_mic_id);
        cfg.monitor_headphone_id = Some(routing.monitor_headphone_id);
        let mixer = self.engine.get_mixer_state();
        cfg.mic_vol = mixer.mic_vol;
        cfg.soundboard_vol = mixer.soundboard_vol;
        cfg.monitor_vol = mixer.monitor_vol;
        cfg.pitch = mixer.voice_fx.pitch;
        cfg.reverb = mixer.voice_fx.reverb;
        cfg.distortion = mixer.voice_fx.distortion;
        let _ = save_config(&cfg);
    }
}

#[tauri::command]
fn get_audio_devices(state: State<'_, AppState>) -> Result<AudioDevicesResponse, String> {
    Ok(state.engine.get_audio_devices())
}

#[tauri::command]
fn set_routing(
    state: State<'_, AppState>,
    mic_device_id: String,
    virtual_mic_id: String,
    monitor_headphone_id: String,
) -> Result<(), String> {
    state
        .engine
        .set_routing(mic_device_id, virtual_mic_id, monitor_headphone_id)?;
    state.sync_and_persist();
    Ok(())
}

#[tauri::command]
fn get_connection_status(state: State<'_, AppState>) -> Result<RoutingStatus, String> {
    Ok(state.engine.get_connection_status())
}

#[tauri::command]
fn get_sounds(state: State<'_, AppState>) -> Result<Vec<SoundItem>, String> {
    Ok(state.engine.get_sounds())
}

#[tauri::command]
fn get_categories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.config.lock().categories.clone())
}

#[tauri::command]
fn add_category(state: State<'_, AppState>, name: String) -> Result<Vec<String>, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Nama kategori tidak boleh kosong".to_string());
    }
    let mut cfg = state.config.lock();
    if !cfg
        .categories
        .iter()
        .any(|c| c.eq_ignore_ascii_case(trimmed))
    {
        cfg.categories.push(trimmed.to_string());
        let _ = save_config(&cfg);
    }
    Ok(cfg.categories.clone())
}

#[tauri::command]
fn add_sound(
    state: State<'_, AppState>,
    name: String,
    file_path: String,
    hotkey: Option<String>,
    category: String,
    icon: Option<String>,
) -> Result<SoundItem, String> {
    let normalized_hk = hotkey
        .as_deref()
        .map(HotkeyRegistry::normalize_combo)
        .filter(|s| !s.is_empty());

    let item = state
        .engine
        .add_sound(name, file_path, normalized_hk.clone(), category, icon)?;

    if let Some(ref hk) = normalized_hk {
        let _ = state.hotkeys.register_hotkey(&item.id, hk);
    }
    state.sync_and_persist();
    Ok(item)
}

#[tauri::command]
fn update_sound(
    state: State<'_, AppState>,
    sound_id: String,
    name: String,
    hotkey: Option<String>,
    category: String,
    favorite: Option<bool>,
    icon: Option<String>,
) -> Result<SoundItem, String> {
    let normalized_hk = hotkey
        .as_deref()
        .map(HotkeyRegistry::normalize_combo)
        .filter(|s| !s.is_empty());

    let updated = state.engine.update_sound(
        sound_id.clone(),
        name,
        normalized_hk.clone(),
        category,
        favorite,
        icon,
    )?;

    if let Some(ref hk) = normalized_hk {
        let _ = state.hotkeys.register_hotkey(&sound_id, hk);
    } else {
        let _ = state.hotkeys.unregister_hotkey(&sound_id);
    }
    state.sync_and_persist();
    Ok(updated)
}

#[tauri::command]
fn remove_sound(state: State<'_, AppState>, sound_id: String) -> Result<(), String> {
    let _ = state.hotkeys.unregister_hotkey(&sound_id);
    state.engine.remove_sound(&sound_id)?;
    state.sync_and_persist();
    Ok(())
}

#[tauri::command]
fn play_sound(state: State<'_, AppState>, sound_id: String) -> Result<(), String> {
    state.engine.play_sound(&sound_id)
}

#[tauri::command]
fn stop_sound(state: State<'_, AppState>, sound_id: String) -> Result<(), String> {
    state.engine.stop_sound(&sound_id)
}

#[tauri::command]
fn stop_all_sounds(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.stop_all_sounds()
}

#[tauri::command]
fn set_panic_mute(state: State<'_, AppState>, muted: bool) -> Result<bool, String> {
    state.engine.set_panic_mute(muted)
}

#[tauri::command]
fn set_sound_volume(
    state: State<'_, AppState>,
    sound_id: String,
    volume: f32,
) -> Result<(), String> {
    state.engine.set_sound_volume(&sound_id, volume)?;
    state.sync_and_persist();
    Ok(())
}

#[tauri::command]
fn register_hotkey(
    state: State<'_, AppState>,
    sound_id: String,
    key_combination: String,
) -> Result<(), String> {
    let normalized = state
        .hotkeys
        .register_hotkey(&sound_id, &key_combination)?;

    let existing = state
        .engine
        .get_sounds()
        .into_iter()
        .find(|s| s.id == sound_id)
        .ok_or_else(|| format!("Suara dengan ID '{sound_id}' tidak ditemukan"))?;

    state.engine.update_sound(
        sound_id,
        existing.name,
        Some(normalized),
        existing.category,
        Some(existing.favorite),
        Some(existing.icon),
    )?;
    state.sync_and_persist();
    Ok(())
}

#[tauri::command]
fn unregister_hotkey(state: State<'_, AppState>, sound_id: String) -> Result<(), String> {
    state.hotkeys.unregister_hotkey(&sound_id)?;
    if let Some(existing) = state
        .engine
        .get_sounds()
        .into_iter()
        .find(|s| s.id == sound_id)
    {
        let _ = state.engine.update_sound(
            sound_id,
            existing.name,
            None,
            existing.category,
            Some(existing.favorite),
            Some(existing.icon),
        );
    }
    state.sync_and_persist();
    Ok(())
}

#[tauri::command]
fn set_mixer_volumes(
    state: State<'_, AppState>,
    mic_vol: f32,
    soundboard_vol: f32,
    monitor_vol: f32,
) -> Result<(), String> {
    state
        .engine
        .set_mixer_volumes(mic_vol, soundboard_vol, monitor_vol)?;
    state.sync_and_persist();
    Ok(())
}

#[tauri::command]
fn set_voice_fx(
    state: State<'_, AppState>,
    pitch: f32,
    reverb: f32,
    distortion: f32,
) -> Result<(), String> {
    state.engine.set_voice_fx(pitch, reverb, distortion)?;
    state.sync_and_persist();
    Ok(())
}

#[tauri::command]
fn get_mixer_state(state: State<'_, AppState>) -> Result<MixerState, String> {
    Ok(state.engine.get_mixer_state())
}

#[tauri::command]
fn get_audio_levels(state: State<'_, AppState>) -> Result<AudioLevelPayload, String> {
    Ok(state.engine.get_audio_levels())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();
    let engine_for_meters = Arc::clone(&app_state.engine);
    let engine_for_hotkeys = Arc::clone(&app_state.engine);
    let hotkeys_for_daemon = Arc::clone(&app_state.hotkeys);

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(app_state)
        .setup(move |app| {
            let meter_handle = app.handle().clone();
            std::thread::Builder::new()
                .name("sonicboard-meter-emitter".to_string())
                .spawn(move || loop {
                    std::thread::sleep(Duration::from_millis(50));
                    let payload = engine_for_meters.get_audio_levels();
                    let _ = meter_handle.emit("audio-levels", payload);
                })
                .ok();

            let hk_handle = app.handle().clone();
            std::thread::Builder::new()
                .name("sonicboard-hotkey-daemon".to_string())
                .spawn(move || {
                    let mut ctrl = false;
                    let mut alt = false;
                    let mut shift = false;
                    let _ = rdev::listen(move |event| match event.event_type {
                        rdev::EventType::KeyPress(key) => match key {
                            rdev::Key::ControlLeft | rdev::Key::ControlRight => ctrl = true,
                            rdev::Key::Alt | rdev::Key::AltGr => alt = true,
                            rdev::Key::ShiftLeft | rdev::Key::ShiftRight => shift = true,
                            other => {
                                if let Some(label) = format_rdev_key(other) {
                                    let mut parts = Vec::new();
                                    if ctrl {
                                        parts.push("Ctrl");
                                    }
                                    if alt {
                                        parts.push("Alt");
                                    }
                                    if shift {
                                        parts.push("Shift");
                                    }
                                    parts.push(label);
                                    let combo = parts.join(" + ");
                                    if let Some(sound_id) =
                                        hotkeys_for_daemon.lookup_sound_by_combo(&combo)
                                    {
                                        let _ = engine_for_hotkeys.play_sound(&sound_id);
                                        let ts = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .map(|d| d.as_millis() as u64)
                                            .unwrap_or(0);
                                        let _ = hk_handle.emit(
                                            "hotkey-triggered",
                                            HotkeyTriggerEvent {
                                                sound_id,
                                                key_combination: combo,
                                                timestamp_ms: ts,
                                            },
                                        );
                                    }
                                }
                            }
                        },
                        rdev::EventType::KeyRelease(key) => match key {
                            rdev::Key::ControlLeft | rdev::Key::ControlRight => ctrl = false,
                            rdev::Key::Alt | rdev::Key::AltGr => alt = false,
                            rdev::Key::ShiftLeft | rdev::Key::ShiftRight => shift = false,
                            _ => {}
                        },
                        _ => {}
                    });
                })
                .ok();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_audio_devices,
            set_routing,
            get_connection_status,
            get_sounds,
            get_categories,
            add_category,
            add_sound,
            update_sound,
            remove_sound,
            play_sound,
            stop_sound,
            stop_all_sounds,
            set_panic_mute,
            set_sound_volume,
            register_hotkey,
            unregister_hotkey,
            set_mixer_volumes,
            set_voice_fx,
            get_mixer_state,
            get_audio_levels,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SonicBoard Tauri application");
}

#[cfg(test)]
mod tests {
    use crate::audio::engine::AudioEngine;
    use crate::hotkeys::HotkeyRegistry;
    use crate::store::default_starter_sounds;

    #[test]
    fn test_audio_devices_and_routing() {
        let engine = AudioEngine::new(default_starter_sounds());
        let devices = engine.get_audio_devices();
        assert!(!devices.inputs.is_empty());
        assert!(!devices.outputs.is_empty());

        let mic_id = devices.inputs[0].id.clone();
        let virt_id = devices
            .outputs
            .iter()
            .find(|d| d.is_virtual)
            .unwrap_or(&devices.outputs[0])
            .id
            .clone();
        let mon_id = devices.outputs[0].id.clone();

        engine
            .set_routing(mic_id.clone(), virt_id.clone(), mon_id.clone())
            .expect("routing should succeed");

        let status = engine.get_connection_status();
        assert!(status.connected);
        assert_eq!(status.mic_device_id, mic_id);
        assert_eq!(status.virtual_mic_id, virt_id);
        assert_eq!(status.monitor_headphone_id, mon_id);
    }

    #[test]
    fn test_sound_playback_and_panic_mute() {
        let engine = AudioEngine::new(default_starter_sounds());
        let added = engine
            .add_sound(
                "Custom Horn".to_string(),
                "/samples/airhorn-triple.wav".to_string(),
                Some("Ctrl + 9".to_string()),
                "Memes".to_string(),
                None,
            )
            .expect("adding sound should succeed");

        assert_eq!(added.waveform.len(), 28);
        engine
            .play_sound(&added.id)
            .expect("playing added sound should succeed");

        let levels = engine.get_audio_levels();
        assert!(levels.active_sound_ids.contains(&added.id));

        engine
            .set_sound_volume(&added.id, 0.45)
            .expect("setting sound volume should succeed");

        engine
            .stop_all_sounds()
            .expect("panic mute should drop all active playbacks");

        let muted_levels = engine.get_audio_levels();
        assert!(muted_levels.panic_muted);
        assert!(muted_levels.active_sound_ids.is_empty());
        assert_eq!(muted_levels.soundboard_level, 0.0);
    }

    #[test]
    fn test_hotkey_registration_and_lookup() {
        let registry = HotkeyRegistry::new(vec![]);
        let normalized = registry
            .register_hotkey("snd-1", "ctrl+numpad4")
            .expect("hotkey registration should succeed");
        assert_eq!(normalized, "Ctrl + Num 4");
        assert_eq!(
            registry.lookup_sound_by_combo("Ctrl + Num 4"),
            Some("snd-1".to_string())
        );

        registry
            .unregister_hotkey("snd-1")
            .expect("unregister should succeed");
        assert_eq!(registry.lookup_sound_by_combo("Ctrl + Num 4"), None);
    }
}
