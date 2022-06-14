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
      fontWeight: 400,
      fontWeightBold: 500,
      scrollback: 5000,
      theme: {}, // TODO: Add theme
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

<div class="inline-block" bind:this={termEl} />
