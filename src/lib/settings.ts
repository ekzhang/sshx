import { persisted } from "svelte-persisted-store";
import themes, { type ThemeName, defaultTheme } from "./ui/themes";
import { derived, type Readable } from "svelte/store";

export type Settings = {
  name: string;
  theme: ThemeName;
  scrollback: number;
};

const storedSettings = persisted<Partial<Settings>>("sshx-settings-store", {});

/** A persisted store for settings of the current user. */
export const settings: Readable<Settings> = derived(
  storedSettings,
  ($storedSettings) => {
    // Do some validation on all of the stored settings.
    const name = $storedSettings.name ?? "";

    let theme = $storedSettings.theme;
    if (!theme || !Object.hasOwn(themes, theme)) {
      theme = defaultTheme;
    }

    let scrollback = $storedSettings.scrollback;
    if (typeof scrollback !== "number" || scrollback < 0) {
      scrollback = 5000;
    }

    return {
      name,
      theme,
      scrollback,
    };
  },
);

export function updateSettings(values: Partial<Settings>) {
  storedSettings.update((settings) => ({ ...settings, ...values }));
}
