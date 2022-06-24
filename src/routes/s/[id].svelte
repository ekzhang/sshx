<script lang="ts">
  import { page } from "$app/stores";

  import { onDestroy, onMount } from "svelte";

  import { Srocket } from "$lib/srocket";
  import type { WsClient, WsServer } from "$lib/types";
  import XTerm from "$lib/XTerm.svelte";
  import logotypeDark from "$lib/assets/logotype-dark.svg";

  let srocket: Srocket<WsServer, WsClient> | null = null;

  /** Bound "write" method for each terminal. */
  const writers: ((data: string) => void)[] = [];

  let connected = false;
  let exitReason: string | null = null;

  onMount(() => {
    srocket = new Srocket<WsServer, WsClient>(`/api/s/${$page.params.id}`, {
      onMessage(message) {
        console.log(message);
        if (message.chunks) {
          const [id, chunks] = message.chunks;
          for (const chunk of chunks) {
            writers[id](chunk[1]);
          }
        } else if (message.shells) {
          if (message.shells.includes(0)) {
            srocket?.send({ subscribe: [0, 0] });
          }
        } else if (message.terminated) {
          console.log("terminated!");
          exitReason = "The session has been terminated";
          srocket?.dispose();
        }
      },

      onConnect() {
        connected = true;
      },

      onDisconnect() {
        connected = false;
      },

      onClose(event) {
        if (event.code === 4404) {
          exitReason = "Failed to connect: " + event.reason;
        }
      },
    });

    // TODO: Implement actual client logic.
    srocket?.send({ create: [] });
  });

  onDestroy(() => srocket?.dispose());

  function handleData(id: number, data: Uint8Array) {
    srocket?.send({ data: [id, data] });
  }
</script>

<main class="p-8">
  <img class="h-16 -mx-2 mb-2" src={logotypeDark} alt="sshx logo" />

  <p>
    This is the page for session <code class="text-violet-300"
      >{$page.params.id}</code
    >.
  </p>

  <div class="py-2">
    {#if exitReason !== null}
      <div class="text-red-400">{exitReason}</div>
    {:else if connected}
      <div class="text-green-400">You are connected!</div>
    {:else}
      <div class="text-yellow-400">Connecting…</div>
    {/if}
  </div>

  <div class="py-6">
    <XTerm
      rows={24}
      cols={80}
      bind:write={writers[0]}
      on:data={({ detail }) => handleData(0, detail)}
    />
  </div>
</main>
