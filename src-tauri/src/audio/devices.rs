use cpal::traits::{DeviceTrait, HostTrait};
use super::{AudioDevicesResponse, DeviceInfo};

pub fn is_virtual_device_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("vb-audio")
        || lower.contains("cable")
        || lower.contains("voicemeeter")
        || lower.contains("virtual")
        || lower.contains("blackhole")
        || lower.contains("pipewire")
        || lower.contains("loopback")
}

pub fn enumerate_audio_devices() -> AudioDevicesResponse {
    let host = cpal::default_host();
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

    // Always expose a virtual cable entry so users without VB-Cable installed can test routing with a fallback warning.
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
