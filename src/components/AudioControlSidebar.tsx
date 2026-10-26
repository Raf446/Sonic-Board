import React, { useRef } from "react";
import {
  AlertOctagon,
  CheckCircle2,
  Headphones,
  Mic,
  Radio,
  RefreshCw,
  RotateCcw,
  Sliders,
  Volume2,
} from "lucide-react";
import {
  AudioDevicesResponse,
  AudioLevelPayload,
  MixerState,
  RoutingStatus,
  VoiceFxState,
} from "../types/soundboard";

interface RotaryKnobProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  unit: string;
  formatValue?: (val: number) => string;
  onChange: (newVal: number) => void;
}

function polarToCartesian(cx: number, cy: number, r: number, angleDeg: number) {
  const rad = ((angleDeg - 90) * Math.PI) / 180;
  return {
    x: Number((cx + r * Math.cos(rad)).toFixed(3)),
    y: Number((cy + r * Math.sin(rad)).toFixed(3)),
  };
}

function describeKnobArc(
  cx: number,
  cy: number,
  r: number,
  startAngleDeg: number,
  endAngleDeg: number
): string {
  const fromAngle = Math.max(-135, Math.min(135, Math.min(startAngleDeg, endAngleDeg)));
  const toAngle = Math.max(-135, Math.min(135, Math.max(startAngleDeg, endAngleDeg)));
  if (toAngle - fromAngle < 0.5) {
    return "";
  }
  const startPt = polarToCartesian(cx, cy, r, fromAngle);
  const endPt = polarToCartesian(cx, cy, r, toAngle);
  const largeArcFlag = toAngle - fromAngle > 180 ? 1 : 0;
  return `M ${startPt.x} ${startPt.y} A ${r} ${r} 0 ${largeArcFlag} 1 ${endPt.x} ${endPt.y}`;
}

const TRACK_ARC_PATH = describeKnobArc(24, 24, 20, -135, 135);

const RotaryKnob: React.FC<RotaryKnobProps> = ({
  label,
  value,
  min,
  max,
  step,
  unit,
  formatValue,
  onChange,
}) => {
  const dragStartY = useRef<number | null>(null);
  const dragStartVal = useRef<number>(value);

  const isBipolar = min < 0 && max > 0;
  const clampedValue = Math.max(min, Math.min(max, value));
  const normalized = Math.max(0, Math.min(1, (clampedValue - min) / (max - min)));
  const angleDeg = isBipolar
    ? (clampedValue / Math.max(Math.abs(min), Math.abs(max))) * 135
    : -135 + normalized * 270;

  const activeArcStartDeg = isBipolar ? 0 : -135;
  const activeArcPath = describeKnobArc(24, 24, 20, activeArcStartDeg, angleDeg);

  const clampAndSnap = (raw: number) => {
    const clamped = Math.max(min, Math.min(max, raw));
    const stepped = Math.round(clamped / step) * step;
    return Number(stepped.toFixed(2));
  };

  const handlePointerDown = (e: React.PointerEvent<HTMLButtonElement>) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    dragStartY.current = e.clientY;
    dragStartVal.current = value;
  };

  const handlePointerMove = (e: React.PointerEvent<HTMLButtonElement>) => {
    if (dragStartY.current === null) return;
    const deltaY = dragStartY.current - e.clientY;
    const range = max - min;
    const nextVal = dragStartVal.current + (deltaY / 110) * range;
    onChange(clampAndSnap(nextVal));
  };

  const handlePointerUp = (e: React.PointerEvent<HTMLButtonElement>) => {
    if (dragStartY.current !== null) {
      e.currentTarget.releasePointerCapture(e.pointerId);
      dragStartY.current = null;
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    if (e.key === "ArrowUp" || e.key === "ArrowRight") {
      e.preventDefault();
      onChange(clampAndSnap(value + step));
    } else if (e.key === "ArrowDown" || e.key === "ArrowLeft") {
      e.preventDefault();
      onChange(clampAndSnap(value - step));
    } else if (e.key === "Home") {
      e.preventDefault();
      onChange(min);
    } else if (e.key === "End") {
      e.preventDefault();
      onChange(max);
    }
  };

  const displayStr = formatValue ? formatValue(value) : `${Math.round(value)}${unit}`;

  return (
    <div className="flex flex-col items-center bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 rounded-md p-2.5">
      <button
        type="button"
        role="slider"
        aria-label={`Kontrol rotari DSP ${label}`}
        aria-valuemin={min}
        aria-valuemax={max}
        aria-valuenow={value}
        aria-valuetext={displayStr}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onKeyDown={handleKeyDown}
        onDoubleClick={() => onChange(min < 0 ? 0 : min)}
        title="Tahan & geser vertikal, atau gunakan tombol panah Atas/Bawah"
        className="relative w-14 h-14 rounded-full bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border-2 border-[#3B4259] hover:border-[#8B5CF6] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF] flex items-center justify-center cursor-ns-resize select-none transition"
      >
        <svg className="w-full h-full pointer-events-none" viewBox="0 0 48 48">
          <path
            d={TRACK_ARC_PATH}
            fill="none"
            stroke="#232734"
            strokeWidth="3.5"
            strokeLinecap="round"
          />
          {isBipolar && (
            <line
              x1="24"
              y1="2.2"
              x2="24"
              y2="5.8"
              stroke="#3B4259"
              strokeWidth="1.5"
              strokeLinecap="round"
            />
          )}
          {activeArcPath && (
            <path
              d={activeArcPath}
              fill="none"
              stroke="#8B5CF6"
              strokeWidth="3.5"
              strokeLinecap="round"
            />
          )}
        </svg>
        <span
          style={{ transform: `rotate(${angleDeg}deg)` }}
          className="absolute inset-1.5 rounded-full flex justify-center items-start pt-1 pointer-events-none"
        >
          <span className="w-1 h-2.5 rounded-full bg-[#00F0FF]" />
        </span>
      </button>
      <span className="mt-1.5 text-xs font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900">
        {label}
      </span>
      <span className="font-mono text-[11px] text-[#A78BFA] dark:text-[#A78BFA] light:text-purple-800">
        {displayStr}
      </span>
    </div>
  );
};

interface VerticalMixerChannelProps {
  label: string;
  shortLabel: string;
  volume: number;
  level: number;
  db: number;
  panicMuted: boolean;
  onChange: (val: number) => void;
}

const VerticalMixerChannel: React.FC<VerticalMixerChannelProps> = ({
  label,
  shortLabel,
  volume,
  level,
  db,
  panicMuted,
  onChange,
}) => {
  const numSegments = 14;
  const activeSegments = panicMuted ? 0 : Math.round(level * numSegments);
  const pct = Math.round(volume * 100);

  return (
    <div className="flex flex-col items-center bg-[#12131A] dark:bg-[#12131A] light:bg-slate-100 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 rounded-md p-2.5">
      <span className="text-[11px] font-semibold text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 text-center truncate w-full">
        {shortLabel}
      </span>
      <span className="font-mono text-[10px] text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600 mb-2">
        {panicMuted || db <= -59 ? "-INF dB" : `${db.toFixed(1)} dB`}
      </span>

      <div className="flex items-center justify-center gap-2.5 h-36 w-full">
        {/* Vertical Fader Input */}
        <div className="relative h-full flex items-center justify-center w-7">
          <input
            type="range"
            min={0}
            max={100}
            value={pct}
            aria-label={`Volume ${label}`}
            onChange={(e) => onChange(Number(e.target.value) / 100)}
            style={{
              writingMode: "vertical-lr",
              direction: "rtl",
            }}
            className="h-32 w-5 cursor-pointer accent-[#00F0FF] bg-[#232734] rounded appearance-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
          />
        </div>

        {/* Multi-Color LED Peak Meter (Green -> Yellow -> Red) */}
        <div
          className="flex flex-col-reverse justify-between h-32 w-3.5 p-0.5 rounded bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-slate-200 border border-[#232734]"
          aria-label={`LED Peak Meter ${label}`}
        >
          {Array.from({ length: numSegments }, (_, idx) => {
            const isLit = idx < activeSegments;
            const isRed = idx >= 12;
            const isYellow = idx >= 9 && idx < 12;
            const activeColor = isRed
              ? "bg-[#EF4444]"
              : isYellow
              ? "bg-[#F59E0B]"
              : "bg-[#10B981]";
            return (
              <span
                key={idx}
                className={`w-full h-1.5 rounded-[1px] transition-colors duration-75 ${
                  isLit ? activeColor : "bg-[#232734] dark:bg-[#232734] light:bg-slate-300"
                }`}
              />
            );
          })}
        </div>
      </div>

      <span className="mt-2 font-mono text-xs font-semibold text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700">
        {pct}%
      </span>
    </div>
  );
};

interface AudioControlSidebarProps {
  devices: AudioDevicesResponse;
  routing: RoutingStatus;
  mixer: MixerState;
  levels: AudioLevelPayload;
  onChangeRouting: (micId: string, virtualId: string, monitorId: string) => void;
  onRefreshDevices: () => void;
  onChangeMixer: (micVol: number, sbVol: number, monVol: number) => void;
  onChangeVoiceFx: (fx: VoiceFxState) => void;
  onTriggerPanicMute: () => void;
  onResumeAudio: () => void;
}

export const AudioControlSidebar: React.FC<AudioControlSidebarProps> = ({
  devices,
  routing,
  mixer,
  levels,
  onChangeRouting,
  onRefreshDevices,
  onChangeMixer,
  onChangeVoiceFx,
  onTriggerPanicMute,
  onResumeAudio,
}) => {
  const isPanic = routing.panicMuted || mixer.panicMuted || levels.panicMuted;

  return (
    <aside
      aria-label="Audio Control Center"
      className="w-full lg:w-[360px] xl:w-[380px] shrink-0 bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border-t lg:border-t-0 lg:border-l border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 p-4 flex flex-col justify-between gap-4 overflow-y-auto"
    >
      <div className="space-y-4">
        {/* Section 1: Input & Output Routing */}
        <section
          aria-labelledby="routing-heading"
          className="rounded-lg bg-[#151720] dark:bg-[#151720] light:bg-slate-50 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 p-3.5"
        >
          <div className="flex items-center justify-between gap-2 mb-3">
            <div className="flex items-center gap-2">
              <Radio className="w-4 h-4 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700" aria-hidden="true" />
              <h2
                id="routing-heading"
                className="text-xs font-bold uppercase tracking-wider text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900"
              >
                Input & Output Routing
              </h2>
            </div>

            {/* Real-Time Connection Status Indicator */}
            <div className="flex items-center gap-1.5">
              <span
                role="status"
                className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono font-bold border ${
                  isPanic
                    ? "bg-[#EF4444]/20 text-[#FCA5A5] border-[#EF4444]"
                    : routing.connected
                    ? "bg-[#10B981]/15 text-[#34D399] dark:text-[#34D399] light:text-emerald-800 border-[#10B981]/60"
                    : "bg-[#F59E0B]/20 text-[#FCD34D] border-[#F59E0B]"
                }`}
              >
                {isPanic ? (
                  <AlertOctagon className="w-3 h-3 text-[#EF4444]" aria-hidden="true" />
                ) : (
                  <CheckCircle2 className="w-3 h-3 text-[#10B981]" aria-hidden="true" />
                )}
                <span>{isPanic ? "MUTED" : "CONNECTED"}</span>
              </span>

              <button
                type="button"
                onClick={onRefreshDevices}
                aria-label="Segarkan daftar perangkat audio"
                title="Refresh perangkat WASAPI / CPAL"
                className="w-7 h-7 rounded flex items-center justify-center bg-[#232734] dark:bg-[#232734] light:bg-slate-200 text-[#9CA3AF] hover:text-[#F3F4F6] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
              >
                <RefreshCw className="w-3.5 h-3.5" aria-hidden="true" />
              </button>
            </div>
          </div>

          <div className="space-y-2.5">
            {/* Physical Microphone Selector */}
            <div>
              <label
                htmlFor="select-physical-mic"
                className="flex items-center gap-1.5 text-xs font-medium text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-700 mb-1"
              >
                <Mic className="w-3.5 h-3.5 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700" aria-hidden="true" />
                <span>Physical Mic (Input Stream)</span>
              </label>
              <select
                id="select-physical-mic"
                value={routing.micDeviceId}
                onChange={(e) =>
                  onChangeRouting(
                    e.target.value,
                    routing.virtualMicId,
                    routing.monitorHeadphoneId
                  )
                }
                className="w-full h-9 px-2.5 rounded bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 text-xs text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 focus:outline-none focus:ring-2 focus:ring-[#00F0FF]"
              >
                {devices.inputs.map((dev) => (
                  <option key={dev.id} value={dev.id}>
                    {dev.name} ({dev.sampleRate / 1000}kHz)
                  </option>
                ))}
              </select>
            </div>

            {/* Virtual Audio Cable Selector */}
            <div>
              <label
                htmlFor="select-virtual-mic"
                className="flex items-center gap-1.5 text-xs font-medium text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-700 mb-1"
              >
                <Volume2 className="w-3.5 h-3.5 text-[#A78BFA] dark:text-[#A78BFA] light:text-purple-700" aria-hidden="true" />
                <span>Virtual Mic (Discord / Game / OBS Sink)</span>
              </label>
              <select
                id="select-virtual-mic"
                value={routing.virtualMicId}
                onChange={(e) =>
                  onChangeRouting(
                    routing.micDeviceId,
                    e.target.value,
                    routing.monitorHeadphoneId
                  )
                }
                className="w-full h-9 px-2.5 rounded bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 text-xs text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 focus:outline-none focus:ring-2 focus:ring-[#00F0FF]"
              >
                {devices.outputs.map((dev) => (
                  <option key={dev.id} value={dev.id}>
                    {dev.isVirtual ? "[VIRTUAL] " : ""}
                    {dev.name}
                  </option>
                ))}
              </select>
            </div>

            {/* Physical Headphone Monitor Selector */}
            <div>
              <label
                htmlFor="select-monitor-headphones"
                className="flex items-center gap-1.5 text-xs font-medium text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-700 mb-1"
              >
                <Headphones className="w-3.5 h-3.5 text-[#10B981]" aria-hidden="true" />
                <span>Monitor (Headphone Output)</span>
              </label>
              <select
                id="select-monitor-headphones"
                value={routing.monitorHeadphoneId}
                onChange={(e) =>
                  onChangeRouting(
                    routing.micDeviceId,
                    routing.virtualMicId,
                    e.target.value
                  )
                }
                className="w-full h-9 px-2.5 rounded bg-[#1A1D26] dark:bg-[#1A1D26] light:bg-white border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 text-xs text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900 focus:outline-none focus:ring-2 focus:ring-[#00F0FF]"
              >
                {devices.outputs.map((dev) => (
                  <option key={dev.id} value={dev.id}>
                    {dev.name}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div className="mt-2.5 pt-2 border-t border-[#232734] dark:border-[#232734] light:border-slate-200 flex items-center justify-between text-[11px] font-mono text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600">
            <span className="truncate">{routing.statusMessage}</span>
            <span className="shrink-0 ml-2 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700">
              {routing.estimatedLatencyMs.toFixed(1)}ms
            </span>
          </div>
        </section>

        {/* Section 2: Virtual Mixer (3 Vertical Sliders + LED Peak Meters) */}
        <section
          aria-labelledby="mixer-heading"
          className="rounded-lg bg-[#151720] dark:bg-[#151720] light:bg-slate-50 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 p-3.5"
        >
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <Sliders className="w-4 h-4 text-[#00F0FF] dark:text-[#00F0FF] light:text-sky-700" aria-hidden="true" />
              <h2
                id="mixer-heading"
                className="text-xs font-bold uppercase tracking-wider text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900"
              >
                Virtual Mixer & Peak Meters
              </h2>
            </div>
            <span className="font-mono text-[11px] text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-600">
              20Hz RMS
            </span>
          </div>

          <div className="grid grid-cols-3 gap-2.5">
            <VerticalMixerChannel
              label="Mic Vol"
              shortLabel="Mic Vol"
              volume={mixer.micVol}
              level={levels.micLevel}
              db={levels.micDb}
              panicMuted={isPanic}
              onChange={(val) =>
                onChangeMixer(val, mixer.soundboardVol, mixer.monitorVol)
              }
            />
            <VerticalMixerChannel
              label="Soundboard"
              shortLabel="Soundboard"
              volume={mixer.soundboardVol}
              level={levels.soundboardLevel}
              db={levels.soundboardDb}
              panicMuted={isPanic}
              onChange={(val) =>
                onChangeMixer(mixer.micVol, val, mixer.monitorVol)
              }
            />
            <VerticalMixerChannel
              label="Monitor (Headphone)"
              shortLabel="Monitor"
              volume={mixer.monitorVol}
              level={levels.monitorLevel}
              db={levels.monitorDb}
              panicMuted={isPanic}
              onChange={(val) =>
                onChangeMixer(mixer.micVol, mixer.soundboardVol, val)
              }
            />
          </div>
        </section>

        {/* Section 3: Voice FX (DSP Rotary Knobs) */}
        <section
          aria-labelledby="dsp-heading"
          className="rounded-lg bg-[#151720] dark:bg-[#151720] light:bg-slate-50 border border-[#2E3446] dark:border-[#2E3446] light:border-slate-300 p-3.5"
        >
          <div className="flex items-center justify-between mb-3">
            <h2
              id="dsp-heading"
              className="text-xs font-bold uppercase tracking-wider text-[#F3F4F6] dark:text-[#F3F4F6] light:text-slate-900"
            >
              Voice FX (Real-Time DSP)
            </h2>
            <button
              type="button"
              onClick={() =>
                onChangeVoiceFx({ pitch: 0, reverb: 0, distortion: 0 })
              }
              aria-label="Reset efek suara DSP ke nol"
              className="px-2 py-0.5 rounded text-[11px] font-mono bg-[#232734] dark:bg-[#232734] light:bg-slate-200 text-[#9CA3AF] dark:text-[#9CA3AF] light:text-slate-700 hover:text-[#F3F4F6] flex items-center gap-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#00F0FF]"
            >
              <RotateCcw className="w-3 h-3" aria-hidden="true" />
              <span>Reset</span>
            </button>
          </div>

          <div className="grid grid-cols-3 gap-2.5">
            <RotaryKnob
              label="Pitch"
              value={mixer.voiceFx.pitch}
              min={-12}
              max={12}
              step={1}
              unit=" st"
              formatValue={(v) => `${v > 0 ? `+${v}` : v} st`}
              onChange={(pitch) =>
                onChangeVoiceFx({ ...mixer.voiceFx, pitch })
              }
            />
            <RotaryKnob
              label="Reverb"
              value={Math.round(mixer.voiceFx.reverb * 100)}
              min={0}
              max={100}
              step={5}
              unit="%"
              onChange={(pct) =>
                onChangeVoiceFx({ ...mixer.voiceFx, reverb: pct / 100 })
              }
            />
            <RotaryKnob
              label="Distortion"
              value={Math.round(mixer.voiceFx.distortion * 100)}
              min={0}
              max={100}
              step={5}
              unit="%"
              onChange={(pct) =>
                onChangeVoiceFx({ ...mixer.voiceFx, distortion: pct / 100 })
              }
            />
          </div>
        </section>
      </div>

      {/* Section 4: Emergency Action (Striped Hazard PANIC MUTE Button) */}
      <div className="pt-2 space-y-2">
        <button
          type="button"
          onClick={onTriggerPanicMute}
          aria-label="PANIC MUTE: Hentikan semua audio dan potong mic pass-through segera"
          className="w-full min-h-[56px] rounded-lg border-2 border-[#EF4444] p-1 relative overflow-hidden group focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-[#EF4444] focus-visible:ring-offset-[#12131A] transition active:scale-[0.99]"
          style={{
            backgroundImage:
              "repeating-linear-gradient(45deg, #7F1D1D, #7F1D1D 12px, #450A0A 12px, #450A0A 24px)",
          }}
        >
          <div className="w-full h-full py-2.5 px-4 rounded bg-[#12131A]/85 group-hover:bg-[#EF4444] group-hover:text-[#090A0F] transition flex items-center justify-center gap-2.5 text-[#FCA5A5] font-bold tracking-wider text-sm">
            <AlertOctagon className="w-5 h-5 shrink-0" aria-hidden="true" />
            <span>PANIC MUTE (KILL ALL AUDIO)</span>
          </div>
        </button>

        {isPanic && (
          <button
            type="button"
            onClick={onResumeAudio}
            className="w-full h-10 rounded-md bg-[#10B981] text-[#090A0F] font-semibold text-xs hover:bg-[#10B981]/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#10B981] transition flex items-center justify-center gap-2"
          >
            <CheckCircle2 className="w-4 h-4" aria-hidden="true" />
            <span>AKTIFKAN KEMBALI ROUTING AUDIO</span>
          </button>
        )}
      </div>
    </aside>
  );
};
