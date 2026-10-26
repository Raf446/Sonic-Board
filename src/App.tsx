import React, { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  AudioWaveform,
  CheckCircle2,
  Loader2,
  Plus,
  RotateCcw,
  Upload,
} from "lucide-react";
import { AudioControlSidebar } from "./components/AudioControlSidebar";
import { HeaderBar } from "./components/HeaderBar";
import { CategoryModal, SoundModal } from "./components/Modals";
import { SoundPadCard } from "./components/SoundPadCard";
import {
  DEFAULT_CATEGORIES,
  FALLBACK_STARTER_SOUNDS,
  ipcAddCategory,
  ipcAddSound,
  ipcGetAudioDevices,
  ipcGetCategories,
  ipcGetConnectionStatus,
  ipcGetMixerState,
  ipcGetSounds,
  ipcPlaySound,
  ipcRegisterHotkey,
  ipcRemoveSound,
  ipcSetMixerVolumes,
  ipcSetPanicMute,
  ipcSetRouting,
  ipcSetSoundVolume,
  ipcSetVoiceFx,
  ipcStopAllSounds,
  ipcStopSound,
  ipcUnregisterHotkey,
  ipcUpdateSound,
  subscribeAudioLevels,
  subscribeHotkeyEvents,
  windowClose,
  windowMinimize,
  windowToggleMaximize,
} from "./lib/tauriIpc";
import {
  AudioDevicesResponse,
  AudioLevelPayload,
  MixerState,
  RoutingStatus,
  SoundItem,
  VoiceFxState,
} from "./types/soundboard";

export const App: React.FC = () => {
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [loading, setLoading] = useState<boolean>(true);
  const [errorBanner, setErrorBanner] = useState<string | null>(null);
  const [statusToast, setStatusToast] = useState<string | null>(null);

  const [sounds, setSounds] = useState<SoundItem[]>(FALLBACK_STARTER_SOUNDS);
  const [categories, setCategories] = useState<string[]>(DEFAULT_CATEGORIES);
  const [activeCategory, setActiveCategory] = useState<string>("Semua");
  const [searchQuery, setSearchQuery] = useState<string>("");

  const [devices, setDevices] = useState<AudioDevicesResponse>({
    inputs: [],
    outputs: [],
  });
  const [routing, setRouting] = useState<RoutingStatus>({
    connected: true,
    micDeviceId: "",
    micDeviceName: "Loading...",
    virtualMicId: "",
    virtualMicName: "Loading...",
    monitorHeadphoneId: "",
    monitorHeadphoneName: "Loading...",
    virtualDriverDetected: true,
    sampleRate: 48000,
    bufferSizeFrames: 256,
    estimatedLatencyMs: 5.3,
    statusMessage: "Initializing WASAPI / CPAL audio streams...",
    panicMuted: false,
  });
  const [mixer, setMixer] = useState<MixerState>({
    micVol: 0.85,
    soundboardVol: 0.9,
    monitorVol: 0.8,
    voiceFx: { pitch: 0, reverb: 0, distortion: 0 },
    panicMuted: false,
  });
  const [levels, setLevels] = useState<AudioLevelPayload>({
    micDb: -60,
    soundboardDb: -60,
    monitorDb: -60,
    micLevel: 0,
    soundboardLevel: 0,
    monitorLevel: 0,
    activeSoundIds: [],
    panicMuted: false,
  });

  const [isSoundModalOpen, setIsSoundModalOpen] = useState(false);
  const [editingSound, setEditingSound] = useState<SoundItem | null>(null);
  const [droppedFilePath, setDroppedFilePath] = useState<string | undefined>(undefined);
  const [droppedFileName, setDroppedFileName] = useState<string | undefined>(undefined);
  const [isCategoryModalOpen, setIsCategoryModalOpen] = useState(false);
  const [isDragOverDropzone, setIsDragOverDropzone] = useState(false);

  const showToast = useCallback((msg: string) => {
    setStatusToast(msg);
  }, []);

  useEffect(() => {
    if (!statusToast) return;
    const t = window.setTimeout(() => setStatusToast(null), 3200);
    return () => window.clearTimeout(t);
  }, [statusToast]);

  useEffect(() => {
    const root = document.documentElement;
    if (theme === "light") {
      root.classList.remove("dark");
      root.classList.add("light");
    } else {
      root.classList.remove("light");
      root.classList.add("dark");
    }
  }, [theme]);

  const loadInitialState = useCallback(async () => {
    setLoading(true);
    setErrorBanner(null);
    try {
      const [devRes, routeRes, sndRes, catRes, mixRes] = await Promise.all([
        ipcGetAudioDevices(),
        ipcGetConnectionStatus(),
        ipcGetSounds(),
        ipcGetCategories(),
        ipcGetMixerState(),
      ]);
      setDevices(devRes);
      setRouting(routeRes);
      setSounds(sndRes);
      setCategories(catRes);
      setMixer(mixRes);
    } catch (err) {
      setErrorBanner(
        err instanceof Error
          ? `Gagal memuat engine audio: ${err.message}`
          : "Gagal menginisialisasi koneksi IPC audio."
      );
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadInitialState();
  }, [loadInitialState]);

  useEffect(() => {
    let cancelled = false;
    let unlistenLevels: (() => void) | null = null;
    let unlistenHotkeys: (() => void) | null = null;

    subscribeAudioLevels((payload) => {
      if (!cancelled) setLevels(payload);
    }).then((fn) => {
      if (cancelled) fn();
      else unlistenLevels = fn;
    });

    subscribeHotkeyEvents((hkEvent) => {
      if (!cancelled) showToast(`Hotkey aktif: ${hkEvent.keyCombination}`);
    }).then((fn) => {
      if (cancelled) fn();
      else unlistenHotkeys = fn;
    });

    return () => {
      cancelled = true;
      if (unlistenLevels) unlistenLevels();
      if (unlistenHotkeys) unlistenHotkeys();
    };
  }, [showToast]);

  // Support in-window keyboard shortcuts (Ctrl+K search focus, registered hotkeys, and Pause/F12 Panic Mute).
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const isInputFocused =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT");

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        const searchEl = document.querySelector<HTMLInputElement>('input[type="search"]');
        searchEl?.focus();
        return;
      }

      if (isInputFocused || isSoundModalOpen || isCategoryModalOpen) {
        return;
      }

      if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) {
        return;
      }

      const parts: string[] = [];
      if (e.ctrlKey) parts.push("Ctrl");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");

      let mainKey = e.key.toUpperCase();
      if (e.code.startsWith("Numpad")) {
        mainKey = `Num ${e.code.replace("Numpad", "")}`;
      } else if (e.code.startsWith("Digit")) {
        mainKey = e.code.replace("Digit", "");
      } else if (e.code === "Space") {
        mainKey = "Space";
      }
      parts.push(mainKey);
      const combo = parts.join(" + ");

      const matched = sounds.find(
        (s) => s.hotkey && s.hotkey.toLowerCase() === combo.toLowerCase()
      );
      if (matched) {
        e.preventDefault();
        ipcPlaySound(matched).catch((err) => setErrorBanner(String(err)));
        showToast(`Trigger ${matched.name} [${matched.hotkey}]`);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [sounds, isSoundModalOpen, isCategoryModalOpen, showToast]);

  const soundCountByCategory = useMemo(() => {
    const map: Record<string, number> = { Favorit: 0 };
    for (const c of categories) {
      if (!(c in map)) map[c] = 0;
    }
    for (const s of sounds) {
      if (s.favorite) map.Favorit = (map.Favorit || 0) + 1;
      map[s.category] = (map[s.category] || 0) + 1;
    }
    return map;
  }, [sounds, categories]);

  const filteredSounds = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    return sounds.filter((s) => {
      const matchesCat =
        activeCategory === "Semua"
          ? true
          : activeCategory === "Favorit"
          ? s.favorite
          : s.category === activeCategory;
      if (!matchesCat) return false;
      if (!q) return true;
      return (
        s.name.toLowerCase().includes(q) ||
        s.category.toLowerCase().includes(q) ||
        (s.hotkey && s.hotkey.toLowerCase().includes(q))
      );
    });
  }, [sounds, activeCategory, searchQuery]);

  const handlePlayToggle = async (sound: SoundItem) => {
    try {
      const currentlyPlaying = levels.activeSoundIds.includes(sound.id);
      if (currentlyPlaying) {
        await ipcStopSound(sound.id);
      } else {
        await ipcPlaySound(sound);
        setRouting((prev) => ({ ...prev, panicMuted: false, connected: true }));
        setMixer((prev) => ({ ...prev, panicMuted: false }));
      }
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleSoundVolumeChange = async (soundId: string, volume: number) => {
    setSounds((prev) =>
      prev.map((s) => (s.id === soundId ? { ...s, volume } : s))
    );
    try {
      await ipcSetSoundVolume(soundId, volume);
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleToggleFavorite = async (sound: SoundItem) => {
    try {
      const updated = await ipcUpdateSound(
        sound.id,
        sound.name,
        sound.hotkey,
        sound.category,
        !sound.favorite,
        sound.icon
      );
      setSounds((prev) => prev.map((s) => (s.id === sound.id ? updated : s)));
      showToast(
        updated.favorite
          ? `${updated.name} ditambahkan ke Favorit`
          : `${updated.name} dihapus dari Favorit`
      );
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDeleteSound = async (sound: SoundItem) => {
    try {
      await ipcRemoveSound(sound.id);
      setSounds((prev) => prev.filter((s) => s.id !== sound.id));
      showToast(`Suara "${sound.name}" dihapus`);
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleSaveSoundModal = async (data: {
    soundId?: string;
    name: string;
    filePath: string;
    hotkey: string | null;
    category: string;
    icon: string;
    favorite: boolean;
  }) => {
    if (data.soundId) {
      const updated = await ipcUpdateSound(
        data.soundId,
        data.name,
        data.hotkey,
        data.category,
        data.favorite,
        data.icon
      );
      if (data.hotkey) {
        await ipcRegisterHotkey(data.soundId, data.hotkey);
      } else {
        await ipcUnregisterHotkey(data.soundId);
      }
      setSounds((prev) => prev.map((s) => (s.id === data.soundId ? updated : s)));
      showToast(`Suara "${updated.name}" diperbarui`);
    } else {
      const created = await ipcAddSound(
        data.name,
        data.filePath,
        data.hotkey,
        data.category,
        data.icon
      );
      if (data.favorite) {
        const favUpdated = await ipcUpdateSound(
          created.id,
          created.name,
          created.hotkey,
          created.category,
          true,
          created.icon
        );
        setSounds((prev) => [...prev, favUpdated]);
      } else {
        setSounds((prev) => [...prev, created]);
      }
      showToast(`Suara baru "${created.name}" berhasil ditambahkan`);
    }
  };

  const handleAddCategory = async (catName: string) => {
    const updatedCats = await ipcAddCategory(catName);
    setCategories(updatedCats);
    setActiveCategory(catName);
    showToast(`Kategori "${catName}" ditambahkan`);
  };

  const handleRoutingChange = async (
    micId: string,
    virtualId: string,
    monitorId: string
  ) => {
    try {
      await ipcSetRouting(micId, virtualId, monitorId);
      const updatedStatus = await ipcGetConnectionStatus();
      setRouting(updatedStatus);
      showToast("Routing I/O audio berhasil diperbarui");
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRefreshDevices = async () => {
    try {
      const [devRes, statusRes] = await Promise.all([
        ipcGetAudioDevices(),
        ipcGetConnectionStatus(),
      ]);
      setDevices(devRes);
      setRouting(statusRes);
      showToast("Daftar perangkat audio diperbarui");
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleMixerChange = async (
    micVol: number,
    soundboardVol: number,
    monitorVol: number
  ) => {
    setMixer((prev) => ({ ...prev, micVol, soundboardVol, monitorVol }));
    try {
      await ipcSetMixerVolumes(micVol, soundboardVol, monitorVol);
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleVoiceFxChange = async (voiceFx: VoiceFxState) => {
    setMixer((prev) => ({ ...prev, voiceFx }));
    try {
      await ipcSetVoiceFx(voiceFx.pitch, voiceFx.reverb, voiceFx.distortion);
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handlePanicMute = async () => {
    try {
      await ipcStopAllSounds();
      const status = await ipcGetConnectionStatus();
      setRouting(status);
      setMixer((prev) => ({ ...prev, panicMuted: true }));
      setLevels((prev) => ({
        ...prev,
        micDb: -60,
        soundboardDb: -60,
        monitorDb: -60,
        micLevel: 0,
        soundboardLevel: 0,
        monitorLevel: 0,
        activeSoundIds: [],
        panicMuted: true,
      }));
      showToast("PANIC MUTE AKTIF: Semua output audio dan mic dihentikan");
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleResumeAudio = async () => {
    try {
      await ipcSetPanicMute(false);
      const status = await ipcGetConnectionStatus();
      setRouting(status);
      setMixer((prev) => ({ ...prev, panicMuted: false }));
      showToast("Routing audio dipulihkan");
    } catch (err) {
      setErrorBanner(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDropZoneDrop = (e: React.DragEvent<HTMLButtonElement>) => {
    e.preventDefault();
    setIsDragOverDropzone(false);
    const file = e.dataTransfer.files?.[0];
    if (file) {
      const cleanTitle = file.name.replace(/\.[^/.]+$/, "").replace(/[-_]/g, " ");
      const reader = new FileReader();
      reader.onload = () => {
        if (typeof reader.result === "string" && reader.result.startsWith("data:")) {
          setEditingSound(null);
          setDroppedFilePath(reader.result);
          setDroppedFileName(cleanTitle);
          setIsSoundModalOpen(true);
        }
      };
      reader.readAsDataURL(file);
    }
  };

  return (
    <div
      className={`min-h-screen flex flex-col ${
        theme === "light"
          ? "bg-[#F1F5F9] text-[#0F172A]"
          : "bg-[#12131A] text-[#F3F4F6]"
      }`}
    >
      {/* Section 1: Header & Category Filter Bar */}
      <HeaderBar
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        categories={categories}
        activeCategory={activeCategory}
        onSelectCategory={setActiveCategory}
        onOpenNewCategoryModal={() => setIsCategoryModalOpen(true)}
        onOpenAddSoundModal={() => {
          setEditingSound(null);
          setDroppedFilePath(undefined);
          setDroppedFileName(undefined);
          setIsSoundModalOpen(true);
        }}
        soundCountByCategory={soundCountByCategory}
        totalSounds={sounds.length}
        activeOutputLevel={levels.soundboardLevel}
        isPlayingAny={levels.activeSoundIds.length > 0}
        panicMuted={routing.panicMuted || mixer.panicMuted}
        theme={theme}
        onToggleTheme={() => setTheme((prev) => (prev === "dark" ? "light" : "dark"))}
        onMinimizeWindow={() => {
          windowMinimize();
          showToast("Jendela diminimalkan");
        }}
        onMaximizeWindow={() => {
          windowToggleMaximize();
          showToast("Ukuran jendela diubah");
        }}
        onCloseWindow={() => {
          windowClose();
        }}
      />

      {/* Diagnostic Error State Banner (R-27) */}
      {errorBanner && (
        <div
          role="alert"
          className="mx-4 mt-3 px-4 py-2.5 rounded-md bg-[#EF4444]/15 border border-[#EF4444] flex items-center justify-between gap-3 text-xs text-[#FCA5A5]"
        >
          <div className="flex items-center gap-2">
            <AlertTriangle className="w-4 h-4 text-[#EF4444] shrink-0" aria-hidden="true" />
            <span>{errorBanner}</span>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <button
              type="button"
              onClick={loadInitialState}
              className="px-2.5 py-1 rounded bg-[#EF4444] text-[#090A0F] font-semibold hover:bg-[#EF4444]/90"
            >
              Muat Ulang
            </button>
            <button
              type="button"
              onClick={() => setErrorBanner(null)}
              className="px-2 py-1 rounded bg-[#232734] text-[#F3F4F6]"
            >
              Tutup
            </button>
          </div>
        </div>
      )}

      {/* Main Studio Workspace: Center Soundboard Grid + Right Audio Control Center */}
      <div className="flex-1 flex flex-col lg:flex-row overflow-hidden">
        <main
          aria-label="Matriks Kartu Soundboard"
          className="flex-1 p-4 sm:p-5 overflow-y-auto"
        >
          {/* Status Header above Grid */}
          <div className="flex flex-wrap items-center justify-between gap-2 mb-4">
            <div className="flex items-center gap-2">
              <AudioWaveform
                className="w-4 h-4 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700"
                aria-hidden="true"
              />
              <h1 className="text-sm font-bold tracking-wide uppercase text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900">
                Kategori: {activeCategory}
              </h1>
              <span className="font-mono text-xs text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600">
                ({filteredSounds.length} sampel siap)
              </span>
            </div>

            {statusToast && (
              <div
                role="status"
                aria-live="polite"
                className="inline-flex items-center gap-1.5 px-3 py-1 rounded bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border border-[#00F0FF]/60 text-xs font-mono text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-800"
              >
                <CheckCircle2 className="w-3.5 h-3.5" aria-hidden="true" />
                <span>{statusToast}</span>
              </div>
            )}
          </div>

          {/* Loading State (R-27) */}
          {loading ? (
            <div
              role="status"
              className="rounded-lg bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border border-[#2E3446] p-8 flex flex-col items-center justify-center text-center gap-3"
            >
              <Loader2 className="w-7 h-7 text-[#00F0FF] animate-spin" aria-hidden="true" />
              <p className="text-sm font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900">
                Memindai Perangkat Audio WASAPI / CPAL & Memuat Sampel...
              </p>
              <p className="text-xs text-[#9CA3AF]">
                Menyiapkan jalur dual-sink untuk mikrofon fisik dan virtual audio cable.
              </p>
            </div>
          ) : (
            <>
              {/* Empty Filter / Search State (R-27) */}
              {filteredSounds.length === 0 && (
                <div
                  role="status"
                  className="mb-4 rounded-lg bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 p-6 flex flex-col sm:flex-row items-center justify-between gap-4"
                >
                  <div>
                    <h2 className="text-sm font-bold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900">
                      Tidak ada sampel suara di tampilan ini
                    </h2>
                    <p className="text-xs text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600 mt-1">
                      {searchQuery
                        ? `Tidak ditemukan suara atau hotkey yang cocok dengan "${searchQuery}" pada kategori ${activeCategory}.`
                        : `Kategori "${activeCategory}" belum memiliki sampel suara. Tambahkan file MP3/WAV baru atau reset filter kategori.`}
                    </p>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    {(searchQuery || activeCategory !== "Semua") && (
                      <button
                        type="button"
                        onClick={() => {
                          setSearchQuery("");
                          setActiveCategory("Semua");
                        }}
                        className="h-9 px-3.5 rounded bg-[#232734] text-xs font-semibold text-[#F3F4F6] hover:bg-[#2E3446] flex items-center gap-1.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
                      >
                        <RotateCcw className="w-3.5 h-3.5" aria-hidden="true" />
                        <span>Reset Filter</span>
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() => {
                        setEditingSound(null);
                        setIsSoundModalOpen(true);
                      }}
                      className="h-9 px-3.5 rounded bg-[#00F0FF] text-[#090A0F] text-xs font-bold hover:bg-[#00F0FF]/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
                    >
                      + Tambah Suara
                    </button>
                  </div>
                </div>
              )}

              {/* Responsive 3-State Soundboard Grid (1 col mobile -> 2 col tablet -> 3 col desktop) */}
              <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3.5">
                {filteredSounds.map((sound) => (
                  <SoundPadCard
                    key={sound.id}
                    sound={sound}
                    isPlaying={levels.activeSoundIds.includes(sound.id)}
                    outputLevel={levels.soundboardLevel}
                    onPlayToggle={handlePlayToggle}
                    onVolumeChange={handleSoundVolumeChange}
                    onToggleFavorite={handleToggleFavorite}
                    onEditSound={(snd) => {
                      setEditingSound(snd);
                      setDroppedFilePath(undefined);
                      setDroppedFileName(undefined);
                      setIsSoundModalOpen(true);
                    }}
                    onDeleteSound={handleDeleteSound}
                  />
                ))}

                {/* Dashed-Border Drop-Zone Card (Last Item in Grid) */}
                <button
                  type="button"
                  onClick={() => {
                    setEditingSound(null);
                    setDroppedFilePath(undefined);
                    setDroppedFileName(undefined);
                    setIsSoundModalOpen(true);
                  }}
                  onDragOver={(e) => {
                    e.preventDefault();
                    setIsDragOverDropzone(true);
                  }}
                  onDragLeave={() => setIsDragOverDropzone(false)}
                  onDrop={handleDropZoneDrop}
                  aria-label="Tambah Suara Baru (Import MP3/WAV, setting hotkey)"
                  className={`min-h-[172px] rounded-lg p-4 border-2 border-dashed transition flex flex-col items-center justify-center text-center gap-2.5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] ${
                    isDragOverDropzone
                      ? "bg-[#00F0FF]/10 border-[#00F0FF] text-[#00F0FF]"
                      : "bg-[#1A1D26]/60 dark:bg-[#1A1D26]/60 light:bg-white border-[#3B4259] dark:border-[#3B4259] light:border-slate-400 hover:border-[#00F0FF] hover:bg-[#1A1D26]"
                  }`}
                >
                  <div className="w-11 h-11 rounded-full bg-[#232734] dark:bg-[#232734] light:bg-slate-200 flex items-center justify-center text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700">
                    {isDragOverDropzone ? (
                      <Upload className="w-5 h-5" aria-hidden="true" />
                    ) : (
                      <Plus className="w-5 h-5" aria-hidden="true" />
                    )}
                  </div>
                  <div>
                    <p className="text-xs font-bold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900">
                      + Tambah Suara Baru (Import MP3/WAV, setting hotkey)
                    </p>
                    <p className="text-[11px] text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600 mt-1">
                      Klik atau tarik file MP3, WAV, OGG, FLAC ke kartu ini
                    </p>
                  </div>
                </button>
              </div>
            </>
          )}
        </main>

        {/* Section 3: Right Sidebar (Audio Control Center) */}
        <AudioControlSidebar
          devices={devices}
          routing={routing}
          mixer={mixer}
          levels={levels}
          onChangeRouting={handleRoutingChange}
          onRefreshDevices={handleRefreshDevices}
          onChangeMixer={handleMixerChange}
          onChangeVoiceFx={handleVoiceFxChange}
          onTriggerPanicMute={handlePanicMute}
          onResumeAudio={handleResumeAudio}
        />
      </div>

      {/* Add / Edit Sound Modal */}
      <SoundModal
        isOpen={isSoundModalOpen}
        editingSound={editingSound}
        categories={categories}
        initialFilePath={droppedFilePath}
        initialName={droppedFileName}
        onClose={() => setIsSoundModalOpen(false)}
        onSave={handleSaveSoundModal}
      />

      {/* New Category Modal */}
      <CategoryModal
        isOpen={isCategoryModalOpen}
        onClose={() => setIsCategoryModalOpen(false)}
        onAddCategory={handleAddCategory}
      />
    </div>
  );
};

export default App;
