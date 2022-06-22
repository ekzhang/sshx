<!-- @component Interactive terminal rendered with xterm.js -->
<script lang="ts" context="module">
  // Deduplicated terminal font loading.
  const waitForFonts = (() => {
    let state: "initial" | "loading" | "loaded" = "initial";
    const waitlist: (() => void)[] = [];

    return async function waitForFonts() {
      if (state === "loaded") return;
      else if (state === "initial") {
        const FontFaceObserver = (await import("fontfaceobserver")).default;
        state = "loading";
        try {
          await new FontFaceObserver("Fira Code VF").load();
        } catch (error) {
          console.warn("Could not load terminal font", error);
        }
        state = "loaded";
        for (const fn of waitlist) fn();
      } else {
        await new Promise((resolve) => {
          if (state === "loaded") resolve(null);
          else waitlist.push(() => resolve(null));
        });
      }
    };
  })();
</script>

<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from "svelte";
  import type { Terminal } from "xterm";

  // Hybrid theme from https://terminal.sexy/, using Alacritty export format.
  const theme = {
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

  const dispatch = createEventDispatcher<{ key: string }>();

  export let rows: number, cols: number;
  export let write: (data: string) => void; // bound function prop

  let termEl: HTMLDivElement;
  let term: Terminal | null = null;

  const preloadBuffer: string[] = [];

  write = (data: string) => {
    if (!term) {
      // Before the terminal is loaded, push data into a buffer.
      preloadBuffer.push(data);
    } else {
      term.write(data);
    }
  };

  $: term?.resize(cols, rows);

  onMount(async () => {
    const { Terminal } = await import("xterm");
    await waitForFonts();

    term = new Terminal({
      allowTransparency: true,
      cursorBlink: false,
      cursorStyle: "block",
      // This is the monospace font family configured in Tailwind.
      fontFamily:
        '"Fira Code VF", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
      fontSize: 14,
      fontWeight: 400,
      fontWeightBold: 500,
      lineHeight: 1.06,
      scrollback: 5000,
      theme,
    });

    term.open(termEl);
    term.resize(cols, rows);
    for (const data of preloadBuffer) {
      term.write(data);
    }

    term.onKey(({ key }) => {
      dispatch("key", key);
    });
  });

  onDestroy(() => term?.dispose());
</script>

<div class="inline-block rounded-lg" style:background={theme.background}>
  <div class="text-center p-2 text-sm text-gray-400 font-bold">
    Remote Terminal
  </div>
  <div class="inline-block px-5 py-2" bind:this={termEl} />
</div>
