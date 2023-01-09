<script lang="ts">
  import { makeToast } from "$lib/toast";
  import { createEventDispatcher } from "svelte";
  import { fade } from "svelte/transition";

  import CircleButton from "./CircleButton.svelte";
  import CircleButtons from "./CircleButtons.svelte";

  const dispatch = createEventDispatcher<{ close: void }>();
</script>

<div
  class="panel flex flex-col h-full max-h-[480px]"
  transition:fade|local={{ duration: 150 }}
>
  <div class="flex items-center p-3">
    <CircleButtons>
      <CircleButton kind="red" on:click={() => dispatch("close")} />
    </CircleButtons>
    <div class="ml-2.5 text-gray-300 text-sm font-bold">Chat Messages</div>
  </div>

  <div class="px-3 py-2 flex-1 overflow-y-auto">
    <div class="w-40 h-8 mb-2 rounded-lg bg-zinc-800 animate-pulse" />
    <div class="w-60 h-8 mb-2 rounded-lg bg-zinc-800 animate-pulse" />
    <div class="w-48 h-8 mb-2 rounded-lg bg-zinc-800 animate-pulse" />

    <div class="w-48 h-8 mt-6 mb-2 rounded-lg bg-zinc-800 animate-pulse" />
    <div class="w-52 h-14 mb-2 rounded-lg bg-zinc-800 animate-pulse" />
  </div>

  <form
    class="p-3"
    on:submit|preventDefault={() =>
      makeToast({ kind: "error", message: "Chat isn't implemented yet!" })}
  >
    <input
      class="w-full rounded-lg bg-zinc-800 px-3 py-1.5 outline-none text-gray-300"
      placeholder="Aa"
    />
  </form>
</div>
