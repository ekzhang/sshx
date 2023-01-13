<script lang="ts">
  import { onMount } from "svelte";
  import { flip } from "svelte/animate";
  import { fly } from "svelte/transition";
  import { Portal } from "@rgossiaux/svelte-headlessui";

  import Toast from "./Toast.svelte";
  import { toastStore } from "$lib/toast";

  onMount(() => {
    // Remove old toasts periodically.
    const id = setInterval(() => {
      const now = Date.now();
      toastStore.update(($toasts) => $toasts.filter((t) => t.expires > now));
    }, 250);
    return () => clearInterval(id);
  });
</script>

<Portal>
  <div class="fixed inset-0 z-40 pointer-events-none flex justify-end p-4">
    <div class="w-full max-w-md">
      {#each $toastStore.slice().reverse() as toast (toast)}
        <div
          class="mb-2"
          on:click={() =>
            ($toastStore = $toastStore.filter((t) => t !== toast))}
          on:keypress={() => null}
          animate:flip={{ duration: 500 }}
          transition:fly={{ x: 360, duration: 500 }}
        >
          <Toast
            kind={toast.kind}
            message={toast.message}
            action={toast.action}
            on:action={toast.onAction ?? (() => null)}
          />
        </div>
      {/each}
    </div>
  </div>
</Portal>
