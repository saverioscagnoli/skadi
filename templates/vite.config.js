import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { homedir } from "os";
import fs from "fs";

const localDir = `${homedir()}/.local/share/wwwidgets`;
const htmlDir = `${localDir}/html`;
const buildDir = `${localDir}/build`;
const configDir = `${homedir()}/.config/wwwidgets`;

if (!fs.existsSync(localDir)) {
  fs.mkdirSync(localDir, { recursive: true });
}

if (!fs.existsSync(configDir)) {
  fs.mkdirSync(configDir, { recursive: true });
}

function resolveHtmlIndices() {
  return fs.readdirSync(htmlDir).map((f) => `${htmlDir}/${f}`);
}

// https://vite.dev/config/
export default defineConfig({
  base: "./",
  plugins: [react({
    
  })],
  build: {
    rollupOptions: {
      input: resolveHtmlIndices(),
    },
    outDir: buildDir,
    emptyOutDir: true,
  },
  server: {
    fs: {
      allow: [localDir, configDir],
    },
  },
});
