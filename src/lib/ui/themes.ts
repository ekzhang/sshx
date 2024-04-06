import type { ITheme } from "sshx-xterm";

/** VSCode default dark theme, from https://glitchbone.github.io/vscode-base16-term/. */
const defaultDark: ITheme = {
  foreground: "#d8d8d8",
  background: "#181818",

  cursor: "#d8d8d8",

  black: "#181818",
  red: "#ab4642",
  green: "#a1b56c",
  yellow: "#f7ca88",
  blue: "#7cafc2",
  magenta: "#ba8baf",
  cyan: "#86c1b9",
  white: "#d8d8d8",

  brightBlack: "#585858",
  brightRed: "#ab4642",
  brightGreen: "#a1b56c",
  brightYellow: "#f7ca88",
  brightBlue: "#7cafc2",
  brightMagenta: "#ba8baf",
  brightCyan: "#86c1b9",
  brightWhite: "#f8f8f8",
};

/** Hybrid theme from https://terminal.sexy/, using Alacritty export format. */
const hybrid: ITheme = {
  foreground: "#c5c8c6",
  background: "#1d1f21",

  black: "#282a2e",
  red: "#a54242",
  green: "#8c9440",
  yellow: "#de935f",
  blue: "#5f819d",
  magenta: "#85678f",
  cyan: "#5e8d87",
  white: "#707880",

  brightBlack: "#373b41",
  brightRed: "#cc6666",
  brightGreen: "#b5bd68",
  brightYellow: "#f0c674",
  brightBlue: "#81a2be",
  brightMagenta: "#b294bb",
  brightCyan: "#8abeb7",
  brightWhite: "#c5c8c6",
};

/** Below themes are converted from https://github.com/alacritty/alacritty-theme/. */
const rosePine: ITheme = {
  foreground: "#e0def4",
  background: "#191724",

  cursor: "#524f67",

  black: "#26233a",
  red: "#eb6f92",
  green: "#31748f",
  yellow: "#f6c177",
  blue: "#9ccfd8",
  magenta: "#c4a7e7",
  cyan: "#ebbcba",
  white: "#e0def4",

  brightBlack: "#6e6a86",
  brightRed: "#eb6f92",
  brightGreen: "#31748f",
  brightYellow: "#f6c177",
  brightBlue: "#9ccfd8",
  brightMagenta: "#c4a7e7",
  brightCyan: "#ebbcba",
  brightWhite: "#e0def4",
};

const ubuntu: ITheme = {
  foreground: "#eeeeec",
  background: "#300a24",
  black: "#2e3436",
  red: "#cc0000",
  green: "#4e9a06",
  yellow: "#c4a000",
  blue: "#3465a4",
  magenta: "#75507b",
  cyan: "#06989a",
  white: "#d3d7cf",
  brightBlack: "#555753",
  brightRed: "#ef2929",
  brightGreen: "#8ae234",
  brightYellow: "#fce94f",
  brightBlue: "#729fcf",
  brightMagenta: "#ad7fa8",
  brightCyan: "#34e2e2",
  brightWhite: "#eeeeec",
};

const dracula: ITheme = {
  foreground: "#f8f8f2",
  background: "#282a36",
  black: "#000000",
  red: "#ff5555",
  green: "#50fa7b",
  yellow: "#f1fa8c",
  blue: "#bd93f9",
  magenta: "#ff79c6",
  cyan: "#8be9fd",
  white: "#bbbbbb",
  brightBlack: "#555555",
  brightRed: "#ff5555",
  brightGreen: "#50fa7b",
  brightYellow: "#f1fa8c",
  brightBlue: "#caa9fa",
  brightMagenta: "#ff79c6",
  brightCyan: "#8be9fd",
  brightWhite: "#ffffff",
};

const githubDark: ITheme = {
  foreground: "#d1d5da",
  background: "#24292e",
  black: "#586069",
  red: "#ea4a5a",
  green: "#34d058",
  yellow: "#ffea7f",
  blue: "#2188ff",
  magenta: "#b392f0",
  cyan: "#39c5cf",
  white: "#d1d5da",
  brightBlack: "#959da5",
  brightRed: "#f97583",
  brightGreen: "#85e89d",
  brightYellow: "#ffea7f",
  brightBlue: "#79b8ff",
  brightMagenta: "#b392f0",
  brightCyan: "#56d4dd",
  brightWhite: "#fafbfc",
};

const gruvboxDark: ITheme = {
  foreground: "#ebdbb2",
  background: "#282828",
  black: "#282828",
  red: "#cc241d",
  green: "#98971a",
  yellow: "#d79921",
  blue: "#458588",
  magenta: "#b16286",
  cyan: "#689d6a",
  white: "#a89984",
  brightBlack: "#928374",
  brightRed: "#fb4934",
  brightGreen: "#b8bb26",
  brightYellow: "#fabd2f",
  brightBlue: "#83a598",
  brightMagenta: "#d3869b",
  brightCyan: "#8ec07c",
  brightWhite: "#ebdbb2",
};

const solarizedDark: ITheme = {
  foreground: "#839496",
  background: "#002b36",
  black: "#073642",
  red: "#dc322f",
  green: "#859900",
  yellow: "#b58900",
  blue: "#268bd2",
  magenta: "#d33682",
  cyan: "#2aa198",
  white: "#eee8d5",
  brightBlack: "#002b36",
  brightRed: "#cb4b16",
  brightGreen: "#586e75",
  brightYellow: "#657b83",
  brightBlue: "#839496",
  brightMagenta: "#6c71c4",
  brightCyan: "#93a1a1",
  brightWhite: "#fdf6e3",
};

const tokyoNight: ITheme = {
  foreground: "#a9b1d6",
  background: "#1a1b26",
  black: "#32344a",
  red: "#f7768e",
  green: "#9ece6a",
  yellow: "#e0af68",
  blue: "#7aa2f7",
  magenta: "#ad8ee6",
  cyan: "#449dab",
  white: "#787c99",
  brightBlack: "#444b6a",
  brightRed: "#ff7a93",
  brightGreen: "#b9f27c",
  brightYellow: "#ff9e64",
  brightBlue: "#7da6ff",
  brightMagenta: "#bb9af7",
  brightCyan: "#0db9d7",
  brightWhite: "#acb0d0",
};

const themes = {
  "VS Code Dark": defaultDark,
  Hybrid: hybrid,
  "Rosé Pine": rosePine,
  Ubuntu: ubuntu,
  Dracula: dracula,
  "GitHub Dark": githubDark,
  "Gruvbox Dark": gruvboxDark,
  "Solarized Dark": solarizedDark,
  "Tokyo Night": tokyoNight,
};

export type ThemeName = keyof typeof themes;

export const defaultTheme: ThemeName = "VS Code Dark";

export default themes;
