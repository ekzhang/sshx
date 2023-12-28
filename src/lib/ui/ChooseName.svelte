<script lang="ts">
  import { browser } from "$app/environment";

  import OverlayMenu from "./OverlayMenu.svelte";
  import { settings } from "$lib/settings";

  let value = "";

  function handleSubmit() {
    settings.set({ ...settings, name: value });
  }
</script>

<OverlayMenu
  title="Welcome!"
  description="Before you join — what should we call you?"
  maxWidth={640}
  open={browser && !$settings.name}
>
  <form class="flex gap-2" on:submit|preventDefault={handleSubmit}>
    <input
      class="flex-1 w-full px-3 py-2 rounded outline-none text-zinc-300 bg-zinc-800"
      placeholder="Your name"
      required
      minlength="2"
      maxlength="50"
      bind:value
    />
    <button
      class="flex-shrink-0 px-3 py-2 bg-pink-700 hover:bg-pink-600 active:ring-4 active:ring-pink-500/50 rounded font-medium"
      >Join</button
    >
  </form>
</OverlayMenu>
