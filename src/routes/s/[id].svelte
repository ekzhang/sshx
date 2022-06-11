<script lang="ts">
  import { page } from "$app/stores";

  import { onMount } from "svelte";

  let termEl: HTMLDivElement;

  onMount(async () => {
    const { Terminal } = await import("xterm");
    const FontFaceObserver = (await import("fontfaceobserver")).default;

    // This is the monospace font family configured in Tailwind.
    let fontFamily =
      '"Fira Code", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace';
    try {
      await Promise.all([
        new FontFaceObserver("Fira Code").load(),
        new FontFaceObserver("Fira Code", { weight: "bold" }).load(),
      ]);
    } catch (error) {
      console.warn("Could not load terminal font", error);
    }

    const term = new Terminal({
      allowTransparency: true,
      cursorBlink: false,
      cursorStyle: "block",
      fontFamily,
      fontWeight: 400,
      fontWeightBold: 700,
      scrollback: 5000,
      theme: {}, // TODO: Add theme
    });
    term.open(termEl);
    term.write("Hello from \x1B[1;3;31mxterm.js\x1B[0m $ ");

    term.onKey(({ key }) => {
      console.log(key.charCodeAt(0));
      term.write(key);
    });

    term.resize(100, 10);
  });
</script>

This is the page for session {$page.params.id}.

<div>
  <div class="inline-block" bind:this={termEl} />
</div>
