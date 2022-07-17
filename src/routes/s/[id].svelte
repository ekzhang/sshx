<script lang="ts">
  import { page } from "$app/stores";

  import { onDestroy, onMount, tick } from "svelte";
  import { fade } from "svelte/transition";

  import { Srocket } from "$lib/srocket";
  import type { WsClient, WsServer, WsWinsize } from "$lib/types";
  import Toolbar from "$lib/Toolbar.svelte";
  import XTerm from "$lib/XTerm.svelte";

  let srocket: Srocket<WsServer, WsClient> | null = null;

  let connected = false;
  let exitReason: string | null = null;

  /** Bound "write" method for each terminal. */
  const writers: Record<number, (data: string) => void> = {};
  const seqnums: Record<number, number> = {};
  let shells: [number, WsWinsize][] = [];
  let subscriptions = new Set<number>();
  const pos: Record<number, { x: number; y: number }> = {};

  onMount(() => {
    srocket = new Srocket<WsServer, WsClient>(`/api/s/${$page.params.id}`, {
      onMessage(message) {
        if (message.chunks) {
          const [id, chunks] = message.chunks;
          tick().then(() => {
            seqnums[id] += chunks.length;
            for (const [, data] of chunks) {
              writers[id](data);
            }
          });
        } else if (message.shells) {
          shells = message.shells;
          for (const [id, _winsize] of message.shells) {
            if (!subscriptions.has(id)) {
              seqnums[id] ??= 0;
              subscriptions.add(id);
              srocket?.send({ subscribe: [id, seqnums[id]] });
            }
          }
        } else if (message.terminated) {
          exitReason = "The session has been terminated";
          srocket?.dispose();
        } else if (message.error) {
          // TODO: Add an actual toast notification.
          console.error("Server error: " + message.error);
        }
      },

      onConnect() {
        connected = true;
      },

      onDisconnect() {
        connected = false;
        subscriptions.clear();
      },

      onClose(event) {
        if (event.code === 4404) {
          exitReason = "Failed to connect: " + event.reason;
        }
      },
    });
  });

  onDestroy(() => srocket?.dispose());

  function handleData(id: number, data: Uint8Array) {
    srocket?.send({ data: [id, data] });
  }
</script>

<main class="p-8">
  <div class="absolute top-8 left-1/2 -translate-x-1/2 inline-block z-10">
    <Toolbar {connected} on:create={() => srocket?.send({ create: [] })} />
  </div>

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
    {#each shells as [id, _winsize] (id)}
      <div
        class="inline-block"
        style:transform="translate({[pos[id]?.x ?? 0]}px, {pos[id]?.y ?? 0}px)"
        transition:fade|local
      >
        <XTerm
          rows={24}
          cols={80}
          bind:write={writers[id]}
          on:data={({ detail }) => handleData(id, detail)}
          on:move={({ detail }) => {
            pos[id] = {
              x: (pos[id]?.x ?? 0) + detail.x,
              y: (pos[id]?.y ?? 0) + detail.y,
            };
          }}
          on:close={() => srocket?.send({ close: id })}
        />
      </div>
    {/each}
  </div>
</main>
