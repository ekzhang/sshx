<script lang="ts">
  import { onDestroy, onMount, tick, beforeUpdate, afterUpdate } from "svelte";
  import { fade } from "svelte/transition";

  import { Srocket } from "./srocket";
  import type { WsClient, WsServer, WsWinsize } from "./protocol";
  import Toolbar from "./ui/Toolbar.svelte";
  import XTerm from "./ui/XTerm.svelte";
  import { slide } from "./ui/slide";

  export let id: string;

  let srocket: Srocket<WsServer, WsClient> | null = null;

  let connected = false;
  let exitReason: string | null = null;

  /** Bound "write" method for each terminal. */
  const writers: Record<number, (data: string) => void> = {};
  const seqnums: Record<number, number> = {};
  let userId = 0;
  let shells: [number, WsWinsize][] = [];
  let subscriptions = new Set<number>();
  let movingOffset = [-1, 0, 0];
  let movingOffsetDone = false;

  onMount(() => {
    srocket = new Srocket<WsServer, WsClient>(`/api/s/${id}`, {
      onMessage(message) {
        if (message.hello) {
          userId = message.hello;
          console.log(`Connected to the server as user ${userId}`);
        } else if (message.chunks) {
          const [id, chunks] = message.chunks;
          tick().then(() => {
            seqnums[id] += chunks.length;
            for (const [, data] of chunks) {
              writers[id](data);
            }
          });
        } else if (message.shells) {
          shells = message.shells;
          if (movingOffsetDone) {
            movingOffset = [-1, 0, 0];
          }
          for (const [id] of message.shells) {
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

  // Stupid hack to preserve input focus when terminals are reordered.
  // See: https://github.com/sveltejs/svelte/issues/3973
  let activeElement: Element | null = null;

  beforeUpdate(() => {
    activeElement = document.activeElement;
  });

  afterUpdate(() => {
    if (activeElement instanceof HTMLElement) activeElement.focus();
  });
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

  <div class="absolute inset-0 flex justify-center items-center">
    {#each shells as [id, winsize] (id)}
      <div
        class="absolute"
        transition:fade|local
        use:slide={{
          x: winsize.x + (id === movingOffset[0] ? movingOffset[1] : 0),
          y: winsize.y + (id === movingOffset[0] ? movingOffset[2] : 0),
        }}
      >
        <XTerm
          rows={winsize.rows}
          cols={winsize.cols}
          bind:write={writers[id]}
          on:data={({ detail }) => srocket?.send({ data: [id, detail] })}
          on:move={({ detail }) => {
            movingOffsetDone = true;
            const newWinsize = {
              x: winsize.x + detail.x,
              y: winsize.y + detail.y,
              rows: winsize.rows,
              cols: winsize.cols,
            };
            srocket?.send({ move: [id, newWinsize] });
          }}
          on:moving={({ detail }) => {
            movingOffset = [id, detail.x, detail.y];
            movingOffsetDone = false;
          }}
          on:close={() => srocket?.send({ close: id })}
          on:focus={() => srocket?.send({ move: [id, null] })}
        />
      </div>
    {/each}
  </div>
</main>
