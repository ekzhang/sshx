<script lang="ts" context="module">
  import type { WsUser } from "$lib/protocol";

  /** Convert a string into a unique, hashed hue from 0 to 360. */
  export function nameToHue(name: string): number {
    // Hashes the string with FNV.
    let hash = 2166136261;
    for (let i = 0; i < name.length; i++) {
      hash = (hash ^ name.charCodeAt(i)) * 16777619;
    }
    hash = (hash * 16777619) ^ -1;
    return 360 * (hash / (1 << 31));
  }
</script>

<script lang="ts">
  import { fade } from "svelte/transition";

  export let user: WsUser;
  export let showName = false;

  let hovering = false;
  let lastMove = Date.now();

  let lastCursor: [number, number] | null = null;
  let time = Date.now();
  $: if (
    !lastCursor ||
    (user.cursor &&
      (lastCursor[0] !== user.cursor[0] || lastCursor[1] != user.cursor[1]))
  ) {
    lastCursor = user.cursor;
    lastMove = Date.now();
    setTimeout(() => {
      time = Date.now();
    }, 1600);
  }
</script>

<div
  class="flex items-start"
  on:mouseenter={() => (hovering = true)}
  on:mouseleave={() => (hovering = false)}
>
  <svg width="23" height="23" viewBox="0 0 23 23">
    <path
      d="M11 22L2 2L22 11L14 14Z"
      fill="hsl({nameToHue(user.name)}, 100%, 50%)"
      stroke="white"
    />
  </svg>
  {#if showName || hovering || time - lastMove < 1500}
    <p
      class="mt-4 bg-zinc-700 text-xs px-1.5 py-[1px] rounded font-medium"
      transition:fade|local={{ duration: 150 }}
    >
      {user.name}
    </p>
  {/if}
</div>
