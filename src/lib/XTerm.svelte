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

  import themes from "./themes";

  const theme = themes.defaultDark;

  const dispatch = createEventDispatcher<{ data: string }>();

  export let rows: number, cols: number;
  export let write: (data: string) => void; // bound function prop

  let termEl: HTMLDivElement;
  let term: Terminal | null = null;

  let loaded = false;
  let currentTitle = "Remote Terminal";

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
    const { WebLinksAddon } = await import("xterm-addon-web-links");

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

    term.loadAddon(new WebLinksAddon());

    term.open(termEl);
    term.resize(cols, rows);
    term.onTitleChange((title) => {
      currentTitle = title;
    });

    loaded = true;
    for (const data of preloadBuffer) {
      term.write(data);
    }

    term.onData((data) => {
      dispatch("data", data);
    });
  });

  onDestroy(() => term?.dispose());
</script>

<div
  class="inline-block rounded-lg border border-gray-600 transition-opacity duration-500"
  style:background={theme.background}
  style:opacity={loaded ? "95%" : "0%"}
>
  <div class="flex cursor-pointer select-none">
    <div class="flex-1 flex items-center space-x-2 px-3">
      <div class="w-3 h-3 rounded-full bg-red-500" />
      <div class="w-3 h-3 rounded-full bg-yellow-500" />
      <div class="w-3 h-3 rounded-full bg-green-500" />
    </div>
    <div class="flex-shrink-0 p-2 text-sm text-gray-300 font-bold">
      {currentTitle}
    </div>
    <div class="flex-1" />
  </div>
  <div class="inline-block px-4 py-2" bind:this={termEl} />
</div>
