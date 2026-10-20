import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri loads the built `dist/` in release, and this dev server on port 1420 in `tauri dev`.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { outDir: "dist", emptyOutDir: true },
});
