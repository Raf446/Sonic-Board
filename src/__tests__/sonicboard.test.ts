import { describe, expect, it } from "vitest";
import {
  DEFAULT_CATEGORIES,
  FALLBACK_STARTER_SOUNDS,
  ipcAddCategory,
  ipcAddSound,
  ipcGetAudioDevices,
  ipcGetConnectionStatus,
  ipcGetMixerState,
  ipcGetSounds,
  ipcPlaySound,
  ipcRegisterHotkey,
  ipcSetMixerVolumes,
  ipcSetRouting,
  ipcSetSoundVolume,
  ipcSetVoiceFx,
  ipcStopAllSounds,
  ipcUnregisterHotkey,
} from "../lib/tauriIpc";

describe("SonicBoard IPC & Audio Engine Contracts", () => {
  it("enumerates physical and virtual audio devices and updates routing", async () => {
    const devices = await ipcGetAudioDevices();
    expect(devices.inputs.length).toBeGreaterThan(0);
    expect(devices.outputs.length).toBeGreaterThan(0);

    const virtualSink = devices.outputs.find((d) => d.isVirtual) || devices.outputs[0];
    await ipcSetRouting(
      devices.inputs[0].id,
      virtualSink.id,
      devices.outputs[0].id
    );

    const status = await ipcGetConnectionStatus();
    expect(status.connected).toBe(true);
    expect(status.virtualMicId).toBe(virtualSink.id);
  });

  it("loads starter sounds, adds new sound, adjusts volume, and registers hotkeys", async () => {
    const initial = await ipcGetSounds();
    expect(initial.length).toBeGreaterThanOrEqual(FALLBACK_STARTER_SOUNDS.length);

    const created = await ipcAddSound(
      "Test Siren FX",
      "/samples/clutch-siren.wav",
      "Ctrl + 8",
      "Sound FX",
      "radio"
    );
    expect(created.name).toBe("Test Siren FX");
    expect(created.waveform.length).toBe(28);

    await ipcSetSoundVolume(created.id, 0.55);
    await ipcRegisterHotkey(created.id, "Alt + 2");
    const afterBind = (await ipcGetSounds()).find((s) => s.id === created.id);
    expect(afterBind?.hotkey).toBe("Alt + 2");
    expect(afterBind?.volume).toBe(0.55);

    await ipcUnregisterHotkey(created.id);
    const afterUnbind = (await ipcGetSounds()).find((s) => s.id === created.id);
    expect(afterUnbind?.hotkey).toBeNull();
  });

  it("manages categories, mixer faders, DSP knobs, and Emergency Panic Mute", async () => {
    const cats = await ipcAddCategory("Stream Stingers");
    expect(cats).toContain("Stream Stingers");
    expect(cats.length).toBeGreaterThan(DEFAULT_CATEGORIES.length);

    await ipcSetMixerVolumes(0.7, 0.85, 0.65);
    await ipcSetVoiceFx(4, 0.35, 0.2);

    const mixer = await ipcGetMixerState();
    expect(mixer.micVol).toBe(0.7);
    expect(mixer.soundboardVol).toBe(0.85);
    expect(mixer.monitorVol).toBe(0.65);
    expect(mixer.voiceFx.pitch).toBe(4);

    const sounds = await ipcGetSounds();
    await ipcPlaySound(sounds[0]);
    await ipcStopAllSounds();

    const postPanicStatus = await ipcGetConnectionStatus();
    expect(postPanicStatus.panicMuted).toBe(true);
    expect(postPanicStatus.connected).toBe(false);
  });
});
