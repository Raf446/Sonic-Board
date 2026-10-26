import React from "react";
import {
  Edit3,
  Keyboard,
  Play,
  Square,
  Star,
  Trash2,
  Volume2,
  VolumeX,
} from "lucide-react";
import { SoundItem } from "../types/soundboard";
import { SoundIcon } from "./SoundIcon";

interface SoundPadCardProps {
  sound: SoundItem;
  isPlaying: boolean;
  outputLevel: number;
  onPlayToggle: (sound: SoundItem) => void;
  onVolumeChange: (soundId: string, volume: number) => void;
  onToggleFavorite: (sound: SoundItem) => void;
  onEditSound: (sound: SoundItem) => void;
  onDeleteSound: (sound: SoundItem) => void;
}

export const SoundPadCard: React.FC<SoundPadCardProps> = ({
  sound,
  isPlaying,
  outputLevel,
  onPlayToggle,
  onVolumeChange,
  onToggleFavorite,
  onEditSound,
  onDeleteSound,
}) => {
  const volumePercent = Math.round(sound.volume * 100);

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    onEditSound(sound);
  };

  return (
    <article
      onContextMenu={handleContextMenu}
      aria-label={`Sound pad ${sound.name}`}
      className={`group relative rounded-lg p-3.5 transition-all duration-150 flex flex-col justify-between border ${
        isPlaying
          ? "bg-[#1E2330] dark:bg-[#1E2330] light:bg-sky-50 border-[#00F0FF] shadow-[0_0_16px_rgba(0,240,255,0.22)]"
          : sound.favorite
          ? "bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border-[#3B4259] dark:border-[#3B4259] light:border-slate-300 hover:border-[#00F0FF]/70"
          : "bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 hover:border-[#00F0FF]/60"
      }`}
    >
      {/* Top Row: Icon, Title, Category, and Hotkey Badge */}
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2.5 min-w-0">
          <button
            type="button"
            onClick={() => onPlayToggle(sound)}
            aria-label={isPlaying ? `Hentikan ${sound.name}` : `Putar ${sound.name}`}
            className={`w-11 h-11 rounded-md shrink-0 flex items-center justify-center transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] ${
              isPlaying
                ? "bg-[#00F0FF] text-[#090A0F]"
                : "bg-[#232734] dark:bg-[#232734] light:bg-slate-100 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700 hover:bg-[#00F0FF] hover:text-[#090A0F]"
            }`}
          >
            {isPlaying ? (
              <Square className="w-4 h-4 fill-current" aria-hidden="true" />
            ) : (
              <SoundIcon name={sound.icon} className="w-5 h-5" />
            )}
          </button>

          <div className="min-w-0">
            <h2 className="font-semibold text-sm text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 truncate">
              {sound.name}
            </h2>
            <div className="flex items-center gap-1.5 mt-0.5">
              <span className="text-[11px] font-medium text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600">
                {sound.category}
              </span>
              <span className="text-[#8A94A6]" aria-hidden="true">
                •
              </span>
              <span className="font-mono text-[11px] text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600">
                {sound.durationSecs.toFixed(1)}s
              </span>
            </div>
          </div>
        </div>

        {/* Registered Hotkey Badge & Quick Actions */}
        <div className="flex items-center gap-1 shrink-0">
          <button
            type="button"
            onClick={() => onEditSound(sound)}
            title="Klik untuk ubah hotkey atau detail suara"
            aria-label={`Ubah hotkey ${sound.name}, saat ini ${sound.hotkey || "belum diatur"}`}
            className={`px-2 py-1 rounded text-[11px] font-mono font-semibold border transition flex items-center gap-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] ${
              sound.hotkey
                ? "bg-[#4C1D95]/60 dark:bg-[#4C1D95]/60 light:bg-purple-100 text-[#EDE9FE] dark:text-[#EDE9FE] light:text-purple-900 border-[#8B5CF6]/70"
                : "bg-[#232734] dark:bg-[#232734] light:bg-slate-100 text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600 border-[#2E3446] dark:border-[#2E3446] light:border-slate-300"
            }`}
          >
            <Keyboard className="w-3 h-3 text-[#A78BFA] dark:text-[#A78BFA] light:text-purple-700" aria-hidden="true" />
            <span>{sound.hotkey || "No Key"}</span>
          </button>

          <button
            type="button"
            onClick={() => onToggleFavorite(sound)}
            aria-label={sound.favorite ? `Hapus ${sound.name} dari Favorit` : `Pin ${sound.name} ke Favorit`}
            title={sound.favorite ? "Hapus dari Favorit" : "Pin ke Favorit"}
            className="w-8 h-8 rounded flex items-center justify-center text-[#9CA3AF] hover:text-[#F59E0B] hover:bg-[#232734] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] transition"
          >
            <Star
              className={`w-4 h-4 ${
                sound.favorite ? "fill-[#F59E0B] text-[#F59E0B]" : "text-[#9CA3AF]"
              }`}
              aria-hidden="true"
            />
          </button>
        </div>
      </div>

      {/* Middle Interactive Waveform Trigger Button */}
      <button
        type="button"
        onClick={() => onPlayToggle(sound)}
        aria-label={`${isPlaying ? "Hentikan" : "Trigger"} sampel ${sound.name}`}
        className="my-3 w-full h-14 px-2.5 py-2 rounded-md bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 border border-[#232734] dark:border-[#232734] light:border-slate-200 flex items-center justify-between gap-1 hover:border-[#00F0FF]/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] transition cursor-pointer"
      >
        <div className="flex items-center gap-[3px] flex-1 h-full">
          {sound.waveform.map((barVal, idx) => {
            const liveBoost = isPlaying
              ? Math.min(
                  1,
                  barVal * (0.65 + 0.55 * Math.abs(Math.sin(idx * 0.7 + outputLevel * 9)))
                )
              : barVal * 0.78;
            const heightPct = Math.max(14, Math.round(liveBoost * 100));
            return (
              <span
                key={idx}
                style={{ height: `${heightPct}%` }}
                className={`flex-1 rounded-full transition-all duration-75 ${
                  isPlaying
                    ? idx % 3 === 0
                      ? "bg-[#00F0FF]"
                      : "bg-[#8B5CF6]"
                    : "bg-[#8A94A6]/55 group-hover:bg-[#00F0FF]/70"
                }`}
              />
            );
          })}
        </div>
        <span
          className={`ml-2 px-2 py-1 rounded text-[11px] font-mono font-semibold shrink-0 flex items-center gap-1 ${
            isPlaying
              ? "bg-[#00F0FF] text-[#090A0F]"
              : "bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300"
          }`}
        >
          <Play className="w-3 h-3 fill-current" aria-hidden="true" />
          <span>{isPlaying ? "LIVE" : "PLAY"}</span>
        </span>
      </button>

      {/* Bottom Row: Individual Volume Mini-Slider & Edit/Delete Controls */}
      <div className="flex items-center justify-between gap-2.5 pt-1 border-t border-[#232734] dark:border-[#232734] light:border-slate-200">
        <div className="flex items-center gap-2 flex-1 min-w-0">
          <button
            type="button"
            onClick={() => onVolumeChange(sound.id, sound.volume > 0 ? 0 : 0.9)}
            aria-label={sound.volume === 0 ? `Unmute ${sound.name}` : `Mute ${sound.name}`}
            className="w-7 h-7 rounded flex items-center justify-center text-[#9CA3AF] hover:text-[#F3F4F6] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
          >
            {sound.volume === 0 ? (
              <VolumeX className="w-3.5 h-3.5 text-[#EF4444]" aria-hidden="true" />
            ) : (
              <Volume2 className="w-3.5 h-3.5 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700" aria-hidden="true" />
            )}
          </button>

          <input
            type="range"
            min={0}
            max={100}
            value={volumePercent}
            aria-label={`Volume ${sound.name}`}
            onChange={(e) => onVolumeChange(sound.id, Number(e.target.value) / 100)}
            className="w-full h-1.5 bg-[#232734] dark:bg-[#232734] light:bg-slate-300 rounded-lg appearance-none cursor-pointer accent-[#00F0FF]"
          />

          <span className="font-mono text-[11px] text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-800 w-9 text-right shrink-0">
            {volumePercent}%
          </span>
        </div>

        <div className="flex items-center gap-1 shrink-0">
          <button
            type="button"
            onClick={() => onEditSound(sound)}
            aria-label={`Edit ${sound.name}`}
            title="Edit suara & hotkey (atau klik kanan kartu)"
            className="w-8 h-8 rounded flex items-center justify-center text-[#9CA3AF] hover:text-[#F3F4F6] hover:bg-[#232734] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] transition"
          >
            <Edit3 className="w-3.5 h-3.5" aria-hidden="true" />
          </button>
          <button
            type="button"
            onClick={() => onDeleteSound(sound)}
            aria-label={`Hapus ${sound.name}`}
            title="Hapus suara"
            className="w-8 h-8 rounded flex items-center justify-center text-[#9CA3AF] hover:text-[#EF4444] hover:bg-[#EF4444]/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#EF4444] transition"
          >
            <Trash2 className="w-3.5 h-3.5" aria-hidden="true" />
          </button>
        </div>
      </div>
    </article>
  );
};
