import type { Config } from "tailwindcss";

export default {
  darkMode: "class",
  content: [
    "./src/pages/**/*.{ts,tsx}",
    "./src/components/**/*.{ts,tsx}",
    "./src/app/**/*.{ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Dark marketplace palette
        surface: {
          DEFAULT: "#0f1117",
          card: "#161b27",
          elevated: "#1e2535",
        },
        accent: {
          purple: "#3B82F6",
          green: "#14f195",
          red: "#f44336",
          yellow: "#f59e0b",
        },
      },
      keyframes: {
        pulse_slow: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.5" },
        },
        countdown_tick: {
          "0%": { transform: "scale(1.05)" },
          "100%": { transform: "scale(1)" },
        },
      },
      animation: {
        pulse_slow: "pulse_slow 2s ease-in-out infinite",
        tick: "countdown_tick 0.1s ease-out",
      },
    },
  },
  plugins: [],
} satisfies Config;
