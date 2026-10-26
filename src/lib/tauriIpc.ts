import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  AudioDevicesResponse,
  AudioLevelPayload,
  HotkeyTriggerEvent,
  MixerState,
  RoutingStatus,
  SoundItem,
} from "../types/soundboard";
import { webAudioEngine } from "./webAudioEngine";

export const DEFAULT_CATEGORIES = ["Favorit", "Memes", "Gaming", "Anime", "Sound FX"];

export const FALLBACK_STARTER_SOUNDS: SoundItem[] = [
  {
    id: "snd-vine-boom",
    name: "Bass Drop Impact",
    filePath: "/samples/vine-boom.wav",
    hotkey: "Ctrl + 1",
    category: "Memes",
    volume: 0.95,
    durationSecs: 1.1,
    favorite: true,
    icon: "speaker",
    waveform: [
      1.0, 0.96, 0.91, 0.86, 0.8, 0.74, 0.68, 0.62, 0.57, 0.51, 0.46, 0.42, 0.37, 0.34,
      0.3, 0.27, 0.24, 0.21, 0.19, 0.17, 0.16, 0.15, 0.14, 0.13, 0.13, 0.12, 0.12, 0.12,
    ],
  },
  {
    id: "snd-clutch-siren",
    name: "Tactical Clutch Alert",
    filePath: "/samples/clutch-siren.wav",
    hotkey: "Ctrl + 2",
    category: "Gaming",
    volume: 0.88,
    durationSecs: 1.2,
    favorite: true,
    icon: "crosshair",
    waveform: [
      0.78, 0.92, 0.98, 0.86, 0.25, 0.74, 0.89, 0.95, 0.82, 0.22, 0.68, 0.84, 0.9, 0.76,
      0.2, 0.62, 0.75, 0.81, 0.67, 0.18, 0.52, 0.64, 0.69, 0.54, 0.15, 0.38, 0.28, 0.16,
    ],
  },
  {
    id: "snd-anime-wow",
    name: "Synth Chime Spark",
    filePath: "/samples/anime-wow.wav",
    hotkey: "Ctrl + 3",
    category: "Anime",
    volume: 0.9,
    durationSecs: 1.0,
    favorite: true,
    icon: "wand",
    waveform: [
      0.35, 0.58, 0.76, 0.92, 1.0, 0.94, 0.85, 0.77, 0.69, 0.62, 0.55, 0.48, 0.43, 0.38,
      0.33, 0.29, 0.25, 0.22, 0.19, 0.17, 0.15, 0.14, 0.13, 0.12, 0.12, 0.12, 0.12, 0.12,
    ],
  },
  {
    id: "snd-airhorn-triple",
    name: "Stadium Airhorn",
    filePath: "/samples/airhorn-triple.wav",
    hotkey: "Num 1",
    category: "Memes",
    volume: 0.82,
    durationSecs: 1.3,
    favorite: false,
    icon: "megaphone",
    waveform: [
      0.96, 0.98, 0.94, 0.18, 0.95, 0.99, 0.92, 0.18, 0.97, 1.0, 0.99, 0.98, 0.96, 0.94,
      0.91, 0.86, 0.78, 0.68, 0.58, 0.49, 0.4, 0.32, 0.25, 0.2, 0.16, 0.14, 0.12, 0.12,
    ],
  },
  {
    id: "snd-victory-fanfare",
    name: "Victory Brass Chord",
    filePath: "/samples/victory-fanfare.wav",
    hotkey: "Num 2",
    category: "Gaming",
    volume: 0.9,
    durationSecs: 1.25,
    favorite: false,
    icon: "trophy",
    waveform: [
      0.68, 0.72, 0.7, 0.65, 0.78, 0.82, 0.79, 0.74, 0.96, 1.0, 0.98, 0.94, 0.88, 0.81,
      0.73, 0.65, 0.58, 0.5, 0.43, 0.37, 0.31, 0.26, 0.22, 0.18, 0.15, 0.13, 0.12, 0.12,
    ],
  },
  {
    id: "snd-power-charge",
    name: "Mecha Laser Charge",
    filePath: "/samples/power-charge.wav",
    hotkey: "Num 3",
    category: "Anime",
    volume: 0.86,
    durationSecs: 1.1,
    favorite: false,
    icon: "zap",
    waveform: [
      0.18, 0.28, 0.42, 0.56, 0.68, 0.79, 0.86, 0.91, 0.94, 0.96, 0.98, 0.99, 1.0, 1.0,
      0.99, 0.98, 0.96, 0.92, 0.84, 0.72, 0.58, 0.45, 0.34, 0.25, 0.18, 0.14, 0.12, 0.12,
    ],
  },
  {
    id: "snd-censor-beep",
    name: "Broadcast 1kHz Bleep",
    filePath: "/samples/censor-beep.wav",
    hotkey: "Num 4",
    category: "Sound FX",
    volume: 0.78,
    durationSecs: 0.65,
    favorite: true,
    icon: "radio",
    waveform: [
      0.24, 0.95, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98,
      0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.98, 0.95, 0.42, 0.22, 0.15,
    ],
  },
  {
    id: "snd-glitch-transition",
    name: "Cyber Stutter Sweep",
    filePath: "/samples/glitch-transition.wav",
    hotkey: "Num 5",
    category: "Sound FX",
    volume: 0.85,
    durationSecs: 0.85,
    favorite: false,
    icon: "sliders",
    waveform: [
      1.0, 0.22, 0.92, 0.2, 0.85, 0.18, 0.78, 0.18, 0.71, 0.16, 0.64, 0.15, 0.58, 0.15,
      0.52, 0.14, 0.46, 0.14, 0.4, 0.13, 0.35, 0.13, 0.3, 0.12, 0.25, 0.12, 0.2, 0.12,
    ],
  },
];

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

const browserFallbackState = {
  sounds: [...FALLBACK_STARTER_SOUNDS],
  categories: [...DEFAULT_CATEGORIES],
  devices: {
    inputs: [
      {
        id: "in::0::FIFINE K688 USB Broadcast Dynamic Mic",
        name: "Microphone (FIFINE K688 USB Broadcast Dynamic)",
        kind: "input",
        isDefault: true,
        isVirtual: false,
        hostApi: "WASAPI",
        sampleRate: 48000,
        channels: 1,
      },
      {
        id: "in::1::Realtek High Definition Audio Mic",
        name: "Microphone Array (Realtek HD Audio)",
        kind: "input",
        isDefault: false,
        isVirtual: false,
        hostApi: "WASAPI",
        sampleRate: 48000,
        channels: 2,
      },
    ],
    outputs: [
      {
        id: "out::0::CABLE Input (VB-Audio Virtual Cable)",
        name: "CABLE Input (VB-Audio Virtual Cable)",
        kind: "output",
        isDefault: false,
        isVirtual: true,
        hostApi: "WASAPI",
        sampleRate: 48000,
        channels: 2,
      },
      {
        id: "out::1::Headphones (Studio Monitor DAC 24-bit)",
        name: "Headphones (Studio Monitor DAC 24-bit)",
        kind: "output",
        isDefault: true,
        isVirtual: false,
        hostApi: "WASAPI",
        sampleRate: 48000,
        channels: 2,
      },
      {
        id: "out::2::VoiceMeeter Aux Input (VB-Audio)",
        name: "VoiceMeeter Aux Input (VB-Audio)",
        kind: "output",
        isDefault: false,
        isVirtual: true,
        hostApi: "WASAPI",
        sampleRate: 48000,
        channels: 2,
      },
    ],
  } as AudioDevicesResponse,
  routing: {
    connected: true,
    micDeviceId: "in::0::FIFINE K688 USB Broadcast Dynamic Mic",
    micDeviceName: "Microphone (FIFINE K688 USB Broadcast Dynamic)",
    virtualMicId: "out::0::CABLE Input (VB-Audio Virtual Cable)",
    virtualMicName: "CABLE Input (VB-Audio Virtual Cable)",
    monitorHeadphoneId: "out::1::Headphones (Studio Monitor DAC 24-bit)",
    monitorHeadphoneName: "Headphones (Studio Monitor DAC 24-bit)",
    virtualDriverDetected: true,
    sampleRate: 48000,
    bufferSizeFrames: 256,
    estimatedLatencyMs: 5.3,
    statusMessage: "Dual-sink routing active (WASAPI 48kHz / 256f)",
    panicMuted: false,
  } as RoutingStatus,
  mixer: {
    micVol: 0.85,
    soundboardVol: 0.9,
    monitorVol: 0.8,
    voiceFx: { pitch: 0, reverb: 0.15, distortion: 0.05 },
    panicMuted: false,
  } as MixerState,
};

export async function ipcGetAudioDevices(): Promise<AudioDevicesResponse> {
  if (isTauriRuntime()) {
    return invoke<AudioDevicesResponse>("get_audio_devices");
  }
  return browserFallbackState.devices;
}

export async function ipcSetRouting(
  micDeviceId: string,
  virtualMicId: string,
  monitorHeadphoneId: string
): Promise<void> {
  if (isTauriRuntime()) {
    await invoke("set_routing", {
      micDeviceId,
      virtualMicId,
      monitorHeadphoneId,
    });
    return;
  }
  const mic = browserFallbackState.devices.inputs.find((d) => d.id === micDeviceId);
  const virt = browserFallbackState.devices.outputs.find((d) => d.id === virtualMicId);
  const mon = browserFallbackState.devices.outputs.find((d) => d.id === monitorHeadphoneId);
  browserFallbackState.routing = {
    ...browserFallbackState.routing,
    micDeviceId,
    micDeviceName: mic?.name ?? micDeviceId,
    virtualMicId,
    virtualMicName: virt?.name ?? virtualMicId,
    monitorHeadphoneId,
    monitorHeadphoneName: mon?.name ?? monitorHeadphoneId,
    virtualDriverDetected: Boolean(virt?.isVirtual),
    connected: !browserFallbackState.mixer.panicMuted,
    statusMessage: virt?.isVirtual
      ? "Dual-sink routing locked to virtual audio cable"
      : "Direct output routing active (non-virtual sink selected)",
  };
}

export async function ipcGetConnectionStatus(): Promise<RoutingStatus> {
  if (isTauriRuntime()) {
    return invoke<RoutingStatus>("get_connection_status");
  }
  return browserFallbackState.routing;
}

export async function ipcGetSounds(): Promise<SoundItem[]> {
  if (isTauriRuntime()) {
    return invoke<SoundItem[]>("get_sounds");
  }
  return [...browserFallbackState.sounds];
}

export async function ipcGetCategories(): Promise<string[]> {
  if (isTauriRuntime()) {
    return invoke<string[]>("get_categories");
  }
  return [...browserFallbackState.categories];
}

export async function ipcAddCategory(name: string): Promise<string[]> {
  if (isTauriRuntime()) {
    return invoke<string[]>("add_category", { name });
  }
  const clean = name.trim();
  if (!clean) throw new Error("Nama kategori tidak boleh kosong");
  if (!browserFallbackState.categories.some((c) => c.toLowerCase() === clean.toLowerCase())) {
    browserFallbackState.categories.push(clean);
  }
  return [...browserFallbackState.categories];
}

export async function ipcAddSound(
  name: string,
  filePath: string,
  hotkey: string | null,
  category: string,
  icon?: string
): Promise<SoundItem> {
  if (isTauriRuntime()) {
    return invoke<SoundItem>("add_sound", {
      name,
      filePath,
      hotkey,
      category,
      icon: icon ?? null,
    });
  }
  const newItem: SoundItem = {
    id: `snd-${Date.now()}`,
    name: name.trim(),
    filePath,
    hotkey: hotkey?.trim() || null,
    category: category || "Memes",
    volume: 0.9,
    durationSecs: 1.15,
    favorite: false,
    icon: icon || "volume-2",
    waveform: Array.from({ length: 28 }, (_, i) =>
      Number((0.2 + 0.75 * Math.abs(Math.sin((i + name.length) * 0.45))).toFixed(2))
    ),
  };
  browserFallbackState.sounds.push(newItem);
  return newItem;
}

export async function ipcUpdateSound(
  soundId: string,
  name: string,
  hotkey: string | null,
  category: string,
  favorite?: boolean,
  icon?: string
): Promise<SoundItem> {
  if (isTauriRuntime()) {
    return invoke<SoundItem>("update_sound", {
      soundId,
      name,
      hotkey,
      category,
      favorite: favorite ?? null,
      icon: icon ?? null,
    });
  }
  const idx = browserFallbackState.sounds.findIndex((s) => s.id === soundId);
  if (idx === -1) throw new Error("Suara tidak ditemukan");
  const current = browserFallbackState.sounds[idx];
  const updated: SoundItem = {
    ...current,
    name: name.trim() || current.name,
    hotkey: hotkey?.trim() || null,
    category: category || current.category,
    favorite: favorite ?? current.favorite,
    icon: icon || current.icon,
  };
  browserFallbackState.sounds[idx] = updated;
  return updated;
}

export async function ipcRemoveSound(soundId: string): Promise<void> {
  webAudioEngine.stopSound(soundId);
  if (isTauriRuntime()) {
    await invoke("remove_sound", { soundId });
    return;
  }
  browserFallbackState.sounds = browserFallbackState.sounds.filter((s) => s.id !== soundId);
}

export async function ipcPlaySound(sound: SoundItem): Promise<void> {
  await webAudioEngine.playSound(sound);
  browserFallbackState.mixer.panicMuted = false;
  browserFallbackState.routing.panicMuted = false;
  browserFallbackState.routing.connected = true;
  if (isTauriRuntime()) {
    await invoke("play_sound", { soundId: sound.id });
  }
}

export async function ipcStopSound(soundId: string): Promise<void> {
  webAudioEngine.stopSound(soundId);
  if (isTauriRuntime()) {
    await invoke("stop_sound", { soundId });
  }
}

export async function ipcStopAllSounds(): Promise<void> {
  webAudioEngine.stopAllSounds();
  browserFallbackState.mixer.panicMuted = true;
  browserFallbackState.routing.panicMuted = true;
  browserFallbackState.routing.connected = false;
  browserFallbackState.routing.statusMessage =
    "PANIC MUTE ENGAGED: All audio sinks and mic pass-through cut";
  if (isTauriRuntime()) {
    await invoke("stop_all_sounds");
  }
}

export async function ipcSetPanicMute(muted: boolean): Promise<boolean> {
  webAudioEngine.setPanicMute(muted);
  browserFallbackState.mixer.panicMuted = muted;
  browserFallbackState.routing.panicMuted = muted;
  browserFallbackState.routing.connected = !muted;
  browserFallbackState.routing.statusMessage = muted
    ? "PANIC MUTE ENGAGED: All audio sinks and mic pass-through cut"
    : "Dual-sink routing active (WASAPI 48kHz / 256f)";
  if (isTauriRuntime()) {
    return invoke<boolean>("set_panic_mute", { muted });
  }
  return muted;
}

export async function ipcSetSoundVolume(soundId: string, volume: number): Promise<void> {
  webAudioEngine.setSoundVolume(soundId, volume);
  const found = browserFallbackState.sounds.find((s) => s.id === soundId);
  if (found) found.volume = volume;
  if (isTauriRuntime()) {
    await invoke("set_sound_volume", { soundId, volume });
  }
}

export async function ipcRegisterHotkey(
  soundId: string,
  keyCombination: string
): Promise<void> {
  if (isTauriRuntime()) {
    await invoke("register_hotkey", { soundId, keyCombination });
    return;
  }
  const found = browserFallbackState.sounds.find((s) => s.id === soundId);
  if (found) found.hotkey = keyCombination;
}

export async function ipcUnregisterHotkey(soundId: string): Promise<void> {
  if (isTauriRuntime()) {
    await invoke("unregister_hotkey", { soundId });
    return;
  }
  const found = browserFallbackState.sounds.find((s) => s.id === soundId);
  if (found) found.hotkey = null;
}

export async function ipcSetMixerVolumes(
  micVol: number,
  soundboardVol: number,
  monitorVol: number
): Promise<void> {
  webAudioEngine.setMixerVolumes(micVol, soundboardVol, monitorVol);
  browserFallbackState.mixer.micVol = micVol;
  browserFallbackState.mixer.soundboardVol = soundboardVol;
  browserFallbackState.mixer.monitorVol = monitorVol;
  if (isTauriRuntime()) {
    await invoke("set_mixer_volumes", { micVol, soundboardVol, monitorVol });
  }
}

export async function ipcSetVoiceFx(
  pitch: number,
  reverb: number,
  distortion: number
): Promise<void> {
  webAudioEngine.setVoiceFx({ pitch, reverb, distortion });
  browserFallbackState.mixer.voiceFx = { pitch, reverb, distortion };
  if (isTauriRuntime()) {
    await invoke("set_voice_fx", { pitch, reverb, distortion });
  }
}

export async function ipcGetMixerState(): Promise<MixerState> {
  if (isTauriRuntime()) {
    const state = await invoke<MixerState>("get_mixer_state");
    webAudioEngine.setMixerVolumes(state.micVol, state.soundboardVol, state.monitorVol);
    webAudioEngine.setVoiceFx(state.voiceFx);
    return state;
  }
  return { ...browserFallbackState.mixer };
}

export async function subscribeAudioLevels(
  onLevels: (payload: AudioLevelPayload) => void
): Promise<UnlistenFn> {
  let tauriUnlisten: UnlistenFn | null = null;
  if (isTauriRuntime()) {
    tauriUnlisten = await listen<AudioLevelPayload>("audio-levels", (event) => {
      const webStats = webAudioEngine.getRealtimeRmsAndActiveIds();
      const mergedActive = Array.from(
        new Set([...event.payload.activeSoundIds, ...webStats.activeIds])
      );
      const sbLevel = Math.max(
        event.payload.soundboardLevel,
        webStats.rms * browserFallbackState.mixer.soundboardVol
      );
      const monLevel = Math.max(
        event.payload.monitorLevel,
        sbLevel * browserFallbackState.mixer.monitorVol
      );
      onLevels({
        ...event.payload,
        soundboardLevel: sbLevel,
        monitorLevel: monLevel,
        soundboardDb: sbLevel <= 0.0005 ? -60 : Math.max(-60, 20 * Math.log10(sbLevel)),
        monitorDb: monLevel <= 0.0005 ? -60 : Math.max(-60, 20 * Math.log10(monLevel)),
        activeSoundIds: mergedActive,
      });
    });
    return () => {
      if (tauriUnlisten) tauriUnlisten();
    };
  }

  const timer = window.setInterval(() => {
    if (browserFallbackState.mixer.panicMuted) {
      onLevels({
        micDb: -60,
        soundboardDb: -60,
        monitorDb: -60,
        micLevel: 0,
        soundboardLevel: 0,
        monitorLevel: 0,
        activeSoundIds: [],
        panicMuted: true,
      });
      return;
    }
    const { rms, activeIds } = webAudioEngine.getRealtimeRmsAndActiveIds();
    const sbLevel = Math.min(1, rms * browserFallbackState.mixer.soundboardVol);
    const monLevel = Math.min(1, sbLevel * browserFallbackState.mixer.monitorVol);
    const micIdleFloor =
      activeIds.length > 0 ? 0.08 * browserFallbackState.mixer.micVol : 0.02 * browserFallbackState.mixer.micVol;
    onLevels({
      micDb: micIdleFloor <= 0.0005 ? -60 : Math.max(-60, 20 * Math.log10(micIdleFloor)),
      soundboardDb: sbLevel <= 0.0005 ? -60 : Math.max(-60, 20 * Math.log10(sbLevel)),
      monitorDb: monLevel <= 0.0005 ? -60 : Math.max(-60, 20 * Math.log10(monLevel)),
      micLevel: micIdleFloor,
      soundboardLevel: sbLevel,
      monitorLevel: monLevel,
      activeSoundIds: activeIds,
      panicMuted: false,
    });
  }, 50);

  return () => window.clearInterval(timer);
}

export async function subscribeHotkeyEvents(
  onTrigger: (payload: HotkeyTriggerEvent) => void
): Promise<UnlistenFn> {
  if (isTauriRuntime()) {
    return listen<HotkeyTriggerEvent>("hotkey-triggered", (e) => onTrigger(e.payload));
  }
  return () => {};
}

export async function windowMinimize(): Promise<void> {
  if (isTauriRuntime()) {
    await getCurrentWindow().minimize();
  }
}

export async function windowToggleMaximize(): Promise<void> {
  if (isTauriRuntime()) {
    await getCurrentWindow().toggleMaximize();
  }
}

export async function windowClose(): Promise<void> {
  if (isTauriRuntime()) {
    await getCurrentWindow().close();
  }
}
