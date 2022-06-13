<script lang="ts">
  import { page } from "$app/stores";

  import { onDestroy, onMount } from "svelte";

  import { Srocket } from "$lib/srocket";
  import type { WsClient, WsServer } from "$lib/types";
  import XTerm from "$lib/XTerm.svelte";

  let srocket: Srocket<WsServer, WsClient> | null = null;

  /** Bound "write" method for each terminal. */
  const writers: ((data: string) => void)[] = [];

  onMount(() => {
    srocket = new Srocket<WsServer, WsClient>(`/api/s/${$page.params.id}`, {
      onMessage(message) {
        if (message.chunks) {
          const [id, chunks] = message.chunks;
          for (const chunk of chunks) {
            writers[id](chunk[1]);
          }
        } else if (message.shells) {
          if (message.shells.includes(0)) {
            srocket?.send({ subscribe: [0, 0] });
          }
        }
      },
    });

    // TODO: Implement actual client logic.
    srocket?.send({ create: null });
  });

  onDestroy(() => srocket?.dispose());

  function handleKey(id: number, key: string) {
    srocket?.send({ data: [id, key] });
  }
</script>

This is the page for session {$page.params.id}.

<div>
  <XTerm
    rows={24}
    cols={80}
    bind:write={writers[0]}
    on:key={({ detail }) => handleKey(0, detail)}
  />
</div>
