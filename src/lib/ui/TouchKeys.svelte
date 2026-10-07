<!-- @component Extra terminal keys for touch devices, which lack them on the on-screen keyboard. -->
<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";

  /** Whether the next typed key should be sent with Ctrl held. */
  export let ctrl = false;

  const dispatch = createEventDispatcher<{ data: string }>();

  const keys: [label: string, data: string][] = [
    ["Esc", "\x1b"],
    ["Tab", "\t"],
    ["←", "\x1b[D"],
    ["↑", "\x1b[A"],
    ["↓", "\x1b[B"],
    ["→", "\x1b[C"],
  ];

  // Distance from the bottom of the layout viewport to the top of the
  // on-screen keyboard, so the bar stays visible above the keyboard.
  let bottom = 0;

  onMount(() => {
    const viewport = window.visualViewport;
    if (!viewport) return;
    const update = () => {
      bottom = Math.max(
        0,
        window.innerHeight - viewport.height - viewport.offsetTop,
      );
    };
    update();
    viewport.addEventListener("resize", update);
    viewport.addEventListener("scroll", update);
    return () => {
      viewport.removeEventListener("resize", update);
      viewport.removeEventListener("scroll", update);
    };
  });
</script>

<!--
  Keys act on pointerdown with preventDefault(), so the terminal keeps focus
  and the on-screen keyboard stays open.
-->
<div
  class="fixed inset-x-0 z-20 flex gap-1.5 p-1.5 overflow-x-auto bg-zinc-900/95 border-t border-zinc-800"
  style:bottom="{bottom}px"
>
  <button
    class="key"
    class:active={ctrl}
    on:pointerdown|preventDefault={() => (ctrl = !ctrl)}>Ctrl</button
  >
  {#each keys as [label, data] (label)}
    <button
      class="key"
      on:pointerdown|preventDefault={() => dispatch("data", data)}
      >{label}</button
    >
  {/each}
</div>

<style lang="postcss">
  .key {
    @apply flex-1 min-w-[44px] h-10 px-2 rounded-md text-sm font-medium;
    @apply bg-zinc-800 text-zinc-200 active:bg-zinc-700 select-none;
  }

  .key.active {
    @apply bg-indigo-600 text-white;
  }
</style>
