<script lang="ts">
  import { spring } from "svelte/motion";

  export let x: number;
  export let y: number;

  const xpos = spring(x, { stiffness: 0.3, damping: 0.8 });
  const ypos = spring(y, { stiffness: 0.3, damping: 0.8 });

  $: xpos.set(x);
  $: ypos.set(y);
</script>

<div style:transform="translate({$xpos}px, {$ypos}px)">
  <slot />
</div>
