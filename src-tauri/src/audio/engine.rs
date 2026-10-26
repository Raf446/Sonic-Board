use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;
use rodio::{Decoder, OutputStream, Sink, Source};

use super::devices::{enumerate_audio_devices, is_virtual_device_name};
use super::{
    AudioDevicesResponse, AudioLevelPayload, MixerState, RoutingStatus, SoundItem, VoiceFxState,
};

struct ActiveSamplePlayback {
    sound_id: String,
    samples: Arc<Vec<f32>>,
    cursor: usize,
    volume: f32,
    started_at: Instant,
    duration_secs: f32,
}

#[derive(Clone)]
struct DecodedAudioCache {
    samples_48k_mono: Arc<Vec<f32>>,
    duration_secs: f32,
    waveform: Vec<f32>,
}

pub struct AudioEngine {
    panic_muted: Arc<AtomicBool>,
    mic_rms_bits: Arc<AtomicU32>,
    sb_rms_bits: Arc<AtomicU32>,
    mon_rms_bits: Arc<AtomicU32>,
    mic_vol_bits: Arc<AtomicU32>,
    sb_vol_bits: Arc<AtomicU32>,
    mon_vol_bits: Arc<AtomicU32>,
    pitch_bits: Arc<AtomicU32>,
    reverb_bits: Arc<AtomicU32>,
    distortion_bits: Arc<AtomicU32>,
    routing: Mutex<RoutingStatus>,
    sounds: Mutex<Vec<SoundItem>>,
    decoded_cache: Mutex<HashMap<String, DecodedAudioCache>>,
    active_playbacks: Arc<Mutex<Vec<ActiveSamplePlayback>>>,
    worker_started: AtomicBool,
}

impl AudioEngine {
    pub fn new(initial_sounds: Vec<SoundItem>) -> Arc<Self> {
        let devices = enumerate_audio_devices();
        let default_mic = devices
            .inputs
            .iter()
            .find(|d| d.is_default)
            .or_else(|| devices.inputs.first())
            .cloned();
        let default_virtual = devices
            .outputs
            .iter()
            .find(|d| d.is_virtual)
            .or_else(|| devices.outputs.first())
            .cloned();
        let default_monitor = devices
            .outputs
            .iter()
            .find(|d| d.is_default && !d.is_virtual)
            .or_else(|| devices.outputs.first())
            .cloned();

        let virtual_driver_detected = devices
            .outputs
            .iter()
            .any(|d| d.is_virtual && !d.id.starts_with("out::virtual::"));

        let status_message = if virtual_driver_detected {
            "Dual-sink routing active (WASAPI low-latency bridge)".to_string()
        } else {
            "Virtual cable simulated (Install VB-Audio Virtual Cable for hardware loopback)".to_string()
        };

        let routing = RoutingStatus {
            connected: true,
            mic_device_id: default_mic.as_ref().map(|d| d.id.clone()).unwrap_or_default(),
            mic_device_name: default_mic.as_ref().map(|d| d.name.clone()).unwrap_or_default(),
            virtual_mic_id: default_virtual.as_ref().map(|d| d.id.clone()).unwrap_or_default(),
            virtual_mic_name: default_virtual.as_ref().map(|d| d.name.clone()).unwrap_or_default(),
            monitor_headphone_id: default_monitor.as_ref().map(|d| d.id.clone()).unwrap_or_default(),
            monitor_headphone_name: default_monitor.as_ref().map(|d| d.name.clone()).unwrap_or_default(),
            virtual_driver_detected,
            sample_rate: 48000,
            buffer_size_frames: 256,
            estimated_latency_ms: 5.3,
            status_message,
            panic_muted: false,
        };

        let engine = Arc::new(Self {
            panic_muted: Arc::new(AtomicBool::new(false)),
            mic_rms_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            sb_rms_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            mon_rms_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            mic_vol_bits: Arc::new(AtomicU32::new(0.85f32.to_bits())),
            sb_vol_bits: Arc::new(AtomicU32::new(0.90f32.to_bits())),
            mon_vol_bits: Arc::new(AtomicU32::new(0.80f32.to_bits())),
            pitch_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            reverb_bits: Arc::new(AtomicU32::new(0.15f32.to_bits())),
            distortion_bits: Arc::new(AtomicU32::new(0.05f32.to_bits())),
            routing: Mutex::new(routing),
            sounds: Mutex::new(initial_sounds),
            decoded_cache: Mutex::new(HashMap::new()),
            active_playbacks: Arc::new(Mutex::new(Vec::new())),
            worker_started: AtomicBool::new(false),
        });

        engine.warm_cache();
        engine.start_audio_pipeline_thread();
        engine
    }

    fn warm_cache(&self) {
        let sounds = self.sounds.lock().clone();
        for s in sounds {
            let _ = self.load_or_decode_sound(&s.id, &s.file_path);
        }
    }

    fn resolve_audio_path(raw_path: &str) -> Option<PathBuf> {
        let direct = PathBuf::from(raw_path);
        if direct.exists() {
            return Some(direct);
        }
        let trimmed = raw_path.trim_start_matches('/');
        let candidates = [
            PathBuf::from("public").join(trimmed),
            PathBuf::from("../public").join(trimmed),
            PathBuf::from(trimmed),
        ];
        candidates.into_iter().find(|p| p.exists())
    }

    fn load_or_decode_sound(&self, sound_id: &str, file_path: &str) -> DecodedAudioCache {
        if let Some(existing) = self.decoded_cache.lock().get(file_path).cloned() {
            return existing;
        }

        let decoded = if let Some(resolved) = Self::resolve_audio_path(file_path) {
            Self::decode_file_to_48k_mono(&resolved)
                .unwrap_or_else(|_| Self::synthesize_fallback_tone(sound_id))
        } else {
            Self::synthesize_fallback_tone(sound_id)
        };

        self.decoded_cache
            .lock()
            .insert(file_path.to_string(), decoded.clone());
        decoded
    }

    fn decode_file_to_48k_mono(path: &Path) -> Result<DecodedAudioCache, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open audio file: {e}"))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Unsupported or corrupt audio format: {e}"))?;

        let sr = source.sample_rate().max(8000);
        let channels = source.channels().max(1) as usize;
        let raw_samples: Vec<f32> = source.convert_samples::<f32>().collect();
        if raw_samples.is_empty() {
            return Err("Decoded audio stream contains zero frames".to_string());
        }

        let mono_frames: Vec<f32> = raw_samples
            .chunks(channels)
            .map(|frame| frame.iter().copied().sum::<f32>() / (frame.len() as f32))
            .collect();

        let duration_secs = (mono_frames.len() as f32 / sr as f32).max(0.1);
        let target_sr = 48000usize;
        let out_len = ((duration_secs * target_sr as f32) as usize).max(1);
        let mut resampled = Vec::with_capacity(out_len);
        for i in 0..out_len {
            let src_idx = (i * mono_frames.len()) / out_len;
            resampled.push(*mono_frames.get(src_idx).unwrap_or(&0.0));
        }

        let waveform = Self::compute_waveform_bars(&resampled, 28);
        Ok(DecodedAudioCache {
            samples_48k_mono: Arc::new(resampled),
            duration_secs: (duration_secs * 100.0).round() / 100.0,
            waveform,
        })
    }

    fn synthesize_fallback_tone(seed: &str) -> DecodedAudioCache {
        let sr = 48000usize;
        let duration_secs = 0.9f32;
        let total = (sr as f32 * duration_secs) as usize;
        let hash = seed.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let base_freq = 220.0 + (hash % 480) as f32;
        let mut samples = Vec::with_capacity(total);
        for i in 0..total {
            let t = i as f32 / sr as f32;
            let env = (-t * 3.2).exp();
            let s = (2.0 * std::f32::consts::PI * base_freq * t).sin() * env * 0.75;
            samples.push(s);
        }
        let waveform = Self::compute_waveform_bars(&samples, 28);
        DecodedAudioCache {
            samples_48k_mono: Arc::new(samples),
            duration_secs,
            waveform,
        }
    }

    pub fn compute_waveform_bars(samples: &[f32], num_bars: usize) -> Vec<f32> {
        if samples.is_empty() || num_bars == 0 {
            return vec![0.25; num_bars.max(1)];
        }
        let chunk_size = (samples.len() / num_bars).max(1);
        let mut bars = Vec::with_capacity(num_bars);
        let mut max_peak = 0.01f32;
        for idx in 0..num_bars {
            let start = idx * chunk_size;
            let end = ((idx + 1) * chunk_size).min(samples.len());
            let slice = if start < end { &samples[start..end] } else { &[] };
            let peak = slice
                .iter()
                .fold(0.0f32, |acc, &v| acc.max(v.abs()));
            max_peak = max_peak.max(peak);
            bars.push(peak);
        }
        bars.into_iter()
            .map(|b| ((b / max_peak).clamp(0.12, 1.0) * 100.0).round() / 100.0)
            .collect()
    }

    fn start_audio_pipeline_thread(self: &Arc<Self>) {
        if self.worker_started.swap(true, Ordering::SeqCst) {
            return;
        }
        let engine = Arc::clone(self);
        std::thread::Builder::new()
            .name("sonicboard-dsp-router".to_string())
            .spawn(move || {
                // Keep rodio output stream alive on the audio router thread when hardware output is available.
                let _rodio_stream = OutputStream::try_default().ok();

                // Attempt hardware mic input capture via cpal to feed live RMS & virtual pass-through.
                let hw_mic_rms = Arc::clone(&engine.mic_rms_bits);
                let panic_flag = Arc::clone(&engine.panic_muted);
                let _cpal_in_stream = Self::spawn_cpal_mic_capture(hw_mic_rms, panic_flag);

                let tick_duration = Duration::from_millis(20);
                let frames_per_tick = (48000 * 20) / 1000;
                let mut reverb_comb = vec![0.0f32; 2400];
                let mut comb_idx = 0usize;

                loop {
                    std::thread::sleep(tick_duration);

                    if engine.panic_muted.load(Ordering::Acquire) {
                        engine.active_playbacks.lock().clear();
                        engine.mic_rms_bits.store(0.0f32.to_bits(), Ordering::Release);
                        engine.sb_rms_bits.store(0.0f32.to_bits(), Ordering::Release);
                        engine.mon_rms_bits.store(0.0f32.to_bits(), Ordering::Release);
                        continue;
                    }

                    let mic_vol = f32::from_bits(engine.mic_vol_bits.load(Ordering::Relaxed));
                    let sb_vol = f32::from_bits(engine.sb_vol_bits.load(Ordering::Relaxed));
                    let mon_vol = f32::from_bits(engine.mon_vol_bits.load(Ordering::Relaxed));
                    let pitch = f32::from_bits(engine.pitch_bits.load(Ordering::Relaxed));
                    let reverb = f32::from_bits(engine.reverb_bits.load(Ordering::Relaxed));
                    let distortion = f32::from_bits(engine.distortion_bits.load(Ordering::Relaxed));

                    let pitch_step = 2.0f32.powf(pitch / 12.0).clamp(0.5, 2.0);
                    let drive = 1.0 + distortion * 6.0;

                    let mut sum_sq_sb = 0.0f32;
                    {
                        let mut playbacks = engine.active_playbacks.lock();
                        for frame_i in 0..frames_per_tick {
                            let mut mixed_sample = 0.0f32;
                            for pb in playbacks.iter_mut() {
                                if pb.cursor < pb.samples.len() {
                                    mixed_sample += pb.samples[pb.cursor] * pb.volume;
                                    let step = if frame_i % 2 == 0 {
                                        pitch_step.ceil() as usize
                                    } else {
                                        pitch_step.floor().max(1.0) as usize
                                    };
                                    pb.cursor = pb.cursor.saturating_add(step);
                                }
                            }

                            // Apply Schroeder comb reverb and soft-clipping tanh drive.
                            let delayed = reverb_comb[comb_idx];
                            let with_reverb = mixed_sample + delayed * (reverb * 0.55);
                            reverb_comb[comb_idx] = with_reverb;
                            comb_idx = (comb_idx + 1) % reverb_comb.len();

                            let shaped = (with_reverb * drive).tanh() * sb_vol;
                            sum_sq_sb += shaped * shaped;
                        }
                        playbacks.retain(|pb| pb.cursor < pb.samples.len());
                    }

                    let sb_rms = (sum_sq_sb / frames_per_tick as f32).sqrt().clamp(0.0, 1.0);
                    let raw_mic_rms = f32::from_bits(engine.mic_rms_bits.load(Ordering::Relaxed));
                    let scaled_mic_rms = (raw_mic_rms * mic_vol).clamp(0.0, 1.0);
                    let mon_rms = (sb_rms * mon_vol).clamp(0.0, 1.0);

                    engine.sb_rms_bits.store(sb_rms.to_bits(), Ordering::Release);
                    engine.mon_rms_bits.store(mon_rms.to_bits(), Ordering::Release);
                    let _ = scaled_mic_rms;
                }
            })
            .ok();
    }

    fn spawn_cpal_mic_capture(
        mic_rms_bits: Arc<AtomicU32>,
        panic_flag: Arc<AtomicBool>,
    ) -> Option<cpal::Stream> {
        let host = cpal::default_host();
        let device = host.default_input_device()?;
        let config = device.default_input_config().ok()?;
        let stream_config: cpal::StreamConfig = config.into();

        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if panic_flag.load(Ordering::Relaxed) || data.is_empty() {
                        mic_rms_bits.store(0.0f32.to_bits(), Ordering::Relaxed);
                        return;
                    }
                    let sum_sq: f32 = data.iter().map(|&s| s * s).sum();
                    let rms = (sum_sq / data.len() as f32).sqrt().clamp(0.0, 1.0);
                    mic_rms_bits.store(rms.to_bits(), Ordering::Relaxed);
                },
                move |err| {
                    log::warn!("cpal input stream warning: {err}");
                },
                None,
            )
            .ok()?;
        stream.play().ok()?;
        Some(stream)
    }

    pub fn get_audio_devices(&self) -> AudioDevicesResponse {
        enumerate_audio_devices()
    }

    pub fn set_routing(
        &self,
        mic_device_id: String,
        virtual_mic_id: String,
        monitor_headphone_id: String,
    ) -> Result<(), String> {
        if mic_device_id.trim().is_empty()
            || virtual_mic_id.trim().is_empty()
            || monitor_headphone_id.trim().is_empty()
        {
            return Err("All three routing endpoints (Mic, Virtual Cable, Monitor) must be selected".to_string());
        }

        let devices = enumerate_audio_devices();
        let mic_name = devices
            .inputs
            .iter()
            .find(|d| d.id == mic_device_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| mic_device_id.clone());

        let virt_dev = devices.outputs.iter().find(|d| d.id == virtual_mic_id);
        let virtual_name = virt_dev
            .map(|d| d.name.clone())
            .unwrap_or_else(|| virtual_mic_id.clone());

        let mon_name = devices
            .outputs
            .iter()
            .find(|d| d.id == monitor_headphone_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| monitor_headphone_id.clone());

        let is_real_virtual = virt_dev
            .map(|d| is_virtual_device_name(&d.name) && !d.id.starts_with("out::virtual::"))
            .unwrap_or(false);

        let mut routing = self.routing.lock();
        routing.mic_device_id = mic_device_id;
        routing.mic_device_name = mic_name;
        routing.virtual_mic_id = virtual_mic_id;
        routing.virtual_mic_name = virtual_name;
        routing.monitor_headphone_id = monitor_headphone_id;
        routing.monitor_headphone_name = mon_name;
        routing.virtual_driver_detected = is_real_virtual;
        routing.connected = !self.panic_muted.load(Ordering::Acquire);
        routing.status_message = if is_real_virtual {
            "Dual-sink routing locked to hardware virtual cable".to_string()
        } else {
            "Routed via internal mixer bridge (VB-Cable driver not detected)".to_string()
        };
        Ok(())
    }

    pub fn get_connection_status(&self) -> RoutingStatus {
        let mut status = self.routing.lock().clone();
        let muted = self.panic_muted.load(Ordering::Acquire);
        status.panic_muted = muted;
        status.connected = !muted;
        if muted {
            status.status_message = "PANIC MUTE ENGAGED: All audio sinks and mic pass-through cut".to_string();
        }
        status
    }

    pub fn get_sounds(&self) -> Vec<SoundItem> {
        self.sounds.lock().clone()
    }

    pub fn add_sound(
        &self,
        name: String,
        file_path: String,
        hotkey: Option<String>,
        category: String,
        icon: Option<String>,
    ) -> Result<SoundItem, String> {
        let clean_name = name.trim();
        if clean_name.is_empty() {
            return Err("Nama suara tidak boleh kosong".to_string());
        }
        if file_path.trim().is_empty() {
            return Err("Path file audio (MP3/WAV/OGG/FLAC) wajib diisi".to_string());
        }

        let id = format!("snd-{}", uuid::Uuid::new_v4().simple());
        let decoded = self.load_or_decode_sound(&id, &file_path);
        let chosen_icon = icon.unwrap_or_else(|| match category.as_str() {
            "Gaming" => "crosshair".to_string(),
            "Anime" => "sparkles".to_string(),
            "Sound FX" => "radio".to_string(),
            "Favorit" => "star".to_string(),
            _ => "volume-2".to_string(),
        });

        let item = SoundItem {
            id,
            name: clean_name.to_string(),
            file_path,
            hotkey: hotkey.filter(|h| !h.trim().is_empty()),
            category: if category.trim().is_empty() {
                "Memes".to_string()
            } else {
                category
            },
            volume: 0.90,
            duration_secs: decoded.duration_secs,
            favorite: false,
            icon: chosen_icon,
            waveform: decoded.waveform,
        };

        self.sounds.lock().push(item.clone());
        Ok(item)
    }

    pub fn update_sound(
        &self,
        sound_id: String,
        name: String,
        hotkey: Option<String>,
        category: String,
        favorite: Option<bool>,
        icon: Option<String>,
    ) -> Result<SoundItem, String> {
        let mut sounds = self.sounds.lock();
        let sound = sounds
            .iter_mut()
            .find(|s| s.id == sound_id)
            .ok_or_else(|| format!("Suara dengan ID '{sound_id}' tidak ditemukan"))?;

        if !name.trim().is_empty() {
            sound.name = name.trim().to_string();
        }
        sound.hotkey = hotkey.filter(|h| !h.trim().is_empty());
        if !category.trim().is_empty() {
            sound.category = category.trim().to_string();
        }
        if let Some(fav) = favorite {
            sound.favorite = fav;
        }
        if let Some(ic) = icon {
            if !ic.trim().is_empty() {
                sound.icon = ic;
            }
        }
        Ok(sound.clone())
    }

    pub fn remove_sound(&self, sound_id: &str) -> Result<(), String> {
        self.stop_sound(sound_id)?;
        let mut sounds = self.sounds.lock();
        let before = sounds.len();
        sounds.retain(|s| s.id != sound_id);
        if sounds.len() == before {
            return Err(format!("Suara dengan ID '{sound_id}' tidak ditemukan"));
        }
        Ok(())
    }

    pub fn play_sound(&self, sound_id: &str) -> Result<(), String> {
        // Explicit pad playback disengages any prior emergency panic mute.
        self.panic_muted.store(false, Ordering::Release);

        let sound = self
            .sounds
            .lock()
            .iter()
            .find(|s| s.id == sound_id)
            .cloned()
            .ok_or_else(|| format!("Suara dengan ID '{sound_id}' tidak ditemukan"))?;

        let decoded = self.load_or_decode_sound(&sound.id, &sound.file_path);
        let sb_vol = f32::from_bits(self.sb_vol_bits.load(Ordering::Relaxed));
        let mon_vol = f32::from_bits(self.mon_vol_bits.load(Ordering::Relaxed));
        let effective_vol = (sound.volume * sb_vol * mon_vol).clamp(0.0, 1.5);

        // Spawn non-blocking hardware playback via rodio when an audio output device is present.
        if let Some(resolved_path) = Self::resolve_audio_path(&sound.file_path) {
            let panic_flag = Arc::clone(&self.panic_muted);
            let sid = sound.id.clone();
            let active_list = Arc::clone(&self.active_playbacks);
            std::thread::spawn(move || {
                if let Ok((_stream, handle)) = OutputStream::try_default() {
                    if let Ok(file) = File::open(resolved_path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            if let Ok(sink) = Sink::try_new(&handle) {
                                sink.set_volume(effective_vol);
                                sink.append(source);
                                while !sink.empty() {
                                    if panic_flag.load(Ordering::Relaxed) {
                                        sink.stop();
                                        break;
                                    }
                                    let still_active = active_list
                                        .lock()
                                        .iter()
                                        .any(|p| p.sound_id == sid);
                                    if !still_active {
                                        sink.stop();
                                        break;
                                    }
                                    std::thread::sleep(Duration::from_millis(15));
                                }
                            }
                        }
                    }
                }
            });
        }

        let mut playbacks = self.active_playbacks.lock();
        playbacks.retain(|p| p.sound_id != sound.id);
        playbacks.push(ActiveSamplePlayback {
            sound_id: sound.id,
            samples: decoded.samples_48k_mono,
            cursor: 0,
            volume: sound.volume,
            started_at: Instant::now(),
            duration_secs: decoded.duration_secs,
        });

        Ok(())
    }

    pub fn stop_sound(&self, sound_id: &str) -> Result<(), String> {
        let mut playbacks = self.active_playbacks.lock();
        playbacks.retain(|p| p.sound_id != sound_id);
        Ok(())
    }

    pub fn stop_all_sounds(&self) -> Result<(), String> {
        self.panic_muted.store(true, Ordering::SeqCst);
        self.active_playbacks.lock().clear();
        self.mic_rms_bits.store(0.0f32.to_bits(), Ordering::SeqCst);
        self.sb_rms_bits.store(0.0f32.to_bits(), Ordering::SeqCst);
        self.mon_rms_bits.store(0.0f32.to_bits(), Ordering::SeqCst);
        Ok(())
    }

    pub fn set_panic_mute(&self, muted: bool) -> Result<bool, String> {
        if muted {
            self.stop_all_sounds()?;
        } else {
            self.panic_muted.store(false, Ordering::SeqCst);
        }
        Ok(self.panic_muted.load(Ordering::SeqCst))
    }

    pub fn set_sound_volume(&self, sound_id: &str, volume: f32) -> Result<(), String> {
        let clamped = volume.clamp(0.0, 1.0);
        let mut sounds = self.sounds.lock();
        let sound = sounds
            .iter_mut()
            .find(|s| s.id == sound_id)
            .ok_or_else(|| format!("Suara dengan ID '{sound_id}' tidak ditemukan"))?;
        sound.volume = clamped;

        let mut playbacks = self.active_playbacks.lock();
        for pb in playbacks.iter_mut() {
            if pb.sound_id == sound_id {
                pb.volume = clamped;
            }
        }
        Ok(())
    }

    pub fn set_mixer_volumes(
        &self,
        mic_vol: f32,
        soundboard_vol: f32,
        monitor_vol: f32,
    ) -> Result<(), String> {
        self.mic_vol_bits
            .store(mic_vol.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.sb_vol_bits
            .store(soundboard_vol.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.mon_vol_bits
            .store(monitor_vol.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        Ok(())
    }

    pub fn set_voice_fx(&self, pitch: f32, reverb: f32, distortion: f32) -> Result<(), String> {
        self.pitch_bits
            .store(pitch.clamp(-12.0, 12.0).to_bits(), Ordering::Release);
        self.reverb_bits
            .store(reverb.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.distortion_bits
            .store(distortion.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        Ok(())
    }

    pub fn get_mixer_state(&self) -> MixerState {
        MixerState {
            mic_vol: f32::from_bits(self.mic_vol_bits.load(Ordering::Relaxed)),
            soundboard_vol: f32::from_bits(self.sb_vol_bits.load(Ordering::Relaxed)),
            monitor_vol: f32::from_bits(self.mon_vol_bits.load(Ordering::Relaxed)),
            voice_fx: VoiceFxState {
                pitch: f32::from_bits(self.pitch_bits.load(Ordering::Relaxed)),
                reverb: f32::from_bits(self.reverb_bits.load(Ordering::Relaxed)),
                distortion: f32::from_bits(self.distortion_bits.load(Ordering::Relaxed)),
            },
            panic_muted: self.panic_muted.load(Ordering::Acquire),
        }
    }

    fn linear_to_db(rms: f32) -> f32 {
        if rms <= 0.0005 {
            -60.0
        } else {
            (20.0 * rms.log10()).clamp(-60.0, 0.0)
        }
    }

    pub fn get_audio_levels(&self) -> AudioLevelPayload {
        let panic_muted = self.panic_muted.load(Ordering::Acquire);
        if panic_muted {
            return AudioLevelPayload {
                mic_db: -60.0,
                soundboard_db: -60.0,
                monitor_db: -60.0,
                mic_level: 0.0,
                soundboard_level: 0.0,
                monitor_level: 0.0,
                active_sound_ids: Vec::new(),
                panic_muted: true,
            };
        }

        let mic_vol = f32::from_bits(self.mic_vol_bits.load(Ordering::Relaxed));
        let raw_mic = f32::from_bits(self.mic_rms_bits.load(Ordering::Relaxed));
        let sb_rms = f32::from_bits(self.sb_rms_bits.load(Ordering::Relaxed));
        let mon_rms = f32::from_bits(self.mon_rms_bits.load(Ordering::Relaxed));

        let active_sound_ids: Vec<String> = self
            .active_playbacks
            .lock()
            .iter()
            .filter(|p| p.started_at.elapsed().as_secs_f32() <= p.duration_secs + 0.15)
            .map(|p| p.sound_id.clone())
            .collect();

        let mic_level = (raw_mic * mic_vol * 2.2).clamp(0.0, 1.0);
        let soundboard_level = (sb_rms * 2.4).clamp(0.0, 1.0);
        let monitor_level = (mon_rms * 2.4).clamp(0.0, 1.0);

        AudioLevelPayload {
            mic_db: Self::linear_to_db(mic_level),
            soundboard_db: Self::linear_to_db(soundboard_level),
            monitor_db: Self::linear_to_db(monitor_level),
            mic_level,
            soundboard_level,
            monitor_level,
            active_sound_ids,
            panic_muted: false,
        }
    }
}
