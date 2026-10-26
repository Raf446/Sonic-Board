use cpal::traits::{DeviceTrait, HostTrait};
use super::{AudioDevicesResponse, DeviceInfo};

pub fn is_virtual_device_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    // Exclude local desktop audio enhancers such as FxSound from being treated as virtual mic cables.
    if lower.contains("fxsound") {
        return false;
    }
    lower.contains("vb-audio")
        || lower.contains("cable")
        || lower.contains("voicemeeter")
        || lower.contains("virtual")
        || lower.contains("blackhole")
        || lower.contains("pipewire")
        || lower.contains("loopback")
}

// Prefer WASAPI on Windows to avoid MME full-duplex and virtual sink failures.
pub fn preferred_cpal_host() -> cpal::Host {
    #[cfg(target_os = "windows")]
    {
        if let Ok(host) = cpal::host_from_id(cpal::HostId::Wasapi) {
            return host;
        }
    }
    cpal::default_host()
}

pub fn resolve_input_device(device_id: &str) -> Option<cpal::Device> {
    let host = preferred_cpal_host();
    let target_name = device_id.split("::").nth(2).unwrap_or(device_id).trim();

    if !target_name.is_empty() {
        if let Ok(mut devices) = host.input_devices() {
            if let Some(matched) = devices.find(|d| {
                d.name()
                    .map(|n| n == target_name || n == device_id || n.contains(target_name))
                    .unwrap_or(false)
            }) {
                return Some(matched);
            }
        }
    }
    host.default_input_device()
}

// Resolve the physical monitor/headphone output device strictly for soundboard sample monitoring.
pub fn resolve_monitor_output_device(monitor_device_id: &str) -> Option<cpal::Device> {
    if monitor_device_id.starts_with("out::virtual::") {
        return None;
    }
    let host = preferred_cpal_host();
    let target_name = monitor_device_id
        .split("::")
        .nth(2)
        .unwrap_or(monitor_device_id)
        .trim();

    if !target_name.is_empty() {
        if let Ok(mut devices) = host.output_devices() {
            if let Some(matched) = devices.find(|d| {
                d.name()
                    .map(|n| n == target_name || n == monitor_device_id || n.contains(target_name))
                    .unwrap_or(false)
            }) {
                return Some(matched);
            }
        }
    }
    host.default_output_device()
}

// Resolve the virtual audio cable output device (e.g. VB-Cable), strictly rejecting monitor/speaker endpoints.
pub fn resolve_virtual_mic_output_device(
    virtual_mic_id: &str,
    monitor_device_id: &str,
) -> Option<cpal::Device> {
    let trimmed_virt = virtual_mic_id.trim();
    if trimmed_virt.is_empty() || trimmed_virt.starts_with("out::virtual::") {
        return None;
    }
    if !monitor_device_id.trim().is_empty() && trimmed_virt == monitor_device_id.trim() {
        return None;
    }

    let target_name = trimmed_virt.split("::").nth(2).unwrap_or(trimmed_virt).trim();
    if target_name.is_empty() {
        return None;
    }

    let monitor_target_name = monitor_device_id
        .split("::")
        .nth(2)
        .unwrap_or(monitor_device_id)
        .trim();

    let host = preferred_cpal_host();
    if let Ok(mut devices) = host.output_devices() {
        if let Some(matched) = devices.find(|d| {
            d.name()
                .map(|n| {
                    let matches_target =
                        n == target_name || n == trimmed_virt || n.contains(target_name);
                    let is_monitor_same =
                        !monitor_target_name.is_empty() && n == monitor_target_name;
                    matches_target && !is_monitor_same && is_virtual_device_name(&n)
                })
                .unwrap_or(false)
        }) {
            return Some(matched);
        }
    }
    None
}

pub fn enumerate_audio_devices() -> AudioDevicesResponse {
    let host = preferred_cpal_host();
    let host_name = format!("{:?}", host.id());

    let default_in_name = host
        .default_input_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let default_out_name = host
        .default_output_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    let mut inputs = Vec::new();
    if let Ok(in_devices) = host.input_devices() {
        for (idx, dev) in in_devices.enumerate() {
            let name = dev.name().unwrap_or_else(|_| format!("Audio Input {}", idx + 1));
            let (sample_rate, channels) = dev
                .default_input_config()
                .map(|c| (c.sample_rate().0, c.channels()))
                .unwrap_or((48000, 2));
            let is_default = !default_in_name.is_empty() && name == default_in_name;
            let is_virtual = is_virtual_device_name(&name);
            inputs.push(DeviceInfo {
                id: format!("in::{idx}::{name}"),
                name,
                kind: "input".to_string(),
                is_default,
                is_virtual,
                host_api: host_name.clone(),
                sample_rate,
                channels,
            });
        }
    }

    let mut outputs = Vec::new();
    if let Ok(out_devices) = host.output_devices() {
        for (idx, dev) in out_devices.enumerate() {
            let name = dev.name().unwrap_or_else(|_| format!("Audio Output {}", idx + 1));
            let (sample_rate, channels) = dev
                .default_output_config()
                .map(|c| (c.sample_rate().0, c.channels()))
                .unwrap_or((48000, 2));
            let is_default = !default_out_name.is_empty() && name == default_out_name;
            let is_virtual = is_virtual_device_name(&name);
            outputs.push(DeviceInfo {
                id: format!("out::{idx}::{name}"),
                name,
                kind: "output".to_string(),
                is_default,
                is_virtual,
                host_api: host_name.clone(),
                sample_rate,
                channels,
            });
        }
    }

    if inputs.is_empty() {
        inputs.push(DeviceInfo {
            id: "in::default::System Default Microphone".to_string(),
            name: "System Default Microphone (Fallback)".to_string(),
            kind: "input".to_string(),
            is_default: true,
            is_virtual: false,
            host_api: host_name.clone(),
            sample_rate: 48000,
            channels: 1,
        });
    }

    if outputs.is_empty() {
        outputs.push(DeviceInfo {
            id: "out::default::System Default Headphones".to_string(),
            name: "System Default Headphones (Fallback)".to_string(),
            kind: "output".to_string(),
            is_default: true,
            is_virtual: false,
            host_api: host_name.clone(),
            sample_rate: 48000,
            channels: 2,
        });
    }

    // Expose a simulated virtual cable option when no physical VB-Cable driver is present.
    let has_virtual_out = outputs.iter().any(|d| d.is_virtual);
    if !has_virtual_out {
        outputs.push(DeviceInfo {
            id: "out::virtual::CABLE Input (VB-Audio Virtual Cable - Simulated Bridge)".to_string(),
            name: "CABLE Input (VB-Audio Virtual Cable: Simulated Bridge)".to_string(),
            kind: "output".to_string(),
            is_default: false,
            is_virtual: true,
            host_api: host_name,
            sample_rate: 48000,
            channels: 2,
        });
    }

    AudioDevicesResponse { inputs, outputs }
}
