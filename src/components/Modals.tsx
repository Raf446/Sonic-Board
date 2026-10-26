import React, { useEffect, useState } from "react";
import { FolderPlus, Keyboard, Upload, Volume2, X } from "lucide-react";
import { SoundItem } from "../types/soundboard";
import { AVAILABLE_ICONS, SoundIcon } from "./SoundIcon";

const BUILTIN_SAMPLE_PRESETS = [
  { label: "Bass Drop Impact (WAV)", path: "/samples/vine-boom.wav" },
  { label: "Tactical Clutch Alert (WAV)", path: "/samples/clutch-siren.wav" },
  { label: "Synth Chime Spark (WAV)", path: "/samples/anime-wow.wav" },
  { label: "Stadium Airhorn (WAV)", path: "/samples/airhorn-triple.wav" },
  { label: "Victory Brass Chord (WAV)", path: "/samples/victory-fanfare.wav" },
  { label: "Mecha Laser Charge (WAV)", path: "/samples/power-charge.wav" },
  { label: "Broadcast 1kHz Bleep (WAV)", path: "/samples/censor-beep.wav" },
  { label: "Cyber Stutter Sweep (WAV)", path: "/samples/glitch-transition.wav" },
];

interface SoundModalProps {
  isOpen: boolean;
  editingSound: SoundItem | null;
  categories: string[];
  initialFilePath?: string;
  initialName?: string;
  onClose: () => void;
  onSave: (data: {
    soundId?: string;
    name: string;
    filePath: string;
    hotkey: string | null;
    category: string;
    icon: string;
    favorite: boolean;
  }) => Promise<void>;
}

export const SoundModal: React.FC<SoundModalProps> = ({
  isOpen,
  editingSound,
  categories,
  initialFilePath,
  initialName,
  onClose,
  onSave,
}) => {
  const [name, setName] = useState("");
  const [filePath, setFilePath] = useState("/samples/vine-boom.wav");
  const [hotkey, setHotkey] = useState("");
  const [category, setCategory] = useState("Memes");
  const [icon, setIcon] = useState("speaker");
  const [favorite, setFavorite] = useState(false);
  const [isRecordingKey, setIsRecordingKey] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    setErrorMsg(null);
    setIsRecordingKey(false);
    if (editingSound) {
      setName(editingSound.name);
      setFilePath(editingSound.filePath);
      setHotkey(editingSound.hotkey || "");
      setCategory(editingSound.category);
      setIcon(editingSound.icon || "speaker");
      setFavorite(editingSound.favorite);
    } else {
      setName(initialName || "");
      setFilePath(initialFilePath || "/samples/vine-boom.wav");
      setHotkey("");
      setCategory(categories.find((c) => c !== "Favorit") || "Memes");
      setIcon("speaker");
      setFavorite(false);
    }
  }, [isOpen, editingSound, initialFilePath, initialName, categories]);

  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (isRecordingKey) {
        e.preventDefault();
        e.stopPropagation();
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
        } else if (e.code === "Space") {
          mainKey = "Space";
        }
        parts.push(mainKey);
        setHotkey(parts.join(" + "));
        setIsRecordingKey(false);
        return;
      }

      if (e.key === "Escape") {
        onClose();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, isRecordingKey, onClose]);

  if (!isOpen) return null;

  const handleFileInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (!name.trim()) {
      const baseName = file.name.replace(/\.[^/.]+$/, "").replace(/[-_]/g, " ");
      setName(baseName);
    }
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === "string" && reader.result.startsWith("data:")) {
        setFilePath(reader.result);
      }
    };
    reader.readAsDataURL(file);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setErrorMsg("Nama suara wajib diisi sebelum menyimpan.");
      return;
    }
    if (!filePath.trim()) {
      setErrorMsg("Pilih file audio (MP3/WAV/OGG/FLAC) atau sampel studio.");
      return;
    }
    setSubmitting(true);
    setErrorMsg(null);
    try {
      await onSave({
        soundId: editingSound?.id,
        name: name.trim(),
        filePath: filePath.trim(),
        hotkey: hotkey.trim() ? hotkey.trim() : null,
        category,
        icon,
        favorite,
      });
      onClose();
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="sound-modal-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4"
    >
      <div className="w-full max-w-lg rounded-lg bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 shadow-2xl overflow-hidden">
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-[#2E3446] dark:border-[#2E3446] light:border-slate-200">
          <div className="flex items-center gap-2">
            <Volume2 className="w-5 h-5 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700" aria-hidden="true" />
            <h2
              id="sound-modal-title"
              className="text-base font-bold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900"
            >
              {editingSound ? `Edit Suara: ${editingSound.name}` : "Tambah Suara Baru"}
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Tutup dialog"
            className="w-8 h-8 rounded flex items-center justify-center text-[#9CA3AF] hover:text-[#F3F4F6] hover:bg-[#232734] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
          >
            <X className="w-4 h-4" aria-hidden="true" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-5 space-y-4">
          {errorMsg && (
            <div
              role="alert"
              className="p-3 rounded bg-[#EF4444]/15 border border-[#EF4444] text-xs text-[#FCA5A5]"
            >
              {errorMsg}
            </div>
          )}

          {/* Sound Title */}
          <div>
            <label
              htmlFor="sound-name-input"
              className="block text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 mb-1"
            >
              Nama Suara
            </label>
            <input
              id="sound-name-input"
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Contoh: Tactical Clutch Horn"
              className="w-full h-10 px-3 rounded bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 text-sm text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 placeholder-[#8A94A6] focus:outline-none focus:ring-2 focus:ring-[#00F0FF]"
            />
          </div>

          {/* Audio Source File / Preset */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label
                htmlFor="sound-preset-select"
                className="block text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 mb-1"
              >
                Sampel Studio Bawaan
              </label>
              <select
                id="sound-preset-select"
                value={
                  BUILTIN_SAMPLE_PRESETS.some((p) => p.path === filePath)
                    ? filePath
                    : "custom"
                }
                onChange={(e) => {
                  if (e.target.value !== "custom") {
                    setFilePath(e.target.value);
                  }
                }}
                className="w-full h-10 px-2.5 rounded bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 text-xs text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 focus:outline-none focus:ring-2 focus:ring-[#00F0FF]"
              >
                {BUILTIN_SAMPLE_PRESETS.map((p) => (
                  <option key={p.path} value={p.path}>
                    {p.label}
                  </option>
                ))}
                <option value="custom">File Lokal Kustom...</option>
              </select>
            </div>

            <div>
              <label
                htmlFor="sound-file-upload"
                className="block text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 mb-1"
              >
                Import File (MP3 / WAV / OGG / FLAC)
              </label>
              <label
                htmlFor="sound-file-upload"
                className="h-10 px-3 rounded bg-[#232734] dark:bg-[#232734] light:bg-slate-200 border border-[#2E3446] hover:border-[#00F0FF] text-xs font-medium text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 flex items-center justify-center gap-2 cursor-pointer transition"
              >
                <Upload className="w-4 h-4 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700" aria-hidden="true" />
                <span className="truncate">Pilih File Audio Lokal</span>
                <input
                  id="sound-file-upload"
                  type="file"
                  accept=".mp3,.wav,.ogg,.flac,audio/*"
                  onChange={handleFileInputChange}
                  className="sr-only"
                />
              </label>
            </div>
          </div>

          {/* Category & Global Hotkey Recorder */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label
                htmlFor="sound-category-select"
                className="block text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 mb-1"
              >
                Kategori
              </label>
              <select
                id="sound-category-select"
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                className="w-full h-10 px-2.5 rounded bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 text-xs text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 focus:outline-none focus:ring-2 focus:ring-[#00F0FF]"
              >
                {categories
                  .filter((c) => c !== "Favorit")
                  .map((c) => (
                    <option key={c} value={c}>
                      {c}
                    </option>
                  ))}
              </select>
            </div>

            <div>
              <label
                htmlFor="sound-hotkey-input"
                className="block text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 mb-1"
              >
                Global Hotkey
              </label>
              <div className="flex items-center gap-1.5">
                <input
                  id="sound-hotkey-input"
                  type="text"
                  value={isRecordingKey ? "Tekan tombol..." : hotkey}
                  onChange={(e) => setHotkey(e.target.value)}
                  placeholder="Ctrl + 4 atau Num 6"
                  className="w-full h-10 px-3 rounded bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 font-mono text-xs text-[#EDE9FE] dark:text-[#EDE9FE] light:text-purple-900 placeholder-[#8A94A6] focus:outline-none focus:ring-2 focus:ring-[#8B5CF6]"
                />
                <button
                  type="button"
                  onClick={() => setIsRecordingKey((prev) => !prev)}
                  aria-label="Rekam kombinasi tombol keyboard"
                  className={`h-10 px-2.5 rounded text-xs font-mono font-semibold border shrink-0 flex items-center gap-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] ${
                    isRecordingKey
                      ? "bg-[#8B5CF6] text-white border-[#8B5CF6]"
                      : "bg-[#232734] dark:bg-[#232734] light:bg-slate-200 text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 border-[#2E3446]"
                  }`}
                >
                  <Keyboard className="w-3.5 h-3.5" aria-hidden="true" />
                  <span>{isRecordingKey ? "Stop" : "Bind"}</span>
                </button>
              </div>
            </div>
          </div>

          {/* Icon Selector */}
          <div>
            <span className="block text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 mb-1.5">
              Ikon Kartu Suara
            </span>
            <div className="grid grid-cols-6 gap-2">
              {AVAILABLE_ICONS.map((item) => {
                const selected = icon === item.id;
                return (
                  <button
                    key={item.id}
                    type="button"
                    onClick={() => setIcon(item.id)}
                    aria-label={item.label}
                    aria-pressed={selected}
                    title={item.label}
                    className={`h-10 rounded flex items-center justify-center border transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] ${
                      selected
                        ? "bg-[#00F0FF] text-[#090A0F] border-[#00F0FF]"
                        : "bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 text-[#9CA3AF] border-[#2E3446] hover:text-[#F3F4F6]"
                    }`}
                  >
                    <SoundIcon name={item.id} className="w-4 h-4" />
                  </button>
                );
              })}
            </div>
          </div>

          <div className="flex items-center justify-between pt-3 border-t border-[#2E3446] dark:border-[#2E3446] light:border-slate-200">
            <label className="inline-flex items-center gap-2 text-xs text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 cursor-pointer">
              <input
                type="checkbox"
                checked={favorite}
                onChange={(e) => setFavorite(e.target.checked)}
                className="w-4 h-4 rounded accent-[#00F0FF]"
              />
              <span>Pin ke Kategori Favorit</span>
            </label>

            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={onClose}
                className="h-10 px-4 rounded bg-[#232734] dark:bg-[#232734] light:bg-slate-200 text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 hover:bg-[#2E3446] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
              >
                Batal
              </button>
              <button
                type="submit"
                disabled={submitting}
                className="h-10 px-5 rounded bg-[#00F0FF] text-[#090A0F] text-xs font-bold hover:bg-[#00F0FF]/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] disabled:opacity-50"
              >
                {submitting ? "Menyimpan..." : editingSound ? "Simpan Perubahan" : "Tambahkan Suara"}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
};

interface CategoryModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAddCategory: (name: string) => Promise<void>;
}

export const CategoryModal: React.FC<CategoryModalProps> = ({
  isOpen,
  onClose,
  onAddCategory,
}) => {
  const [categoryName, setCategoryName] = useState("");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) return;
    setCategoryName("");
    setErrorMsg(null);
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!categoryName.trim()) {
      setErrorMsg("Masukkan nama kategori baru.");
      return;
    }
    try {
      await onAddCategory(categoryName.trim());
      onClose();
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="category-modal-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4"
    >
      <div className="w-full max-w-sm rounded-lg bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 shadow-2xl overflow-hidden">
        <div className="flex items-center justify-between px-5 py-3.5 border-b border-[#2E3446] dark:border-[#2E3446] light:border-slate-200">
          <div className="flex items-center gap-2">
            <FolderPlus className="w-4 h-4 text-[#A78BFA]" aria-hidden="true" />
            <h2
              id="category-modal-title"
              className="text-sm font-bold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900"
            >
              Tambah Kategori Baru
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Tutup modal kategori"
            className="w-8 h-8 rounded flex items-center justify-center text-[#9CA3AF] hover:text-[#F3F4F6] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
          >
            <X className="w-4 h-4" aria-hidden="true" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-5 space-y-4">
          {errorMsg && (
            <div role="alert" className="p-2.5 rounded bg-[#EF4444]/15 border border-[#EF4444] text-xs text-[#FCA5A5]">
              {errorMsg}
            </div>
          )}
          <div>
            <label
              htmlFor="new-category-name"
              className="block text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 mb-1"
            >
              Nama Kategori
            </label>
            <input
              id="new-category-name"
              type="text"
              autoFocus
              required
              value={categoryName}
              onChange={(e) => setCategoryName(e.target.value)}
              placeholder="Contoh: Podcast Intro"
              className="w-full h-10 px-3 rounded bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 border border-[#2E3446] text-sm text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 placeholder-[#8A94A6] focus:outline-none focus:ring-2 focus:ring-[#8B5CF6]"
            />
          </div>

          <div className="flex items-center justify-end gap-2">
            <button
              type="button"
              onClick={onClose}
              className="h-9 px-3.5 rounded bg-[#232734] text-xs font-semibold text-[#F3F4F6] hover:bg-[#2E3446] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
            >
              Batal
            </button>
            <button
              type="submit"
              className="h-9 px-4 rounded bg-[#8B5CF6] text-white text-xs font-bold hover:bg-[#8B5CF6]/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
            >
              Simpan Kategori
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
