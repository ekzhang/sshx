import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

export default defineConfig({
  plugins: [sveltekit()],

  server: {
    proxy: {
      "/api": {
        target: "http://[::1]:8051",
        changeOrigin: true,
        ws: true,
      },
    },
  },
});
