<script lang="ts">
  import type { WsUser } from "$lib/protocol";

  export let users: WsUser[];

  function nameToInitials(name: string): string {
    const parts = name.split(/\s/).filter((s) => s);
    if (parts.length === 0) {
      return "-";
    } else if (parts.length === 1) {
      return parts[0][0].toLocaleUpperCase();
    } else {
      return (parts[0][0] + parts[1][0]).toLocaleUpperCase();
    }
  }
</script>

<div>
  {#each users as user}
    {nameToInitials(user.name)}
  {/each}
</div>
