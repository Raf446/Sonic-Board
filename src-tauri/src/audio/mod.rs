pub mod devices;
pub mod engine;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub is_default: bool,
    pub is_virtual: bool,
    pub host_api: String,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevicesResponse {
    pub inputs: Vec<DeviceInfo>,
    pub outputs: Vec<DeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingStatus {
    pub connected: bool,
    pub mic_device_id: String,
    pub mic_device_name: String,
    pub virtual_mic_id: String,
    pub virtual_mic_name: String,
    pub monitor_headphone_id: String,
    pub monitor_headphone_name: String,
    pub virtual_driver_detected: bool,
    pub sample_rate: u32,
    pub buffer_size_frames: u32,
    pub estimated_latency_ms: f32,
    pub status_message: String,
    pub panic_muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundItem {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub hotkey: Option<String>,
    pub category: String,
    pub volume: f32,
    pub duration_secs: f32,
    pub favorite: bool,
    pub icon: String,
    pub waveform: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceFxState {
    pub pitch: f32,
    pub reverb: f32,
    pub distortion: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MixerState {
    pub mic_vol: f32,
    pub soundboard_vol: f32,
    pub monitor_vol: f32,
    pub voice_fx: VoiceFxState,
    pub panic_muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioLevelPayload {
    pub mic_db: f32,
    pub soundboard_db: f32,
    pub monitor_db: f32,
    pub mic_level: f32,
    pub soundboard_level: f32,
    pub monitor_level: f32,
    pub active_sound_ids: Vec<String>,
    pub panic_muted: bool,
}
