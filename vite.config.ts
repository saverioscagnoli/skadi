import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { homedir } from "os";
import fs from "fs";
import tsconfigPaths from "vite-tsconfig-paths";

const CONFIG_PATH = `${homedir()}/.config/skadi/config.json`;

type Config = {
  windows: {
    label: string;
  }[];
};

function resolveHtmlEntryPoints(): string[] {
  let raw = fs.readFileSync(CONFIG_PATH, "utf-8");
  let config: Config = JSON.parse(raw);

  return config.windows.map(w => `html/${w.label}.html`);
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss(), tsconfigPaths()],
  publicDir: `${homedir()}/.config/skadi/assets`,
  build: {
    rollupOptions: {
      input: resolveHtmlEntryPoints()
    }
  },
  server: {
    fs: {
      allow: [`${homedir()}/.config/skadi/plugins`]
    }
  }
});
