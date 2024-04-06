import { persisted } from "svelte-persisted-store";
import { type ThemeName, defaultTheme } from "./ui/themes";

export type SettingsStore = {
  name: string;
  theme: ThemeName;
};

/** A persisted store for settings of the current user. */
export const settings = persisted<SettingsStore>("sshx-settings-store", {
  name: "",
  theme: defaultTheme,
});
