const defaultTheme = require("tailwindcss/defaultTheme");

/** @type {import("tailwindcss").Config} */
const config = {
  content: ["./src/**/*.{html,js,svelte,ts}"],

  darkMode: "class",
  theme: {
    extend: {
      fontFamily: {
        sans: ["Inter Variable", ...defaultTheme.fontFamily.sans],
        mono: ["Fira Code VF", ...defaultTheme.fontFamily.mono],
      },
    },
  },

  plugins: [],
};

module.exports = config;
