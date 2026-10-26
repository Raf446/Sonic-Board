import React, { useRef } from "react";
import {
  FolderPlus,
  Maximize2,
  Minimize2,
  Moon,
  Search,
  Star,
  Sun,
  X,
} from "lucide-react";

interface HeaderBarProps {
  searchQuery: string;
  onSearchChange: (q: string) => void;
  categories: string[];
  activeCategory: string;
  onSelectCategory: (cat: string) => void;
  onOpenNewCategoryModal: () => void;
  onOpenAddSoundModal: () => void;
  soundCountByCategory: Record<string, number>;
  totalSounds: number;
  activeOutputLevel: number;
  isPlayingAny: boolean;
  panicMuted: boolean;
  theme: "dark" | "light";
  onToggleTheme: () => void;
  onMinimizeWindow: () => void;
  onMaximizeWindow: () => void;
  onCloseWindow: () => void;
}

export const HeaderBar: React.FC<HeaderBarProps> = ({
  searchQuery,
  onSearchChange,
  categories,
  activeCategory,
  onSelectCategory,
  onOpenNewCategoryModal,
  onOpenAddSoundModal,
  soundCountByCategory,
  totalSounds,
  activeOutputLevel,
  isPlayingAny,
  panicMuted,
  theme,
  onToggleTheme,
  onMinimizeWindow,
  onMaximizeWindow,
  onCloseWindow,
}) => {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const waveBars = [0.45, 0.85, 1.0, 0.65, 0.9, 0.5];

  return (
    <header className="border-b border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 bg-[#151720] dark:bg-[#151720] light:bg-white select-none">
      {/* Top Studio Transport & Window Bar */}
      <div
        data-tauri-drag-region
        className="flex flex-wrap items-center justify-between gap-3 px-4 py-2.5 border-b border-[#232734] dark:border-[#232734] light:border-slate-200"
      >
        {/* App Brand & Audio-Reactive Soundwave */}
        <div className="flex items-center gap-3">
          <div
            className="flex items-center gap-2.5 px-2.5 py-1.5 rounded-md bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300"
            title={
              panicMuted
                ? "Panic Mute Aktif"
                : isPlayingAny
                ? "Transmisi Audio Aktif"
                : "SonicBoard Audio Engine Siap"
            }
          >
            <div className="flex items-end gap-0.5 h-5 w-6" aria-hidden="true">
              {waveBars.map((baseHeight, idx) => {
                const dynamicScale = panicMuted
                  ? 0.2
                  : isPlayingAny
                  ? Math.max(0.28, Math.min(1, baseHeight * (0.45 + activeOutputLevel * 0.9)))
                  : baseHeight * 0.45;
                return (
                  <span
                    key={idx}
                    style={{ height: `${Math.round(dynamicScale * 100)}%` }}
                    className={`w-1 rounded-sm transition-all duration-75 ${
                      panicMuted
                        ? "bg-[#EF4444]"
                        : isPlayingAny
                        ? "bg-[#00F0FF]"
                        : "bg-[#00F0FF]/70"
                    }`}
                  />
                );
              })}
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold tracking-tight text-base text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900">
                  SonicBoard
                </span>
                <span className="font-mono text-[11px] px-1.5 py-0.5 rounded bg-[#232734] dark:bg-[#232734] light:bg-slate-200 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700 font-semibold">
                  v2.1.0 DSP
                </span>
              </div>
              <p className="text-[11px] text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600 leading-none mt-0.5">
                Low-Latency Dual-Sink Soundboard
              </p>
            </div>
          </div>
        </div>

        {/* Center Search Input ("Cari suara atau hotkey...") */}
        <div className="relative flex-1 max-w-md min-w-[220px]">
          <Search
            className="w-4 h-4 text-[#9CA3AF] absolute left-3.5 top-1/2 -translate-y-1/2 pointer-events-none"
            aria-hidden="true"
          />
          <input
            ref={searchInputRef}
            type="search"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Cari suara atau hotkey... (mis. Bass, Ctrl + 1, Num 4)"
            aria-label="Cari suara atau hotkey"
            className="w-full h-10 pl-10 pr-16 rounded-md bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 text-sm text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 placeholder-[#8A94A6] focus:outline-none focus:ring-2 focus:ring-[#00F0FF] focus:border-transparent transition"
          />
          {searchQuery ? (
            <button
              type="button"
              onClick={() => {
                onSearchChange("");
                searchInputRef.current?.focus();
              }}
              aria-label="Bersihkan pencarian"
              className="absolute right-2.5 top-1/2 -translate-y-1/2 px-2 py-1 text-xs font-mono rounded bg-[#232734] text-[#F3F4F6] hover:bg-[#2E3446] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
            >
              Reset
            </button>
          ) : (
            <kbd className="hidden sm:inline-block absolute right-3 top-1/2 -translate-y-1/2 px-1.5 py-0.5 text-[11px] font-mono rounded bg-[#232734] dark:bg-[#232734] light:bg-slate-200 text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 pointer-events-none">
              Ctrl+K
            </kbd>
          )}
        </div>

        {/* Right Utility & Custom Window Controls */}
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={onOpenAddSoundModal}
            className="h-10 px-3.5 rounded-md font-medium text-xs bg-[#00F0FF] text-[#090A0F] hover:bg-[#00F0FF]/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[#00F0FF] focus-visible:ring-offset-[#12131A] transition flex items-center gap-1.5 font-semibold"
          >
            <span>+ Import Suara</span>
          </button>

          <button
            type="button"
            onClick={onToggleTheme}
            aria-label={theme === "dark" ? "Ganti ke mode terang" : "Ganti ke mode gelap"}
            title={theme === "dark" ? "Mode Siang (Daylight Broadcast)" : "Mode Studio Gelap"}
            className="h-10 px-3 rounded-md bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 text-xs font-medium text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 hover:border-[#00F0FF] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] flex items-center gap-1.5 transition"
          >
            {theme === "dark" ? (
              <>
                <Sun className="w-4 h-4 text-[#F59E0B]" aria-hidden="true" />
                <span className="hidden md:inline">Daylight</span>
              </>
            ) : (
              <>
                <Moon className="w-4 h-4 text-[#6D28D9]" aria-hidden="true" />
                <span className="hidden md:inline">Dark Rack</span>
              </>
            )}
          </button>

          {/* Custom Window Controls (Minimize, Maximize, Close) */}
          <div
            className="flex items-center ml-1 pl-2 border-l border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 gap-1"
            role="group"
            aria-label="Kontrol Jendela Aplikasi"
          >
            <button
              type="button"
              onClick={onMinimizeWindow}
              aria-label="Minimize jendela"
              title="Minimize"
              className="w-9 h-9 rounded flex items-center justify-center text-[#9CA3AF] hover:text-[#F3F4F6] hover:bg-[#232734] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] transition"
            >
              <Minimize2 className="w-4 h-4" aria-hidden="true" />
            </button>
            <button
              type="button"
              onClick={onMaximizeWindow}
              aria-label="Maximize jendela"
              title="Maximize"
              className="w-9 h-9 rounded flex items-center justify-center text-[#9CA3AF] hover:text-[#F3F4F6] hover:bg-[#232734] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] transition"
            >
              <Maximize2 className="w-4 h-4" aria-hidden="true" />
            </button>
            <button
              type="button"
              onClick={onCloseWindow}
              aria-label="Tutup aplikasi"
              title="Close"
              className="w-9 h-9 rounded flex items-center justify-center text-[#9CA3AF] hover:text-white hover:bg-[#EF4444] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#EF4444] transition"
            >
              <X className="w-4 h-4" aria-hidden="true" />
            </button>
          </div>
        </div>
      </div>

      {/* Category Filter Bar */}
      <nav
        aria-label="Filter kategori suara"
        className="flex items-center justify-between gap-2 px-4 py-2 overflow-x-auto no-scrollbar"
      >
        <div className="flex items-center gap-2 flex-wrap">
          <button
            type="button"
            onClick={() => onSelectCategory("Semua")}
            aria-pressed={activeCategory === "Semua"}
            className={`min-h-[38px] px-3.5 py-1.5 rounded-md text-xs font-semibold transition flex items-center gap-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] ${
              activeCategory === "Semua"
                ? "bg-[#00F0FF] text-[#090A0F]"
                : "bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-slate-100 text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 hover:border-[#00F0FF]"
            }`}
          >
            <span>Semua</span>
            <span
              className={`font-mono text-[11px] px-1.5 py-0.2 rounded ${
                activeCategory === "Semua"
                  ? "bg-[#090A0F]/20 text-[#090A0F]"
                  : "bg-[#232734] dark:bg-[#232734] light:bg-slate-200 text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-700"
              }`}
            >
              {totalSounds}
            </span>
          </button>

          {categories.map((cat) => {
            const isSelected = activeCategory === cat;
            const count = soundCountByCategory[cat] ?? 0;
            return (
              <button
                key={cat}
                type="button"
                onClick={() => onSelectCategory(cat)}
                aria-pressed={isSelected}
                className={`min-h-[38px] px-3.5 py-1.5 rounded-md text-xs font-semibold transition flex items-center gap-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] ${
                  isSelected
                    ? "bg-[#00F0FF] text-[#090A0F]"
                    : "bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-slate-100 text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 hover:border-[#00F0FF]"
                }`}
              >
                {cat === "Favorit" && (
                  <Star
                    className={`w-3.5 h-3.5 ${
                      isSelected ? "fill-[#090A0F] text-[#090A0F]" : "text-[#F59E0B]"
                    }`}
                    aria-hidden="true"
                  />
                )}
                <span>{cat}</span>
                <span
                  className={`font-mono text-[11px] px-1.5 py-0.2 rounded ${
                    isSelected
                      ? "bg-[#090A0F]/20 text-[#090A0F]"
                      : "bg-[#232734] dark:bg-[#232734] light:bg-slate-200 text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-700"
                  }`}
                >
                  {count}
                </span>
              </button>
            );
          })}

          <button
            type="button"
            onClick={onOpenNewCategoryModal}
            className="min-h-[38px] px-3.5 py-1.5 rounded-md text-xs font-semibold bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-slate-100 text-[#EDE9FE] dark:text-[#EDE9FE] light:text-purple-900 border border-dashed border-[#8B5CF6] hover:bg-[#8B5CF6]/15 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#8B5CF6] transition flex items-center gap-1.5"
          >
            <FolderPlus className="w-3.5 h-3.5 text-[#A78BFA]" aria-hidden="true" />
            <span>+ Kategori Baru</span>
          </button>
        </div>
      </nav>
    </header>
  );
};
