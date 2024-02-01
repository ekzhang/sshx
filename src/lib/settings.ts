import { persisted } from "svelte-persisted-store";
import type themes from "./ui/themes.ts";

export type SettingsStore = {
  name: string;
  theme: keyof typeof themes;
};

/** A persisted store for settings of the current user. */
export const settings = persisted<SettingsStore>("sshx-settings-store", {
  name: "",
  theme: "defaultDark",
});
