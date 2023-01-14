<script lang="ts">
  import { fade } from "svelte/transition";

  import type { WsUser } from "$lib/protocol";

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
  class="flex items-center"
  on:mouseenter={() => (hovering = true)}
  on:mouseleave={() => (hovering = false)}
>
  <svg width="27" height="27" viewBox="0 0 27 27">
    <path d="M14 26L2 2L26 14H14V26Z" fill="#76FF84" stroke="#111111" />
  </svg>
  {#if showName || hovering || time - lastMove < 1500}
    <p
      class="ml-1 bg-zinc-600 text-sm px-1.5 py-[1px] rounded-sm font-bold"
      transition:fade|local={{ duration: 150 }}
    >
      {user.name}
    </p>
  {/if}
</div>
