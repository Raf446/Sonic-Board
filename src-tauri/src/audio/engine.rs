use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{BufferSize, SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfig};
use parking_lot::Mutex;
use rodio::{Decoder, Source};

use crate::store::{
    canonical_sound_wav_path, decode_data_uri_bytes, persist_canonical_wav_48k_mono,
};
use super::devices::{
    enumerate_audio_devices, is_virtual_device_name, resolve_input_device,
    resolve_monitor_output_device, resolve_virtual_mic_output_device,
};
use super::{
    AudioDevicesResponse, AudioLevelPayload, MixerState, RoutingStatus, SoundItem, VoiceFxState,
};

const INTERNAL_SAMPLE_RATE: u32 = 48_000;
const SAFE_WASAPI_BUFFER_FRAMES: u32 = 512;
const MAX_MIC_RING_SAMPLES: usize = 16_384;

struct ActiveSamplePlayback {
    sound_id: String,
    samples: Arc<Vec<f32>>,
    current_frame: usize,
    cursor_f64: f64,
    volume_bits: Arc<AtomicU32>,
    cancelled: Arc<AtomicBool>,
    is_finished: bool,
}

struct VirtualSinkTrack {
    sound_id: String,
    samples_48k_mono: Arc<Vec<f32>>,
    resampled_for_rate: Option<(u32, Arc<Vec<f32>>)>,
    current_frame: usize,
    cursor_f64: f64,
    volume_bits: Arc<AtomicU32>,
    cancelled: Arc<AtomicBool>,
    is_finished: bool,
}

#[derive(Clone)]
struct DecodedAudioCache {
    samples_48k_mono: Arc<Vec<f32>>,
    duration_secs: f32,
    waveform: Vec<f32>,
}

pub struct AudioEngine {
    panic_muted: Arc<AtomicBool>,
    stream_needs_recovery: Arc<AtomicBool>,
    virtual_stream_active: Arc<AtomicBool>,
    routing_generation: Arc<AtomicU32>,
    mic_rms_bits: Arc<AtomicU32>,
    sb_rms_bits: Arc<AtomicU32>,
    mon_rms_bits: Arc<AtomicU32>,
    mic_vol_bits: Arc<AtomicU32>,
    sb_vol_bits: Arc<AtomicU32>,
    mon_vol_bits: Arc<AtomicU32>,
    pitch_bits: Arc<AtomicU32>,
    reverb_bits: Arc<AtomicU32>,
    distortion_bits: Arc<AtomicU32>,
    mic_ring_buffer: Arc<Mutex<VecDeque<f32>>>,
    virtual_sb_queue: Arc<Mutex<Vec<VirtualSinkTrack>>>,
    routing: Mutex<RoutingStatus>,
    sounds: Mutex<Vec<SoundItem>>,
    sound_pcm_cache: Mutex<HashMap<String, Arc<Vec<f32>>>>,
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
            sample_rate: INTERNAL_SAMPLE_RATE,
            buffer_size_frames: SAFE_WASAPI_BUFFER_FRAMES,
            estimated_latency_ms: 10.6,
            status_message,
            panic_muted: false,
        };

        let engine = Arc::new(Self {
            panic_muted: Arc::new(AtomicBool::new(false)),
            stream_needs_recovery: Arc::new(AtomicBool::new(false)),
            virtual_stream_active: Arc::new(AtomicBool::new(false)),
            routing_generation: Arc::new(AtomicU32::new(1)),
            mic_rms_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            sb_rms_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            mon_rms_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            mic_vol_bits: Arc::new(AtomicU32::new(0.85f32.to_bits())),
            sb_vol_bits: Arc::new(AtomicU32::new(0.90f32.to_bits())),
            mon_vol_bits: Arc::new(AtomicU32::new(0.80f32.to_bits())),
            // Default Voice FX strictly to neutral bypass (0.0) on startup.
            pitch_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            reverb_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            distortion_bits: Arc::new(AtomicU32::new(0.0f32.to_bits())),
            mic_ring_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_MIC_RING_SAMPLES))),
            virtual_sb_queue: Arc::new(Mutex::new(Vec::new())),
            routing: Mutex::new(routing),
            sounds: Mutex::new(initial_sounds),
            sound_pcm_cache: Mutex::new(HashMap::new()),
            decoded_cache: Mutex::new(HashMap::new()),
            active_playbacks: Arc::new(Mutex::new(Vec::new())),
            worker_started: AtomicBool::new(false),
        });

        engine.warm_cache();
        engine.start_audio_pipeline_thread();
        engine
    }

    // Build preferred (512-1024 frame) and fallback (driver default) CPAL stream configs for WASAPI stability.
    fn preferred_and_fallback_stream_configs(
        supported: &SupportedStreamConfig,
    ) -> (StreamConfig, StreamConfig) {
        let mut preferred: StreamConfig = supported.clone().into();
        let fallback: StreamConfig = supported.clone().into();
        let target_frames = match supported.buffer_size() {
            SupportedBufferSize::Range { min, max } => {
                SAFE_WASAPI_BUFFER_FRAMES.clamp(*min, (*max).max(*min))
            }
            SupportedBufferSize::Unknown => SAFE_WASAPI_BUFFER_FRAMES,
        };
        preferred.buffer_size = BufferSize::Fixed(target_frames);
        (preferred, fallback)
    }

    // Peak-normalize 48 kHz mono PCM samples to prevent clipping while preserving dynamic range.
    pub fn normalize_pcm_48k_mono(mut samples: Vec<f32>) -> Vec<f32> {
        let peak = samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        if peak > 0.98 {
            let gain = 0.95 / peak;
            for s in samples.iter_mut() {
                *s = (*s * gain).clamp(-1.0, 1.0);
            }
        } else if peak > 0.01 && peak < 0.25 {
            let gain = (0.75 / peak).min(4.0);
            for s in samples.iter_mut() {
                *s = (*s * gain).clamp(-1.0, 1.0);
            }
        }
        samples
    }

    // Downmix interleaved multi-channel audio frames into mono f32 samples.
    pub fn downmix_to_mono(interleaved: &[f32], channels: usize) -> Vec<f32> {
        let ch = channels.max(1);
        if ch == 1 {
            return interleaved.to_vec();
        }
        interleaved
            .chunks(ch)
            .map(|frame| frame.iter().copied().sum::<f32>() / (frame.len() as f32))
            .collect()
    }

    // Linearly interpolate mono audio samples between arbitrary source and target sample rates.
    pub fn resample_linear(input: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }
        if src_rate == dst_rate || src_rate == 0 || dst_rate == 0 {
            return input.to_vec();
        }
        let ratio = src_rate as f64 / dst_rate as f64;
        let out_len = ((input.len() as f64) / ratio).ceil().max(1.0) as usize;
        let mut output = Vec::with_capacity(out_len);
        let max_idx = input.len().saturating_sub(1);

        for i in 0..out_len {
            let src_pos = (i as f64) * ratio;
            let idx0 = (src_pos.floor() as usize).min(max_idx);
            let idx1 = (idx0 + 1).min(max_idx);
            let frac = (src_pos - (idx0 as f64)) as f32;
            let sample = input[idx0] + frac * (input[idx1] - input[idx0]);
            output.push(sample);
        }
        output
    }

    // Convert mono samples into interleaved multi-channel frames (e.g. mono -> stereo duplicate).
    pub fn mono_to_interleaved_channels(mono: &[f32], channels: usize) -> Vec<f32> {
        let ch = channels.max(1);
        if ch == 1 {
            return mono.to_vec();
        }
        let mut out = Vec::with_capacity(mono.len() * ch);
        for &sample in mono {
            for _ in 0..ch {
                out.push(sample);
            }
        }
        out
    }

    fn warm_cache(&self) {
        let mut sounds = self.sounds.lock().clone();
        let mut path_updated = false;
        for s in sounds.iter_mut() {
            let (decoded, canonical_path) = self.decode_and_cache_sound(&s.id, &s.file_path);
            if let Some(new_path) = canonical_path {
                if s.file_path != new_path {
                    s.file_path = new_path;
                    path_updated = true;
                }
            }
            s.duration_secs = decoded.duration_secs;
            if s.waveform.is_empty() {
                s.waveform = decoded.waveform;
            }
        }
        if path_updated {
            *self.sounds.lock() = sounds;
        }
    }

    fn resolve_audio_path(sound_id: &str, raw_path: &str) -> Option<PathBuf> {
        let canonical = canonical_sound_wav_path(sound_id);
        if canonical.exists() && !raw_path.starts_with("/samples/") {
            return Some(canonical);
        }
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
        if let Some(found) = candidates.into_iter().find(|p| p.exists()) {
            return Some(found);
        }
        if canonical.exists() {
            return Some(canonical);
        }
        None
    }

    // Retrieve pre-decoded 48 kHz mono PCM from RAM cache, decoding once only on cache miss.
    fn load_or_decode_sound(&self, sound_id: &str, file_path: &str) -> DecodedAudioCache {
        if let Some(existing) = self.decoded_cache.lock().get(sound_id).cloned() {
            return existing;
        }
        if let Some(existing) = self.decoded_cache.lock().get(file_path).cloned() {
            self.sound_pcm_cache
                .lock()
                .insert(sound_id.to_string(), Arc::clone(&existing.samples_48k_mono));
            return existing;
        }
        self.decode_and_cache_sound(sound_id, file_path).0
    }

    fn decode_and_cache_sound(
        &self,
        sound_id: &str,
        file_path: &str,
    ) -> (DecodedAudioCache, Option<String>) {
        if let Some(existing) = self.decoded_cache.lock().get(sound_id).cloned() {
            return (existing, None);
        }

        let mut canonical_saved_path: Option<String> = None;

        let decoded = if let Some(raw_bytes) = decode_data_uri_bytes(file_path) {
            match Self::decode_bytes_to_48k_mono(raw_bytes) {
                Ok(dec) => {
                    if let Ok(saved_path) =
                        persist_canonical_wav_48k_mono(sound_id, &dec.samples_48k_mono)
                    {
                        canonical_saved_path = Some(saved_path.to_string_lossy().to_string());
                    }
                    dec
                }
                Err(_) => Self::synthesize_fallback_tone(sound_id),
            }
        } else if let Some(resolved) = Self::resolve_audio_path(sound_id, file_path) {
            match Self::decode_file_to_48k_mono(&resolved) {
                Ok(dec) => {
                    // Persist custom imported files as canonical 48 kHz mono WAV so restarts never lose or warp them.
                    if !file_path.starts_with("/samples/") {
                        let canonical_target = canonical_sound_wav_path(sound_id);
                        if resolved != canonical_target {
                            if let Ok(saved_path) =
                                persist_canonical_wav_48k_mono(sound_id, &dec.samples_48k_mono)
                            {
                                canonical_saved_path =
                                    Some(saved_path.to_string_lossy().to_string());
                            }
                        } else {
                            canonical_saved_path =
                                Some(canonical_target.to_string_lossy().to_string());
                        }
                    }
                    dec
                }
                Err(_) => Self::synthesize_fallback_tone(sound_id),
            }
        } else {
            Self::synthesize_fallback_tone(sound_id)
        };

        let cache_key_path = canonical_saved_path
            .as_deref()
            .unwrap_or(file_path)
            .to_string();

        {
            let mut pcm_map = self.sound_pcm_cache.lock();
            pcm_map.insert(sound_id.to_string(), Arc::clone(&decoded.samples_48k_mono));
            pcm_map.insert(cache_key_path.clone(), Arc::clone(&decoded.samples_48k_mono));
        }
        {
            let mut meta_map = self.decoded_cache.lock();
            meta_map.insert(sound_id.to_string(), decoded.clone());
            meta_map.insert(cache_key_path, decoded.clone());
        }

        (decoded, canonical_saved_path)
    }

    fn decode_file_to_48k_mono(path: &Path) -> Result<DecodedAudioCache, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open audio file: {e}"))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Unsupported or corrupt audio format: {e}"))?;
        Self::decode_rodio_source_to_48k_mono(source)
    }

    fn decode_bytes_to_48k_mono(bytes: Vec<u8>) -> Result<DecodedAudioCache, String> {
        let source = Decoder::new(Cursor::new(bytes))
            .map_err(|e| format!("Unsupported or corrupt uploaded audio format: {e}"))?;
        Self::decode_rodio_source_to_48k_mono(source)
    }

    fn decode_rodio_source_to_48k_mono<R>(source: Decoder<R>) -> Result<DecodedAudioCache, String>
    where
        R: std::io::Read + std::io::Seek + Send + Sync + 'static,
    {
        let sr = source.sample_rate().max(8000);
        let channels = source.channels().max(1) as usize;
        let raw_samples: Vec<f32> = source.convert_samples::<f32>().collect();
        if raw_samples.is_empty() {
            return Err("Decoded audio stream contains zero frames".to_string());
        }

        let mono_frames = Self::downmix_to_mono(&raw_samples, channels);
        let duration_secs = (mono_frames.len() as f32 / sr as f32).max(0.1);
        let resampled = Self::resample_linear(&mono_frames, sr, INTERNAL_SAMPLE_RATE);
        let normalized = Self::normalize_pcm_48k_mono(resampled);

        let waveform = Self::compute_waveform_bars(&normalized, 28);
        Ok(DecodedAudioCache {
            samples_48k_mono: Arc::new(normalized),
            duration_secs: (duration_secs * 100.0).round() / 100.0,
            waveform,
        })
    }

    fn synthesize_fallback_tone(seed: &str) -> DecodedAudioCache {
        let sr = INTERNAL_SAMPLE_RATE as usize;
        let duration_secs = 0.9f32;
        let total = (sr as f32 * duration_secs) as usize;
        let hash = seed
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
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
            let peak = slice.iter().fold(0.0f32, |acc, &v| acc.max(v.abs()));
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
                let mut active_gen = 0u32;
                let mut _cpal_in_stream: Option<cpal::Stream> = None;
                let mut _cpal_virtual_out_stream: Option<cpal::Stream> = None;
                let mut last_recovery_attempt = Instant::now() - Duration::from_secs(5);

                let tick_duration = Duration::from_millis(20);
                let frames_per_tick = (INTERNAL_SAMPLE_RATE as usize * 20) / 1000;
                let mut reverb_comb = vec![0.0f32; 2400];
                let mut comb_idx = 0usize;

                loop {
                    std::thread::sleep(tick_duration);

                    let current_gen = engine.routing_generation.load(Ordering::Acquire);
                    let needs_recovery = engine.stream_needs_recovery.load(Ordering::Acquire);

                    if (current_gen != active_gen || needs_recovery)
                        && last_recovery_attempt.elapsed() >= Duration::from_millis(500)
                    {
                        last_recovery_attempt = Instant::now();
                        engine.stream_needs_recovery.store(false, Ordering::Release);
                        active_gen = current_gen;

                        let (mic_id, virt_id, mon_id) = {
                            let r = engine.routing.lock();
                            (
                                r.mic_device_id.clone(),
                                r.virtual_mic_id.clone(),
                                r.monitor_headphone_id.clone(),
                            )
                        };

                        _cpal_in_stream = None;
                        _cpal_virtual_out_stream = None;
                        engine.virtual_stream_active.store(false, Ordering::Release);

                        _cpal_in_stream = Self::spawn_cpal_mic_capture(
                            &mic_id,
                            Arc::clone(&engine.mic_rms_bits),
                            Arc::clone(&engine.mic_ring_buffer),
                            Arc::clone(&engine.panic_muted),
                            Arc::clone(&engine.stream_needs_recovery),
                        );

                        _cpal_virtual_out_stream = Self::spawn_virtual_mic_passthrough_stream(
                            &virt_id,
                            &mon_id,
                            Arc::clone(&engine.mic_ring_buffer),
                            Arc::clone(&engine.virtual_sb_queue),
                            Arc::clone(&engine.mic_vol_bits),
                            Arc::clone(&engine.sb_vol_bits),
                            Arc::clone(&engine.pitch_bits),
                            Arc::clone(&engine.reverb_bits),
                            Arc::clone(&engine.distortion_bits),
                            Arc::clone(&engine.panic_muted),
                            Arc::clone(&engine.stream_needs_recovery),
                            Arc::clone(&engine.virtual_stream_active),
                        );
                    }

                    // If no hardware virtual cable output stream is active, discard captured mic frames so they never accumulate.
                    if _cpal_virtual_out_stream.is_none() {
                        if let Some(mut ring) = engine.mic_ring_buffer.try_lock() {
                            ring.clear();
                        }
                    }

                    if engine.panic_muted.load(Ordering::Acquire) {
                        {
                            let mut playbacks = engine.active_playbacks.lock();
                            for pb in playbacks.iter() {
                                pb.cancelled.store(true, Ordering::Release);
                            }
                            playbacks.clear();
                        }
                        {
                            let mut v_queue = engine.virtual_sb_queue.lock();
                            for track in v_queue.iter() {
                                track.cancelled.store(true, Ordering::Release);
                            }
                            v_queue.clear();
                        }
                        if let Some(mut ring) = engine.mic_ring_buffer.try_lock() {
                            ring.clear();
                        }
                        engine.mic_rms_bits.store(0.0f32.to_bits(), Ordering::Release);
                        engine.sb_rms_bits.store(0.0f32.to_bits(), Ordering::Release);
                        engine.mon_rms_bits.store(0.0f32.to_bits(), Ordering::Release);
                        continue;
                    }

                    let sb_vol = f32::from_bits(engine.sb_vol_bits.load(Ordering::Relaxed));
                    let mon_vol = f32::from_bits(engine.mon_vol_bits.load(Ordering::Relaxed));
                    let pitch = f32::from_bits(engine.pitch_bits.load(Ordering::Relaxed));
                    let reverb = f32::from_bits(engine.reverb_bits.load(Ordering::Relaxed));
                    let distortion = f32::from_bits(engine.distortion_bits.load(Ordering::Relaxed));

                    let pitch_active = pitch.abs() > 1e-4;
                    let reverb_active = reverb.abs() > 1e-4;
                    let distortion_active = distortion.abs() > 1e-4;
                    let pitch_step = if pitch_active {
                        2.0f64.powf((pitch as f64) / 12.0).clamp(0.5, 2.0)
                    } else {
                        1.0
                    };
                    let drive = 1.0 + distortion * 6.0;

                    let mut sum_sq_sb = 0.0f32;
                    {
                        let mut playbacks = engine.active_playbacks.lock();
                        for _ in 0..frames_per_tick {
                            let mut mixed_sample = 0.0f32;
                            for pb in playbacks.iter_mut() {
                                if pb.cancelled.load(Ordering::Relaxed) {
                                    pb.is_finished = true;
                                    continue;
                                }
                                let total_samples = pb.samples.len();
                                let sample_index = pb.cursor_f64.floor() as usize;
                                if sample_index < total_samples {
                                    let vol = f32::from_bits(pb.volume_bits.load(Ordering::Relaxed));
                                    mixed_sample += pb.samples[sample_index] * vol;
                                    pb.cursor_f64 += pitch_step;
                                    pb.current_frame = pb.cursor_f64.floor() as usize;
                                } else {
                                    pb.is_finished = true;
                                }
                            }

                            let mut processed = mixed_sample;
                            if reverb_active {
                                let delayed = reverb_comb[comb_idx];
                                let wet = processed + delayed * (reverb * 0.55);
                                reverb_comb[comb_idx] = wet;
                                comb_idx = (comb_idx + 1) % reverb_comb.len();
                                processed = wet;
                            }
                            if distortion_active {
                                processed = (processed * drive).tanh();
                            }

                            let shaped = (processed * sb_vol).clamp(-1.0, 1.0);
                            sum_sq_sb += shaped * shaped;
                        }
                        playbacks.retain(|pb| !pb.is_finished && !pb.cancelled.load(Ordering::Relaxed));
                    }

                    let sb_rms = (sum_sq_sb / frames_per_tick as f32).sqrt().clamp(0.0, 1.0);
                    let mon_rms = (sb_rms * mon_vol).clamp(0.0, 1.0);

                    engine.sb_rms_bits.store(sb_rms.to_bits(), Ordering::Release);
                    engine.mon_rms_bits.store(mon_rms.to_bits(), Ordering::Release);
                }
            })
            .ok();
    }

    fn push_captured_mic_samples(
        interleaved_f32: &[f32],
        in_channels: usize,
        in_sample_rate: u32,
        mic_rms_bits: &AtomicU32,
        mic_ring_buffer: &Mutex<VecDeque<f32>>,
        panic_flag: &AtomicBool,
    ) {
        if panic_flag.load(Ordering::Relaxed) || interleaved_f32.is_empty() {
            mic_rms_bits.store(0.0f32.to_bits(), Ordering::Relaxed);
            return;
        }
        let mono = Self::downmix_to_mono(interleaved_f32, in_channels);
        let sum_sq: f32 = mono.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / mono.len().max(1) as f32).sqrt().clamp(0.0, 1.0);
        mic_rms_bits.store(rms.to_bits(), Ordering::Relaxed);

        let resampled_48k = Self::resample_linear(&mono, in_sample_rate, INTERNAL_SAMPLE_RATE);
        // Use non-blocking try_lock in real-time audio capture callback to prevent mutex contention glitches.
        if let Some(mut ring) = mic_ring_buffer.try_lock() {
            for s in resampled_48k {
                if ring.len() >= MAX_MIC_RING_SAMPLES {
                    ring.pop_front();
                }
                ring.push_back(s);
            }
        }
    }

    fn spawn_cpal_mic_capture(
        mic_device_id: &str,
        mic_rms_bits: Arc<AtomicU32>,
        mic_ring_buffer: Arc<Mutex<VecDeque<f32>>>,
        panic_flag: Arc<AtomicBool>,
        recovery_flag: Arc<AtomicBool>,
    ) -> Option<cpal::Stream> {
        let device = resolve_input_device(mic_device_id)?;
        let supported_config = match device.default_input_config() {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("Audio stream error (default_input_config): {:?}", err);
                return None;
            }
        };

        let sample_format = supported_config.sample_format();
        let in_channels = supported_config.channels().max(1) as usize;
        let in_sample_rate = supported_config.sample_rate().0;
        let (preferred_cfg, fallback_cfg) =
            Self::preferred_and_fallback_stream_configs(&supported_config);

        let build_for_cfg = |cfg: &StreamConfig| -> Result<cpal::Stream, cpal::BuildStreamError> {
            let err_recovery = Arc::clone(&recovery_flag);
            let err_fn = move |err| {
                eprintln!("Audio stream error: {:?}", err);
                err_recovery.store(true, Ordering::Release);
            };
            let rms_cb = Arc::clone(&mic_rms_bits);
            let ring_cb = Arc::clone(&mic_ring_buffer);
            let panic_cb = Arc::clone(&panic_flag);

            match sample_format {
                SampleFormat::F32 => device.build_input_stream(
                    cfg,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        Self::push_captured_mic_samples(
                            data,
                            in_channels,
                            in_sample_rate,
                            &rms_cb,
                            &ring_cb,
                            &panic_cb,
                        );
                    },
                    err_fn,
                    None,
                ),
                SampleFormat::I16 => device.build_input_stream(
                    cfg,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let converted: Vec<f32> =
                            data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                        Self::push_captured_mic_samples(
                            &converted,
                            in_channels,
                            in_sample_rate,
                            &rms_cb,
                            &ring_cb,
                            &panic_cb,
                        );
                    },
                    err_fn,
                    None,
                ),
                SampleFormat::U16 => device.build_input_stream(
                    cfg,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let converted: Vec<f32> = data
                            .iter()
                            .map(|&s| (s as f32 - 32768.0) / 32768.0)
                            .collect();
                        Self::push_captured_mic_samples(
                            &converted,
                            in_channels,
                            in_sample_rate,
                            &rms_cb,
                            &ring_cb,
                            &panic_cb,
                        );
                    },
                    err_fn,
                    None,
                ),
                _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
            }
        };

        let stream_res = build_for_cfg(&preferred_cfg).or_else(|_| build_for_cfg(&fallback_cfg));

        match stream_res {
            Ok(stream) => {
                if let Err(err) = stream.play() {
                    eprintln!("Audio stream error (mic stream play): {:?}", err);
                    recovery_flag.store(true, Ordering::Release);
                    return None;
                }
                Some(stream)
            }
            Err(err) => {
                eprintln!("Audio stream error (build_input_stream): {:?}", err);
                None
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_virtual_mic_passthrough_stream(
        virtual_mic_id: &str,
        monitor_headphone_id: &str,
        mic_ring_buffer: Arc<Mutex<VecDeque<f32>>>,
        virtual_sb_queue: Arc<Mutex<Vec<VirtualSinkTrack>>>,
        mic_vol_bits: Arc<AtomicU32>,
        sb_vol_bits: Arc<AtomicU32>,
        pitch_bits: Arc<AtomicU32>,
        reverb_bits: Arc<AtomicU32>,
        distortion_bits: Arc<AtomicU32>,
        panic_flag: Arc<AtomicBool>,
        recovery_flag: Arc<AtomicBool>,
        virtual_stream_active: Arc<AtomicBool>,
    ) -> Option<cpal::Stream> {
        // Strictly resolve only a virtual audio cable; never allow fallback or routing to monitor/speakers.
        let device = resolve_virtual_mic_output_device(virtual_mic_id, monitor_headphone_id)?;
        let supported_config = match device.default_output_config() {
            Ok(cfg) => cfg,
            Err(err) => {
                eprintln!("Audio stream error (virtual default_output_config): {:?}", err);
                return None;
            }
        };

        let sample_format = supported_config.sample_format();
        let out_channels = supported_config.channels().max(1) as usize;
        let out_sample_rate = supported_config.sample_rate().0;
        let (preferred_cfg, fallback_cfg) =
            Self::preferred_and_fallback_stream_configs(&supported_config);

        let build_for_cfg = |cfg: &StreamConfig| -> Result<cpal::Stream, cpal::BuildStreamError> {
            let err_recovery = Arc::clone(&recovery_flag);
            let err_active = Arc::clone(&virtual_stream_active);
            let err_fn = move |err| {
                eprintln!("Audio stream error: {:?}", err);
                err_active.store(false, Ordering::Release);
                err_recovery.store(true, Ordering::Release);
            };

            let mic_ring_cb = Arc::clone(&mic_ring_buffer);
            let sb_queue_cb = Arc::clone(&virtual_sb_queue);
            let mic_vol_cb = Arc::clone(&mic_vol_bits);
            let sb_vol_cb = Arc::clone(&sb_vol_bits);
            let pitch_cb = Arc::clone(&pitch_bits);
            let reverb_cb = Arc::clone(&reverb_bits);
            let distortion_cb = Arc::clone(&distortion_bits);
            let panic_cb = Arc::clone(&panic_flag);

            let mut reverb_line = vec![0.0f32; 2048];
            let mut rev_cursor = 0usize;
            let mut pitch_phase = 0.0f32;

            let mut render_f32 = move |out_frames: usize| -> Vec<f32> {
                if panic_cb.load(Ordering::Relaxed) || out_frames == 0 {
                    return vec![0.0f32; out_frames * out_channels];
                }

                let mic_vol = f32::from_bits(mic_vol_cb.load(Ordering::Relaxed));
                let sb_vol = f32::from_bits(sb_vol_cb.load(Ordering::Relaxed));
                let pitch = f32::from_bits(pitch_cb.load(Ordering::Relaxed));
                let reverb = f32::from_bits(reverb_cb.load(Ordering::Relaxed));
                let distortion = f32::from_bits(distortion_cb.load(Ordering::Relaxed));

                let pitch_active = pitch.abs() > 1e-4;
                let reverb_active = reverb.abs() > 1e-4;
                let distortion_active = distortion.abs() > 1e-4;
                let drive = 1.0 + distortion * 6.0;
                let pitch_factor = if pitch_active {
                    2.0f64.powf((pitch as f64) / 12.0).clamp(0.5, 2.0)
                } else {
                    1.0
                };

                let needed_48k = (((out_frames as u64) * (INTERNAL_SAMPLE_RATE as u64))
                    / (out_sample_rate.max(1) as u64))
                    .max(1) as usize;

                let mut mic_mono_48k = vec![0.0f32; needed_48k];
                if let Some(mut ring) = mic_ring_cb.try_lock() {
                    for slot in mic_mono_48k.iter_mut() {
                        let raw = if !pitch_active {
                            ring.pop_front().unwrap_or(0.0)
                        } else {
                            pitch_phase += pitch_factor as f32;
                            let step = pitch_phase.floor() as usize;
                            pitch_phase -= step as f32;
                            let mut s_val = 0.0f32;
                            for _ in 0..step.max(1) {
                                if let Some(s) = ring.pop_front() {
                                    s_val = s;
                                }
                            }
                            s_val
                        };
                        *slot = raw * mic_vol;
                    }
                }

                let mut mixed_mono =
                    Self::resample_linear(&mic_mono_48k, INTERNAL_SAMPLE_RATE, out_sample_rate);
                mixed_mono.resize(out_frames, 0.0f32);

                // Mix active soundboard samples non-blockingly at the virtual device's native sample rate.
                if let Some(mut sb_tracks) = sb_queue_cb.try_lock() {
                    for track in sb_tracks.iter_mut() {
                        if track.cancelled.load(Ordering::Relaxed) {
                            track.is_finished = true;
                            continue;
                        }

                        let device_pcm = match &track.resampled_for_rate {
                            Some((rate, buf)) if *rate == out_sample_rate => Arc::clone(buf),
                            _ => {
                                let prev_len = track
                                    .resampled_for_rate
                                    .as_ref()
                                    .map(|(_, b)| b.len())
                                    .unwrap_or(track.samples_48k_mono.len())
                                    .max(1);
                                let resampled = Arc::new(Self::resample_linear(
                                    &track.samples_48k_mono,
                                    INTERNAL_SAMPLE_RATE,
                                    out_sample_rate,
                                ));
                                if track.current_frame > 0 && !resampled.is_empty() {
                                    let progress =
                                        (track.cursor_f64 / (prev_len as f64)).clamp(0.0, 1.0);
                                    track.cursor_f64 = progress * (resampled.len() as f64);
                                    track.current_frame = track.cursor_f64.floor() as usize;
                                }
                                track.resampled_for_rate =
                                    Some((out_sample_rate, Arc::clone(&resampled)));
                                resampled
                            }
                        };

                        let total_samples = device_pcm.len();
                        if total_samples == 0 {
                            track.is_finished = true;
                            continue;
                        }

                        let pad_vol = f32::from_bits(track.volume_bits.load(Ordering::Relaxed));
                        let eff_gain = (pad_vol * sb_vol).clamp(0.0, 1.5);
                        for frame_sample in mixed_mono.iter_mut() {
                            let sample_index = track.cursor_f64.floor() as usize;
                            if sample_index < total_samples {
                                let next_idx = (sample_index + 1).min(total_samples.saturating_sub(1));
                                let frac = (track.cursor_f64 - (sample_index as f64)) as f32;
                                let raw_sb = device_pcm[sample_index]
                                    + frac * (device_pcm[next_idx] - device_pcm[sample_index]);
                                *frame_sample =
                                    (*frame_sample + raw_sb * eff_gain).clamp(-1.0, 1.0);
                                track.cursor_f64 += pitch_factor;
                                track.current_frame = track.cursor_f64.floor() as usize;
                            } else {
                                track.is_finished = true;
                                break;
                            }
                        }

                        if (track.cursor_f64.floor() as usize) >= total_samples {
                            track.is_finished = true;
                        }
                    }
                    sb_tracks.retain(|t| !t.is_finished && !t.cancelled.load(Ordering::Relaxed));
                }

                if reverb_active || distortion_active {
                    for sample in mixed_mono.iter_mut() {
                        let mut val = *sample;
                        if reverb_active {
                            let delayed = reverb_line[rev_cursor];
                            let wet = val + delayed * (reverb * 0.45);
                            reverb_line[rev_cursor] = wet;
                            rev_cursor = (rev_cursor + 1) % reverb_line.len();
                            val = wet;
                        }
                        if distortion_active {
                            val = (val * drive).tanh();
                        }
                        *sample = val.clamp(-1.0, 1.0);
                    }
                }

                let mut interleaved =
                    Self::mono_to_interleaved_channels(&mixed_mono, out_channels);
                interleaved.resize(out_frames * out_channels, 0.0f32);
                interleaved
            };

            match sample_format {
                SampleFormat::F32 => device.build_output_stream(
                    cfg,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let frames = data.len() / out_channels;
                        let rendered = render_f32(frames);
                        for (dst, &src) in data.iter_mut().zip(rendered.iter()) {
                            *dst = src;
                        }
                    },
                    err_fn,
                    None,
                ),
                SampleFormat::I16 => device.build_output_stream(
                    cfg,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        let frames = data.len() / out_channels;
                        let rendered = render_f32(frames);
                        for (dst, &src) in data.iter_mut().zip(rendered.iter()) {
                            *dst = (src.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        }
                    },
                    err_fn,
                    None,
                ),
                SampleFormat::U16 => device.build_output_stream(
                    cfg,
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        let frames = data.len() / out_channels;
                        let rendered = render_f32(frames);
                        for (dst, &src) in data.iter_mut().zip(rendered.iter()) {
                            *dst = ((src.clamp(-1.0, 1.0) * 32767.0) + 32768.0) as u16;
                        }
                    },
                    err_fn,
                    None,
                ),
                _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
            }
        };

        let stream_res = build_for_cfg(&preferred_cfg).or_else(|_| build_for_cfg(&fallback_cfg));

        match stream_res {
            Ok(stream) => {
                if let Err(err) = stream.play() {
                    eprintln!("Audio stream error (virtual output play): {:?}", err);
                    virtual_stream_active.store(false, Ordering::Release);
                    recovery_flag.store(true, Ordering::Release);
                    return None;
                }
                virtual_stream_active.store(true, Ordering::Release);
                Some(stream)
            }
            Err(err) => {
                eprintln!("Audio stream error (virtual build_output_stream): {:?}", err);
                virtual_stream_active.store(false, Ordering::Release);
                None
            }
        }
    }

    // Spawn an independent CPAL output stream for a single soundboard playback sink (Monitor or Virtual Mic).
    #[allow(clippy::too_many_arguments)]
    fn spawn_independent_sink_playback(
        sink_label: &'static str,
        target_device_id: String,
        monitor_device_id: String,
        is_virtual_sink: bool,
        samples_48k_mono: Arc<Vec<f32>>,
        pad_vol_bits: Arc<AtomicU32>,
        sb_vol_bits: Arc<AtomicU32>,
        mon_vol_bits: Arc<AtomicU32>,
        pitch_bits: Arc<AtomicU32>,
        reverb_bits: Arc<AtomicU32>,
        distortion_bits: Arc<AtomicU32>,
        panic_flag: Arc<AtomicBool>,
        cancelled_flag: Arc<AtomicBool>,
        recovery_flag: Arc<AtomicBool>,
    ) {
        std::thread::Builder::new()
            .name(format!("sonicboard-sink-{sink_label}"))
            .spawn(move || {
                let device_opt = if is_virtual_sink {
                    resolve_virtual_mic_output_device(&target_device_id, &monitor_device_id)
                } else {
                    resolve_monitor_output_device(&target_device_id)
                };

                let Some(device) = device_opt else {
                    if !target_device_id.starts_with("out::virtual::") {
                        eprintln!(
                            "Audio stream error ({sink_label}): device '{target_device_id}' not found"
                        );
                    }
                    return;
                };

                let supported_config = match device.default_output_config() {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        eprintln!(
                            "Audio stream error ({sink_label} default_output_config): {:?}",
                            err
                        );
                        return;
                    }
                };

                let sample_format = supported_config.sample_format();
                let out_channels = supported_config.channels().max(1) as usize;
                let out_sample_rate = supported_config.sample_rate().0;
                let (preferred_cfg, fallback_cfg) =
                    Self::preferred_and_fallback_stream_configs(&supported_config);

                let resampled_mono = Arc::new(Self::resample_linear(
                    &samples_48k_mono,
                    INTERNAL_SAMPLE_RATE,
                    out_sample_rate,
                ));
                let total_samples = resampled_mono.len();
                let finished = Arc::new(AtomicBool::new(false));

                let build_for_cfg =
                    |cfg: &StreamConfig| -> Result<cpal::Stream, cpal::BuildStreamError> {
                        let err_recovery = Arc::clone(&recovery_flag);
                        let err_fn = move |err| {
                            eprintln!("Audio stream error ({sink_label}): {:?}", err);
                            err_recovery.store(true, Ordering::Release);
                        };

                        let finished_cb = Arc::clone(&finished);
                        let panic_cb = Arc::clone(&panic_flag);
                        let cancelled_cb = Arc::clone(&cancelled_flag);
                        let samples_cb = Arc::clone(&resampled_mono);
                        let pad_vol_cb = Arc::clone(&pad_vol_bits);
                        let sb_vol_cb = Arc::clone(&sb_vol_bits);
                        let mon_vol_cb = Arc::clone(&mon_vol_bits);
                        let pitch_cb = Arc::clone(&pitch_bits);
                        let reverb_cb = Arc::clone(&reverb_bits);
                        let distortion_cb = Arc::clone(&distortion_bits);

                        let mut current_frame_f64 = 0.0f64;
                        let mut tail_silence_frames = 0usize;
                        let mut reverb_comb = vec![0.0f32; 2048];
                        let mut rev_idx = 0usize;

                        let mut render_sink_f32 = move |out_frames: usize| -> Vec<f32> {
                            if panic_cb.load(Ordering::Relaxed)
                                || cancelled_cb.load(Ordering::Relaxed)
                                || out_frames == 0
                            {
                                finished_cb.store(true, Ordering::Release);
                                return vec![0.0f32; out_frames * out_channels];
                            }

                            let pad_vol = f32::from_bits(pad_vol_cb.load(Ordering::Relaxed));
                            let sb_vol = f32::from_bits(sb_vol_cb.load(Ordering::Relaxed));
                            let mon_vol = f32::from_bits(mon_vol_cb.load(Ordering::Relaxed));
                            let effective_vol = if is_virtual_sink {
                                (pad_vol * sb_vol).clamp(0.0, 1.5)
                            } else {
                                (pad_vol * sb_vol * mon_vol).clamp(0.0, 1.5)
                            };

                            let pitch = f32::from_bits(pitch_cb.load(Ordering::Relaxed));
                            let reverb = f32::from_bits(reverb_cb.load(Ordering::Relaxed));
                            let distortion = f32::from_bits(distortion_cb.load(Ordering::Relaxed));

                            let pitch_active = pitch.abs() > 1e-4;
                            let reverb_active = reverb.abs() > 1e-4;
                            let distortion_active = distortion.abs() > 1e-4;
                            let pitch_step = if pitch_active {
                                2.0f64.powf((pitch as f64) / 12.0).clamp(0.5, 2.0)
                            } else {
                                1.0
                            };
                            let drive = 1.0 + distortion * 6.0;

                            let mut mono_out = Vec::with_capacity(out_frames);
                            for _ in 0..out_frames {
                                let sample_index = current_frame_f64.floor() as usize;
                                if sample_index < total_samples {
                                    let next_idx =
                                        (sample_index + 1).min(total_samples.saturating_sub(1));
                                    let frac =
                                        (current_frame_f64 - (sample_index as f64)) as f32;
                                    let mut mono_val = samples_cb[sample_index]
                                        + frac
                                            * (samples_cb[next_idx] - samples_cb[sample_index]);

                                    if reverb_active {
                                        let delayed = reverb_comb[rev_idx];
                                        let wet = mono_val + delayed * (reverb * 0.45);
                                        reverb_comb[rev_idx] = wet;
                                        rev_idx = (rev_idx + 1) % reverb_comb.len();
                                        mono_val = wet;
                                    }
                                    if distortion_active {
                                        mono_val = (mono_val * drive).tanh();
                                    }

                                    mono_out.push((mono_val * effective_vol).clamp(-1.0, 1.0));
                                    current_frame_f64 += pitch_step;
                                } else {
                                    mono_out.push(0.0);
                                    tail_silence_frames += 1;
                                }
                            }

                            if (current_frame_f64.floor() as usize) >= total_samples
                                && tail_silence_frames >= SAFE_WASAPI_BUFFER_FRAMES as usize
                            {
                                finished_cb.store(true, Ordering::Release);
                            }

                            let mut interleaved =
                                Self::mono_to_interleaved_channels(&mono_out, out_channels);
                            interleaved.resize(out_frames * out_channels, 0.0f32);
                            interleaved
                        };

                        match sample_format {
                            SampleFormat::F32 => device.build_output_stream(
                                cfg,
                                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                    let frames = data.len() / out_channels;
                                    let rendered = render_sink_f32(frames);
                                    for (dst, &src) in data.iter_mut().zip(rendered.iter()) {
                                        *dst = src;
                                    }
                                },
                                err_fn,
                                None,
                            ),
                            SampleFormat::I16 => device.build_output_stream(
                                cfg,
                                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                                    let frames = data.len() / out_channels;
                                    let rendered = render_sink_f32(frames);
                                    for (dst, &src) in data.iter_mut().zip(rendered.iter()) {
                                        *dst = (src.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                                    }
                                },
                                err_fn,
                                None,
                            ),
                            SampleFormat::U16 => device.build_output_stream(
                                cfg,
                                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                                    let frames = data.len() / out_channels;
                                    let rendered = render_sink_f32(frames);
                                    for (dst, &src) in data.iter_mut().zip(rendered.iter()) {
                                        *dst = ((src.clamp(-1.0, 1.0) * 32767.0) + 32768.0) as u16;
                                    }
                                },
                                err_fn,
                                None,
                            ),
                            _ => Err(cpal::BuildStreamError::StreamConfigNotSupported),
                        }
                    };

                let stream_res =
                    build_for_cfg(&preferred_cfg).or_else(|_| build_for_cfg(&fallback_cfg));

                let stream = match stream_res {
                    Ok(s) => s,
                    Err(err) => {
                        eprintln!(
                            "Audio stream error ({sink_label} build_output_stream): {:?}",
                            err
                        );
                        return;
                    }
                };

                if let Err(err) = stream.play() {
                    eprintln!("Audio stream error ({sink_label} play): {:?}", err);
                    return;
                }

                while !finished.load(Ordering::Acquire) {
                    if panic_flag.load(Ordering::Relaxed)
                        || cancelled_flag.load(Ordering::Acquire)
                    {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(15));
                }
            })
            .ok();
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
            return Err(
                "All three routing endpoints (Mic, Virtual Cable, Monitor) must be selected"
                    .to_string(),
            );
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

        {
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
                "Dual-sink routing locked to hardware virtual cable (WASAPI)".to_string()
            } else {
                "Routed via internal mixer bridge (VB-Cable driver not detected)".to_string()
            };
        }

        // Signal the audio pipeline thread to rebuild input and virtual output streams immediately.
        self.routing_generation.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn get_connection_status(&self) -> RoutingStatus {
        let mut status = self.routing.lock().clone();
        let muted = self.panic_muted.load(Ordering::Acquire);
        status.panic_muted = muted;
        status.connected = !muted;
        if muted {
            status.status_message =
                "PANIC MUTE ENGAGED: All audio sinks and mic pass-through cut".to_string();
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
        let (decoded, canonical_path) = self.decode_and_cache_sound(&id, &file_path);
        let persisted_file_path = canonical_path.unwrap_or(file_path);
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
            file_path: persisted_file_path,
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
        self.sound_pcm_cache.lock().remove(sound_id);
        self.decoded_cache.lock().remove(sound_id);
        let canonical = canonical_sound_wav_path(sound_id);
        if canonical.exists() {
            let _ = std::fs::remove_file(canonical);
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

        // Clone pre-decoded 48 kHz mono PCM Arc slice directly from RAM cache (zero disk I/O on play).
        let samples_48k_mono = {
            let cached = self.sound_pcm_cache.lock().get(&sound.id).cloned();
            match cached {
                Some(arc_pcm) => arc_pcm,
                None => {
                    self.load_or_decode_sound(&sound.id, &sound.file_path)
                        .samples_48k_mono
                }
            }
        };

        let (virtual_mic_id, monitor_headphone_id) = {
            let r = self.routing.lock();
            (r.virtual_mic_id.clone(), r.monitor_headphone_id.clone())
        };

        let pad_vol_bits = Arc::new(AtomicU32::new(sound.volume.to_bits()));
        let cancelled_flag = Arc::new(AtomicBool::new(false));

        {
            let mut playbacks = self.active_playbacks.lock();
            for pb in playbacks.iter() {
                if pb.sound_id == sound.id {
                    pb.cancelled.store(true, Ordering::Release);
                }
            }
            playbacks.retain(|p| p.sound_id != sound.id);
            playbacks.push(ActiveSamplePlayback {
                sound_id: sound.id.clone(),
                samples: Arc::clone(&samples_48k_mono),
                current_frame: 0,
                cursor_f64: 0.0,
                volume_bits: Arc::clone(&pad_vol_bits),
                cancelled: Arc::clone(&cancelled_flag),
                is_finished: false,
            });
        }

        // Enqueue soundboard frames into the Virtual Mic stream's non-blocking mixer queue.
        {
            let mut v_queue = self.virtual_sb_queue.lock();
            for track in v_queue.iter() {
                if track.sound_id == sound.id {
                    track.cancelled.store(true, Ordering::Release);
                }
            }
            v_queue.retain(|t| t.sound_id != sound.id);
            v_queue.push(VirtualSinkTrack {
                sound_id: sound.id.clone(),
                samples_48k_mono: Arc::clone(&samples_48k_mono),
                resampled_for_rate: None,
                current_frame: 0,
                cursor_f64: 0.0,
                volume_bits: Arc::clone(&pad_vol_bits),
                cancelled: Arc::clone(&cancelled_flag),
                is_finished: false,
            });
        }

        // Spawn Monitor Headphone sink exclusively for soundboard sample playback.
        Self::spawn_independent_sink_playback(
            "monitor",
            monitor_headphone_id.clone(),
            monitor_headphone_id.clone(),
            false,
            Arc::clone(&samples_48k_mono),
            Arc::clone(&pad_vol_bits),
            Arc::clone(&self.sb_vol_bits),
            Arc::clone(&self.mon_vol_bits),
            Arc::clone(&self.pitch_bits),
            Arc::clone(&self.reverb_bits),
            Arc::clone(&self.distortion_bits),
            Arc::clone(&self.panic_muted),
            Arc::clone(&cancelled_flag),
            Arc::clone(&self.stream_needs_recovery),
        );

        // If the persistent Virtual Mic stream is not active yet, spawn an independent fallback virtual sink stream.
        if !self.virtual_stream_active.load(Ordering::Acquire) {
            Self::spawn_independent_sink_playback(
                "virtual-mic",
                virtual_mic_id,
                monitor_headphone_id,
                true,
                samples_48k_mono,
                pad_vol_bits,
                Arc::clone(&self.sb_vol_bits),
                Arc::clone(&self.mon_vol_bits),
                Arc::clone(&self.pitch_bits),
                Arc::clone(&self.reverb_bits),
                Arc::clone(&self.distortion_bits),
                Arc::clone(&self.panic_muted),
                cancelled_flag,
                Arc::clone(&self.stream_needs_recovery),
            );
        }

        Ok(())
    }

    pub fn stop_sound(&self, sound_id: &str) -> Result<(), String> {
        {
            let mut playbacks = self.active_playbacks.lock();
            for pb in playbacks.iter() {
                if pb.sound_id == sound_id {
                    pb.cancelled.store(true, Ordering::Release);
                }
            }
            playbacks.retain(|p| p.sound_id != sound_id);
        }
        {
            let mut v_queue = self.virtual_sb_queue.lock();
            for track in v_queue.iter() {
                if track.sound_id == sound_id {
                    track.cancelled.store(true, Ordering::Release);
                }
            }
            v_queue.retain(|t| t.sound_id != sound_id);
        }
        Ok(())
    }

    pub fn stop_all_sounds(&self) -> Result<(), String> {
        self.panic_muted.store(true, Ordering::SeqCst);
        {
            let mut playbacks = self.active_playbacks.lock();
            for pb in playbacks.iter() {
                pb.cancelled.store(true, Ordering::Release);
            }
            playbacks.clear();
        }
        {
            let mut v_queue = self.virtual_sb_queue.lock();
            for track in v_queue.iter() {
                track.cancelled.store(true, Ordering::Release);
            }
            v_queue.clear();
        }
        if let Some(mut ring) = self.mic_ring_buffer.try_lock() {
            ring.clear();
        }
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
            self.stream_needs_recovery.store(true, Ordering::Release);
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

        let playbacks = self.active_playbacks.lock();
        for pb in playbacks.iter() {
            if pb.sound_id == sound_id {
                pb.volume_bits.store(clamped.to_bits(), Ordering::Release);
            }
        }
        let v_queue = self.virtual_sb_queue.lock();
        for track in v_queue.iter() {
            if track.sound_id == sound_id {
                track.volume_bits.store(clamped.to_bits(), Ordering::Release);
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
            .filter(|p| !p.is_finished && !p.cancelled.load(Ordering::Relaxed))
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
