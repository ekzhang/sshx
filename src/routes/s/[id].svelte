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

    const term = new Terminal({ fontFamily });
    term.open(termEl);
    term.write("Hello from \x1B[1;3;31mxterm.js\x1B[0m $ ");
  });
</script>

This is the page for session {$page.params.id}.

<div bind:this={termEl} />
