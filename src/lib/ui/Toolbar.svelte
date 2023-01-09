<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    MessageSquareIcon,
    PlusCircleIcon,
    SettingsIcon,
    WifiIcon,
  } from "svelte-feather-icons";

  import logo from "$lib/assets/logo.svg";

  export let connected: boolean;

  const dispatch = createEventDispatcher<{ create: void }>();
</script>

<div
  class="inline-block border border-zinc-800 bg-zinc-900/60 backdrop-blur-sm rounded-xl px-3 py-2"
>
  <div class="flex items-center select-none">
    <a href="/"><img src={logo} alt="sshx logo" class="h-10" /></a>
    <p class="ml-1.5 mr-2 font-medium">sshx</p>

    <div class="v-divider" />

    <div class="flex space-x-1">
      <button
        class="icon-button"
        on:click={() => dispatch("create")}
        disabled={!connected}
      >
        <PlusCircleIcon strokeWidth={1.5} class="p-0.5" />
      </button>
      <button class="icon-button">
        <MessageSquareIcon strokeWidth={1.5} class="p-0.5" />
      </button>
      <button class="icon-button">
        <SettingsIcon strokeWidth={1.5} class="p-0.5" />
      </button>
    </div>

    <div class="v-divider" />

    <div class="flex space-x-1">
      <button class="icon-button">
        <WifiIcon strokeWidth={1.5} class="p-0.5" />
      </button>
    </div>
  </div>
</div>

<style lang="postcss">
  .v-divider {
    @apply h-5 mx-2 border-l-4 border-zinc-800;
  }

  .icon-button {
    @apply rounded-md p-1 hover:bg-zinc-700 active:bg-indigo-700 transition-colors;
    @apply disabled:opacity-50 disabled:bg-transparent;
  }
</style>
