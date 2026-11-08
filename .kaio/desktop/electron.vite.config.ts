import { resolve } from "path";
import { defineConfig, externalizeDepsPlugin } from "electron-vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
    resolve: {
      alias: {
        "@shared": resolve("src/shared"),
        "@main": resolve("src/main"),
        "@engine": resolve("src/engine")
      }
    },
    build: {
      outDir: resolve("out/main"),
      rollupOptions: {
        input: {
          index: resolve("src/main/index.ts"),
          "engine-entry": resolve("src/engine/entry.ts")
        },
        output: {
          entryFileNames: "[name].js",
          chunkFileNames: "[name]-[hash].js",
          assetFileNames: "[name]-[hash][extname]"
        },
        external: ["node-pty"]
      }
    }
  },

  preload: {
    plugins: [externalizeDepsPlugin()],
    resolve: {
      alias: {
        "@shared": resolve("src/shared"),
        "@preload": resolve("src/preload")
      }
    },
    build: {
      outDir: resolve("out/preload"),
      rollupOptions: {
        input: {
          index: resolve("src/preload/index.ts")
        },
        output: {
          format: "cjs",
          entryFileNames: "[name].js"
        }
      }
    }
  },

  renderer: {
    root: resolve("src/renderer"),
    base: "./",
    resolve: {
      alias: {
        "@shared": resolve("src/shared"),
        "@renderer": resolve("src/renderer")
      }
    },
    plugins: [react(), tailwindcss()],
    build: {
      outDir: resolve("out/renderer"),
      rollupOptions: {
        input: resolve("src/renderer/index.html")
      }
    }
  }
});
