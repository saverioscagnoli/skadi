import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { homedir } from "os";
import fs from "fs";

const CONFIG_PATH = `${homedir()}/.config/skadi/config.json`;

/**
 * @typedef {Object} Config
 * @property {Object[]} windows - Array of window configurations.
 * @property {string} windows[].label - The label for the window.
 */

/**
 * @returns {string[]} - Array of HTML entry points based on the window labels in the config.
 */
function resolveHtmlEntryPoints() {
  let raw = fs.readFileSync(CONFIG_PATH, "utf-8");
  /** @type {Config} */
  let config = JSON.parse(raw);

  return config.windows.map((w) => `html/${w.label}.html`);
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  publicDir: `${homedir()}/.config/skadi/assets`,
  build: {
    rollupOptions: {
      input: resolveHtmlEntryPoints(),
    },
  },
  server: {
    fs: {
      allow: [`${homedir()}/.config/skadi/plugins`],
    },
  },
});
