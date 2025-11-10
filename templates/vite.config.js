import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { homedir } from "os";
import fs from "fs";
import path from "path";
import tailwindcss from "@tailwindcss/vite";
import postcss from "@tailwindcss/postcss";

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
  plugins: [
    react({
      babel: {
        plugins: [["babel-plugin-react-compiler"]],
      },
    }),
    tailwindcss(),
  ],
  css: {
    postcss: {
      plugins: [postcss],
    },
  },
  build: {
    rollupOptions: {
      input: resolveHtmlIndices(),
    },
    outDir: buildDir,
    emptyOutDir: true,
  },
  resolve: {
    alias: {
      // Force resolving node_modules to local_dir/node_modules, otherwise
      // if react is not installed in the config directory it will throw
      react: path.resolve(localDir, "node_modules/react"),
      "react-dom": path.resolve(localDir, "node_modules/react-dom"),
    },
  },
  server: {
    fs: {
      allow: [localDir, configDir],
    },
    headers: {
      "Cache-Control":
        "no-store, no-cache, must-revalidate, proxy-revalidate, max-age=0",
      Pragma: "no-cache",
      Expires: "0",
    },
    proxy: {
      "/backend": {
        target: "http://localhost:10978",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/backend/, ""),
      },
    },
  },
  optimizeDeps: {
    include: ["react", "react-dom"],
  },
});
