import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
// @ts-expect-error create-tauri-app intentionally avoids a direct @types/node dependency.
import process from "node:process";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  // Tauri serves the production bundle from a custom protocol origin.
  // Relative asset URLs keep imported artwork inside frontendDist.
  base: "./",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
