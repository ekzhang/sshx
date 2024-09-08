<script lang="ts">
  import { ExternalLinkIcon } from "svelte-feather-icons";

  export let href: string;
</script>

<a
  {href}
  class="flex items-baseline gap-1.5 py-0.5 px-1.5 rounded bg-white/[7%] hover:bg-white/[15%] border border-transparent active:border-white/50 text-zinc-300 transition-colors"
>
  <span class="text-sm">
    <slot />
  </span>
  <ExternalLinkIcon size="12" class="text-zinc-400" />
</a>
