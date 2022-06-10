<script lang="ts">
  import { page } from "$app/stores";

  import { onMount } from "svelte";

  let termEl: HTMLDivElement;

  onMount(async () => {
    const { Terminal } = await import("xterm");
    const term = new Terminal();
    term.open(termEl);
    term.write("Hello from \x1B[1;3;31mxterm.js\x1B[0m $ ");
  });
</script>

This is the page for session {$page.params.id}.

<div bind:this={termEl} />
