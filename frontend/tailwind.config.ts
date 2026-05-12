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
        // Virtuals.io-matched dark palette
        surface: {
          DEFAULT: "#000000",
          card: "#060c18",
          elevated: "#0c1e2a",
        },
        accent: {
          purple: "#3B82F6",   // kept for fallback references
          teal: "#44bcc3",     // primary accent — matches virtuals.io
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
        blob: {
          "0%, 100%": { transform: "translate(0px, 0px) scale(1)" },
          "33%":       { transform: "translate(30px, -45px) scale(1.08)" },
          "66%":       { transform: "translate(-25px, 20px) scale(0.94)" },
        },
      },
      animation: {
        pulse_slow: "pulse_slow 2s ease-in-out infinite",
        tick: "countdown_tick 0.1s ease-out",
        blob: "blob 8s ease-in-out infinite",
      },
    },
  },
  plugins: [],
} satisfies Config;
