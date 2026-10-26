import React from "react";
import {
  Bell,
  Crosshair,
  Disc,
  Megaphone,
  Mic,
  Music,
  Radio,
  Sliders,
  Speaker,
  Trophy,
  Volume2,
  Wand2,
  Zap,
} from "lucide-react";

export const AVAILABLE_ICONS = [
  { id: "speaker", label: "Sub Speaker" },
  { id: "crosshair", label: "Tactical Crosshair" },
  { id: "wand", label: "Synth Chime" },
  { id: "megaphone", label: "Airhorn" },
  { id: "trophy", label: "Victory Trophy" },
  { id: "zap", label: "Laser Charge" },
  { id: "radio", label: "Broadcast Tone" },
  { id: "sliders", label: "FX Sweep" },
  { id: "music", label: "Musical Stinger" },
  { id: "bell", label: "Alert Bell" },
  { id: "mic", label: "Vocal Sample" },
  { id: "disc", label: "Scratch Disc" },
] as const;

interface SoundIconProps {
  name: string;
  className?: string;
}

export const SoundIcon: React.FC<SoundIconProps> = ({ name, className = "w-5 h-5" }) => {
  switch (name) {
    case "speaker":
      return <Speaker className={className} aria-hidden="true" />;
    case "crosshair":
      return <Crosshair className={className} aria-hidden="true" />;
    case "wand":
    case "sparkles":
      return <Wand2 className={className} aria-hidden="true" />;
    case "megaphone":
      return <Megaphone className={className} aria-hidden="true" />;
    case "trophy":
      return <Trophy className={className} aria-hidden="true" />;
    case "zap":
      return <Zap className={className} aria-hidden="true" />;
    case "radio":
      return <Radio className={className} aria-hidden="true" />;
    case "sliders":
      return <Sliders className={className} aria-hidden="true" />;
    case "music":
      return <Music className={className} aria-hidden="true" />;
    case "bell":
      return <Bell className={className} aria-hidden="true" />;
    case "mic":
      return <Mic className={className} aria-hidden="true" />;
    case "disc":
      return <Disc className={className} aria-hidden="true" />;
    default:
      return <Volume2 className={className} aria-hidden="true" />;
  }
};
