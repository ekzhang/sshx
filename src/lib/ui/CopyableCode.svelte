<script lang="ts">
  import { CheckIcon, CopyIcon } from "svelte-feather-icons";

  export let value: string;

  let copied = false;

  async function handleClick() {
    await navigator.clipboard.writeText(value);
    copied = true;
    setTimeout(() => {
      copied = false;
    }, 1000);
  }
</script>

<div class="flex items-center gap-4">
  <code class="text-zinc-100">{value}</code>
  <button
    class={"rounded p-1.5 transition-colors " +
      (!copied ? "hover:bg-white/10" : "hover:bg-green-500/10")}
    on:click={handleClick}
  >
    {#if copied}
      <CheckIcon size="16" class="text-green-400" />
    {:else}
      <CopyIcon size="16" />
    {/if}
  </button>
</div>
