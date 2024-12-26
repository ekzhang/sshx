<script lang="ts">
  import { flip } from "svelte/animate";

  import type { WsUser } from "$lib/protocol";
  import { nameToHue } from "./LiveCursor.svelte";

  export let users: [number, WsUser][];
  $: sortedUsers = [...users].sort(
    (a, b) => Number(b[1].canWrite) - Number(a[1].canWrite),
  );
</script>

<ul class="flex flex-col">
  {#each sortedUsers as [id, user] (id)}
    <li
      class={`flex p-1 gap-3 items-center ${user.canWrite ? "" : "opacity-75"}`}
      animate:flip={{ duration: 250 }}
    >
      <div
        style:background="hsl({nameToHue(user.name)}, 75%, 60%)"
        class="w-3.5 h-3.5 rounded-full"
      />
      <div
        class="text-sm font-medium bg-zinc-800 px-1.5 py-0.5 rounded text-zinc-300"
      >
        {user.name}
      </div>
    </li>
  {/each}
</ul>
