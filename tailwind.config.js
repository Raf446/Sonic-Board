/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        rack: {
          bg: "#12131A",
          card: "#1A1D26",
          control: "#232734",
          border: "#2E3446",
        },
        neon: {
          cyan: "#00F0FF",
          purple: "#8B5CF6",
          red: "#EF4444",
          green: "#10B981",
          amber: "#F59E0B",
        },
      },
      fontFamily: {
        sans: ["Inter", "Segoe UI", "system-ui", "-apple-system", "sans-serif"],
        mono: ["JetBrains Mono", "Consolas", "ui-monospace", "monospace"],
      },
    },
  },
  plugins: [],
};
