import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { homedir } from "os";
import fs from "fs";
import postcss from "@tailwindcss/postcss";

const LOCAL_PATH = `${homedir()}/.local/share/skadi/html`;

/**
 * @typedef {Object} Config
 * @property {Object[]} windows - Array of window configurations.
 * @property {string} windows[].label - The label for the window.
 */

/**
 * @returns {string[]} - Array of HTML entry points based on the window labels in the config.
 */
function resolveHtmlEntryPoints() {
  let rd = fs.readdirSync(LOCAL_PATH);
  return rd.map((f) => `html/${f}`);
}

// https://vite.dev/config/
export default defineConfig({
  base: "./",
  plugins: [react(), tailwindcss()],
  css: {
    postcss: {
      plugins: [postcss],
    },
  },
  publicDir: `${homedir()}/.config/skadi/assets`,
  build: {
    rollupOptions: {
      input: resolveHtmlEntryPoints(),
    },
  },
  server: {
    fs: {
      allow: [`${homedir()}/.config/skadi`, `${homedir()}/.local/share/skadi`],
    },
  },
});
