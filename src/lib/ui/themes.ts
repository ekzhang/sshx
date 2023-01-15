import type { ITheme } from "sshx-xterm";

/** VSCode default dark theme, from https://glitchbone.github.io/vscode-base16-term/. */
export const defaultDark: ITheme = {
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
export const hybrid: ITheme = {
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

export default { defaultDark, hybrid };
