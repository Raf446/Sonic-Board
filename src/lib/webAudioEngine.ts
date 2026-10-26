import { SoundItem, VoiceFxState } from "../types/soundboard";

interface ActiveWebSource {
  soundId: string;
  source: AudioBufferSourceNode;
  gainNode: GainNode;
  startedAt: number;
  duration: number;
}

class WebAudioMonitorEngine {
  private ctx: AudioContext | null = null;
  private analyser: AnalyserNode | null = null;
  private masterGain: GainNode | null = null;
  private distortionNode: WaveShaperNode | null = null;
  private reverbDelay: DelayNode | null = null;
  private reverbFeedback: GainNode | null = null;
  private reverbWet: GainNode | null = null;
  private bufferCache: Map<string, AudioBuffer> = new Map();
  private activeSources: Map<string, ActiveWebSource> = new Map();
  private timeDomainData: Float32Array<ArrayBuffer> = new Float32Array(new ArrayBuffer(256 * 4));

  private soundboardVol = 0.9;
  private monitorVol = 0.8;
  private voiceFx: VoiceFxState = { pitch: 0, reverb: 0.15, distortion: 0.05 };
  private panicMuted = false;

  private ensureContext(): AudioContext | null {
    if (typeof window === "undefined") return null;
    const AudioCtx =
      window.AudioContext ||
      (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AudioCtx) return null;

    if (!this.ctx) {
      this.ctx = new AudioCtx();
      this.analyser = this.ctx.createAnalyser();
      this.analyser.fftSize = 256;
      this.analyser.smoothingTimeConstant = 0.65;

      this.masterGain = this.ctx.createGain();
      this.distortionNode = this.ctx.createWaveShaper();
      this.reverbDelay = this.ctx.createDelay(1.0);
      this.reverbDelay.delayTime.value = 0.085;
      this.reverbFeedback = this.ctx.createGain();
      this.reverbWet = this.ctx.createGain();

      this.updateDspNodes();

      this.distortionNode.connect(this.masterGain);
      this.distortionNode.connect(this.reverbDelay);
      this.reverbDelay.connect(this.reverbFeedback);
      this.reverbFeedback.connect(this.reverbDelay);
      this.reverbDelay.connect(this.reverbWet);
      this.reverbWet.connect(this.masterGain);

      this.masterGain.connect(this.analyser);
      this.analyser.connect(this.ctx.destination);
    }

    if (this.ctx.state === "suspended") {
      this.ctx.resume().catch(() => {});
    }
    return this.ctx;
  }

  private makeDistortionCurve(amount: number): Float32Array<ArrayBuffer> {
    const k = Math.max(0, amount) * 90;
    const samples = 256;
    const buffer = new ArrayBuffer(samples * 4);
    const curve = new Float32Array(buffer);
    const deg = Math.PI / 180;
    for (let i = 0; i < samples; i++) {
      const x = (i * 2) / samples - 1;
      curve[i] = k === 0 ? x : ((3 + k) * x * 20 * deg) / (Math.PI + k * Math.abs(x));
    }
    return curve;
  }

  private updateDspNodes(): void {
    if (!this.masterGain || !this.distortionNode || !this.reverbFeedback || !this.reverbWet) {
      return;
    }
    const effectiveMaster = this.panicMuted ? 0 : this.soundboardVol * this.monitorVol;
    this.masterGain.gain.value = effectiveMaster;
    this.distortionNode.curve = this.makeDistortionCurve(this.voiceFx.distortion);
    this.reverbFeedback.gain.value = Math.min(0.75, this.voiceFx.reverb * 0.65);
    this.reverbWet.gain.value = this.voiceFx.reverb * 0.5;
  }

  public setMixerVolumes(_micVol: number, soundboardVol: number, monitorVol: number): void {
    this.soundboardVol = Math.max(0, Math.min(1, soundboardVol));
    this.monitorVol = Math.max(0, Math.min(1, monitorVol));
    this.updateDspNodes();
  }

  public setVoiceFx(fx: VoiceFxState): void {
    this.voiceFx = { ...fx };
    this.updateDspNodes();
    const rate = Math.pow(2, this.voiceFx.pitch / 12);
    for (const active of this.activeSources.values()) {
      active.source.playbackRate.value = rate;
    }
  }

  private synthesizeBuffer(ctx: AudioContext, sound: SoundItem): AudioBuffer {
    const sr = ctx.sampleRate;
    const duration = Math.max(0.4, sound.durationSecs || 1.0);
    const length = Math.floor(sr * duration);
    const buffer = ctx.createBuffer(1, length, sr);
    const data = buffer.getChannelData(0);

    let seed = 0;
    for (let i = 0; i < sound.id.length; i++) {
      seed = (seed * 31 + sound.id.charCodeAt(i)) >>> 0;
    }
    const baseFreq = 180 + (seed % 420);

    for (let i = 0; i < length; i++) {
      const t = i / sr;
      const wfIdx = Math.min(
        sound.waveform.length - 1,
        Math.floor((t / duration) * sound.waveform.length)
      );
      const env = sound.waveform[wfIdx] ?? Math.exp(-t * 3);
      data[i] =
        (Math.sin(2 * Math.PI * baseFreq * t) * 0.65 +
          Math.sin(2 * Math.PI * (baseFreq * 1.498) * t) * 0.35) *
        env *
        0.5;
    }
    return buffer;
  }

  private async loadBuffer(ctx: AudioContext, sound: SoundItem): Promise<AudioBuffer> {
    const cached = this.bufferCache.get(sound.filePath);
    if (cached) return cached;

    try {
      if (
        sound.filePath.startsWith("/") ||
        sound.filePath.startsWith("http") ||
        sound.filePath.startsWith("blob:") ||
        sound.filePath.startsWith("data:")
      ) {
        const res = await fetch(sound.filePath);
        if (res.ok) {
          const arr = await res.arrayBuffer();
          const decoded = await ctx.decodeAudioData(arr);
          this.bufferCache.set(sound.filePath, decoded);
          return decoded;
        }
      }
    } catch {
      // Fallback to synthesized buffer when running in test or restricted file protocol.
    }

    const fallback = this.synthesizeBuffer(ctx, sound);
    this.bufferCache.set(sound.filePath, fallback);
    return fallback;
  }

  public async playSound(sound: SoundItem): Promise<void> {
    this.panicMuted = false;
    const ctx = this.ensureContext();
    if (!ctx || !this.distortionNode) return;

    this.stopSound(sound.id);
    const buffer = await this.loadBuffer(ctx, sound);

    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.playbackRate.value = Math.pow(2, this.voiceFx.pitch / 12);

    const gainNode = ctx.createGain();
    gainNode.gain.value = Math.max(0, Math.min(1, sound.volume));

    source.connect(gainNode);
    gainNode.connect(this.distortionNode);

    const entry: ActiveWebSource = {
      soundId: sound.id,
      source,
      gainNode,
      startedAt: ctx.currentTime,
      duration: buffer.duration,
    };
    this.activeSources.set(sound.id, entry);

    source.onended = () => {
      const current = this.activeSources.get(sound.id);
      if (current === entry) {
        this.activeSources.delete(sound.id);
      }
    };

    this.updateDspNodes();
    source.start(0);
  }

  public stopSound(soundId: string): void {
    const existing = this.activeSources.get(soundId);
    if (existing) {
      try {
        existing.source.stop();
        existing.source.disconnect();
      } catch {
        // Ignore already-stopped source state.
      }
      this.activeSources.delete(soundId);
    }
  }

  public stopAllSounds(): void {
    this.panicMuted = true;
    for (const id of Array.from(this.activeSources.keys())) {
      this.stopSound(id);
    }
    this.updateDspNodes();
  }

  public setPanicMute(muted: boolean): void {
    if (muted) {
      this.stopAllSounds();
    } else {
      this.panicMuted = false;
      this.updateDspNodes();
    }
  }

  public setSoundVolume(soundId: string, volume: number): void {
    const active = this.activeSources.get(soundId);
    if (active) {
      active.gainNode.gain.value = Math.max(0, Math.min(1, volume));
    }
  }

  public getRealtimeRmsAndActiveIds(): { rms: number; activeIds: string[] } {
    if (this.panicMuted) {
      return { rms: 0, activeIds: [] };
    }
    const activeIds = Array.from(this.activeSources.keys());
    if (!this.analyser || activeIds.length === 0) {
      return { rms: 0, activeIds };
    }
    this.analyser.getFloatTimeDomainData(this.timeDomainData);
    let sumSq = 0;
    for (let i = 0; i < this.timeDomainData.length; i++) {
      const v = this.timeDomainData[i];
      sumSq += v * v;
    }
    const rms = Math.min(1, Math.sqrt(sumSq / this.timeDomainData.length) * 2.5);
    return { rms, activeIds };
  }
}

export const webAudioEngine = new WebAudioMonitorEngine();
