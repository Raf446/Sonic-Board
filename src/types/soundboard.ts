export interface DeviceInfo {
  id: string;
  name: string;
  kind: "input" | "output" | string;
  isDefault: boolean;
  isVirtual: boolean;
  hostApi: string;
  sampleRate: number;
  channels: number;
}

export interface AudioDevicesResponse {
  inputs: DeviceInfo[];
  outputs: DeviceInfo[];
}

export interface RoutingStatus {
  connected: boolean;
  micDeviceId: string;
  micDeviceName: string;
  virtualMicId: string;
  virtualMicName: string;
  monitorHeadphoneId: string;
  monitorHeadphoneName: string;
  virtualDriverDetected: boolean;
  sampleRate: number;
  bufferSizeFrames: number;
  estimatedLatencyMs: number;
  statusMessage: string;
  panicMuted: boolean;
}

export interface SoundItem {
  id: string;
  name: string;
  filePath: string;
  hotkey: string | null;
  category: string;
  volume: number;
  durationSecs: number;
  favorite: boolean;
  icon: string;
  waveform: number[];
}

export interface VoiceFxState {
  pitch: number;
  reverb: number;
  distortion: number;
}

export interface MixerState {
  micVol: number;
  soundboardVol: number;
  monitorVol: number;
  voiceFx: VoiceFxState;
  panicMuted: boolean;
}

export interface AudioLevelPayload {
  micDb: number;
  soundboardDb: number;
  monitorDb: number;
  micLevel: number;
  soundboardLevel: number;
  monitorLevel: number;
  activeSoundIds: string[];
  panicMuted: boolean;
}

export interface HotkeyTriggerEvent {
  soundId: string;
  keyCombination: string;
  timestampMs: number;
}
