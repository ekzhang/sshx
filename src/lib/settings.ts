import { persisted } from "svelte-persisted-store";

export type SettingsStore = {
  name: string;
};

/** A persisted store for settings of the current user. */
export const settings = persisted<SettingsStore>("sshx-settings-store", {
  name: "",
});
