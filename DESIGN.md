# SonicBoard: Design & Architecture Specification (`DESIGN.md`)

**Design Read:**
Reading this as: low-latency desktop broadcast soundboard & audio routing console for streamers and gamers, in a tactile hardware-rack studio style, dial `ENERGY 2 / RHYTHM 2 / MOTION 2`.

## 1. Brand Identity & Aesthetic Direction
- **Product:** SonicBoard: Universal Low-Latency Desktop Soundboard & Virtual Audio Router.
- **Target Audience:** Streamers, competitive gamers, podcasters, and content creators running dual-sink audio pipelines (Physical Mic + Soundboard -> Virtual Audio Cable + Headphone Monitor).
- **Personality:** Tactile broadcast hardware console meets dark-room streaming studio. Every control resembles a physical studio unit: calibrated dB peak meters, rotary DSP potentiometers, hardware-style keybind triggers, and a guarded hazard Panic Mute kill-switch.

## 2. Color System & Contrast Verification (WCAG AA Checked)
Limited to 3 core surface/brand colors + 1 primary focus accent + semantic signal indicators (R-29):

### Dark Mode ("Studio Rack", default for dark-room broadcast/gaming setups, R-21)
- **Canvas Background (`--bg-canvas`):** `#12131A` (Deep rack chassis)
- **Elevated Card Surface (`--bg-card`):** `#1A1D26` (Slate hardware module)
- **Control Surface (`--bg-control`):** `#232734` (Recessed fader/pad well)
- **Primary Text (`--text-primary`):** `#F3F4F6` on `#12131A` (16.4:1 contrast, PASS AA/AAA) and on `#1A1D26` (14.5:1 contrast, PASS AA/AAA)
- **Secondary Text (`--text-secondary`):** `#9CA3AF` on `#12131A` (6.8:1 contrast, PASS AA) and on `#1A1D26` (6.0:1 contrast, PASS AA)
- **Muted / Technical Label (`--text-muted`):** `#8A94A6` on `#1A1D26` (4.85:1 contrast, PASS AA)
- **Primary Focus Accent (`--accent-cyan`):** `#00F0FF` (Neon Cyan, used on active playing sound pad border/waveform and focus ring; dark text `#090A0F` over `#00F0FF` has 14.8:1 contrast, PASS AA)
- **DSP / Routing Accent (`--accent-purple`):** `#A78BFA` / `#8B5CF6` (Electric Purple, used for DSP Voice FX rotary ring & hotkey badges; `#EDE9FE` text on `#4C1D95` badge fill has 9.8:1 contrast, PASS AA)
- **Emergency / Clip Signal (`--accent-red`):** `#EF4444` (Hazard Red, used exclusively for the Panic Mute kill-switch and > -3 dB peak meter clipping)

### Light Mode ("Daylight Broadcast", full working theme toggle, R-21 / R-34)
- **Canvas Background:** `#F1F5F9`
- **Elevated Card Surface:** `#FFFFFF`
- **Control Surface:** `#E2E8F0`
- **Primary Text:** `#0F172A` on `#FFFFFF` (17.8:1 contrast, PASS AA/AAA)
- **Secondary Text:** `#334155` on `#FFFFFF` (10.3:1 contrast, PASS AA)
- **Muted / Technical Label:** `#475569` on `#FFFFFF` (7.5:1 contrast, PASS AA)
- **Primary Focus Accent:** `#0369A1` (High-contrast Broadcast Cyan `sky-700` on light surfaces, 5.93:1 contrast on `#FFFFFF`, PASS AA)
- **DSP / Routing Accent:** `#6D28D9` (Deep Studio Purple, 7.2:1 contrast on `#FFFFFF`)
- **Emergency / Clip Signal:** `#DC2626` (Hazard Red, 5.6:1 contrast on `#FFFFFF`)

## 3. Dials & Liveliness Levers
- **ENERGY: 2 (Balanced):** High-contrast broadcast instrumentation without visual clutter or background orbs.
- **RHYTHM: 2 (Consistent with structural variation):** Distinct 3-zone desktop layout:
  1. Compact top studio transport/filter bar with live output dB readout and category pills.
  2. Center responsive soundboard matrix where active pads, pinned favorites, and the dashed import drop-zone have distinct visual weight.
  3. Right-hand fixed hardware rack (Audio Control Center) housing I/O routing selectors, 3-channel vertical LED peak meters, 3 rotary DSP knobs, and the striped hazard Panic Mute bar.
- **MOTION: 2 (Audio-reactive & direct manipulation transitions):** Motion is strictly tied to user action or real-time audio signal:
  - Top-bar soundwave bars and mixer LED segments respond directly to real-time RMS/dB peak levels (no fake un-triggered loops).
  - Sound pad waveform canvas animates playback progress and amplitude while a sample is actively playing.
  - Snappy spring transitions (`150ms`) on pad trigger press and modal open/close.
- **Identity Motif:** Calibrated studio rack markings (dB scale ticks `-48`, `-24`, `-12`, `-6`, `0 dB`, hardware keycap badges, and diagonal yellow/red hazard striping on the Panic Mute bar).

## 4. Written Reasons for Visual Decisions (R-31 & Group 2 Purpose-Gate)
- **Why dark slate palette (`#12131A` / `#1A1D26`):** Prevents monitor glare on a streamer's face and webcam while running on a secondary display in a dark gaming room, paired with a full Daylight mode toggle for bright environments (R-01, R-21, R-34).
- **Why Neon Cyan (`#00F0FF`) & Electric Purple (`#8B5CF6`):** Separates two distinct functional layers at a glance during live broadcasts: Cyan indicates active audio playback/monitoring, while Purple marks control bindings (global hotkeys and DSP voice processing) (R-01, R-29).
- **Why Hazard Red (`#EF4444`) with diagonal stripes:** Reserves emergency visual weight exclusively for the Panic Mute button and 0 dB clipping alerts so a streamer can hit the kill-switch in peripheral vision (R-01, R-14).
- **Why glow is capped to active playing pad only:** Glow appears only on a currently playing sound pad's border to flag live audio transmission into the virtual mic, keeping idle cards matte (R-13).
- **Why shadow is used only on modals and floating context menus:** Cards sit flat in the rack grid separated by 1px structural borders (`3:1` boundary contrast in high-contrast focus), reserving elevation shadows strictly for overlays (R-12).
- **Why typography uses system geometric sans (`Inter` / `Segoe UI`) + tabular monospace (`JetBrains Mono` / `Consolas`):** Sans-serif keeps Indonesian/English sound titles scannable, while tabular monospace prevents layout jitter on rapidly updating dB meters, millisecond durations, and keybind codes (`Ctrl + 1`, `Num 4`) (R-06).
- **Why custom category SVG icons instead of decorative emojis:** Each sound pad displays a clean, domain-relevant vector audio icon (or waveform category mark) instead of scattered text emojis, keeping labels crisp and accessible (R-04).
- **Why card variation in Center Grid:** Playing pads expand their active waveform readout, favorite pads display a dedicated quick-pin indicator, and the final grid cell is a recessed dashed drop-zone for importing new MP3/WAV/OGG/FLAC files (R-14).
