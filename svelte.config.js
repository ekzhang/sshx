import http from "http";

import adapter from "@sveltejs/adapter-static";
import preprocess from "svelte-preprocess";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: [
    preprocess({
      postcss: true,
    }),
  ],

  kit: {
    adapter: adapter({
      fallback: "spa.html", // SPA mode
      precompress: true,
    }),

    vite: {
      server: {
        proxy: {
          "/api": {
            target: "http://[::1]:8051",
            changeOrigin: true,
            ws: true,
            agent: new http.Agent(), // See https://github.com/vitejs/vite/issues/4794
          },
        },
      },
    },
  },
};

export default config;
