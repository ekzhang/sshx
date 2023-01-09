<script lang="ts">
  import { onDestroy, onMount, tick, beforeUpdate, afterUpdate } from "svelte";
  import { fade } from "svelte/transition";

  import { Srocket } from "./srocket";
  import type { WsClient, WsServer, WsWinsize } from "./protocol";
  import Chat from "./ui/Chat.svelte";
  import Toolbar from "./ui/Toolbar.svelte";
  import XTerm from "./ui/XTerm.svelte";
  import { slide } from "./ui/slide";
  import { makeToast } from "./toast";

  export let id: string;

  let srocket: Srocket<WsServer, WsClient> | null = null;

  let connected = false;
  let exitReason: string | null = null;

  let showChat = false; // @hmr:keep

  /** Bound "write" method for each terminal. */
  const writers: Record<number, (data: string) => void> = {};
  const termElements: Record<number, HTMLDivElement> = {};
  const seqnums: Record<number, number> = {};
  let userId = 0;
  let shells: [number, WsWinsize][] = [];
  let subscriptions = new Set<number>();

  let moving = -1; // Terminal ID that is being dragged.
  let movingOrigin = [0, 0]; // Coordinates of mouse at origin when drag started.
  let movingSize: WsWinsize; // New [x, y] position of the dragged terminal.
  let movingLast = 0; // Time when the last move message was sent.
  let movingIsDone = false; // Moving finished but hasn't been acknowledged.

  let resizing = -1; // Terminal ID that is being resized.
  let resizingOrigin = [0, 0]; // Coordinates of top-left origin when resize started.
  let resizingCell = [0, 0]; // Pixel dimensions of a single terminal cell.
  let resizingSize: WsWinsize; // Last resize message sent.

  onMount(() => {
    srocket = new Srocket<WsServer, WsClient>(`/api/s/${id}`, {
      onMessage(message) {
        if (message.hello) {
          userId = message.hello;
          makeToast({
            kind: "success",
            message: `Connected to the server as user ${userId}.`,
          });
        } else if (message.chunks) {
          const [id, chunks] = message.chunks;
          tick().then(() => {
            seqnums[id] += chunks.length;
            for (const [, data] of chunks) {
              writers[id](data);
            }
          });
        } else if (message.users) {
          console.log("users", message.users);
        } else if (message.userDiff) {
          console.log("userDiff", message.userDiff);
        } else if (message.shells) {
          shells = message.shells;
          if (movingIsDone) {
            moving = -1;
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
          makeToast({
            kind: "error",
            message: "Server error: " + message.error,
          });
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

  // Global mouse handler logic follows, attached to the window element for smoothness.
  onMount(() => {
    function handleDrag(event: MouseEvent) {
      if (moving !== -1 && !movingIsDone) {
        movingSize = {
          ...movingSize,
          x: event.pageX - movingOrigin[0],
          y: event.pageY - movingOrigin[1],
        };
        const now = Date.now();
        if (now >= movingLast + 40) {
          // 40 milliseconds between successive updates.
          movingLast = now;
          srocket?.send({ move: [moving, movingSize] });
        }
      }
      if (resizing !== -1) {
        const cols = Math.max(
          Math.floor((event.pageX - resizingOrigin[0]) / resizingCell[0]),
          60, // Minimum number of columns.
        );
        const rows = Math.max(
          Math.floor((event.pageY - resizingOrigin[1]) / resizingCell[1]),
          8, // Minimum number of rows.
        );
        if (rows !== resizingSize.rows || cols !== resizingSize.cols) {
          resizingSize = { ...resizingSize, rows, cols };
          srocket?.send({ move: [resizing, resizingSize] });
        }
      }
    }
    function handleDragEnd(event: MouseEvent) {
      if (moving !== -1) {
        movingIsDone = true;
        srocket?.send({ move: [moving, movingSize] });
      }
      if (resizing !== -1) {
        resizing = -1;
      }
    }
    window.addEventListener("mousemove", handleDrag);
    window.addEventListener("mouseup", handleDragEnd);
    window.addEventListener("mouseleave", handleDragEnd);
    return () => {
      window.removeEventListener("mousemove", handleDrag);
      window.removeEventListener("mouseup", handleDragEnd);
      window.removeEventListener("mouseleave", handleDragEnd);
    };
  });
</script>

<main class="p-8" class:cursor-nwse-resize={resizing !== -1}>
  <div
    class="absolute top-8 left-1/2 -translate-x-1/2 pointer-events-none z-10"
  >
    <Toolbar
      {connected}
      on:create={() => srocket?.send({ create: [] })}
      on:chat={() => (showChat = !showChat)}
    />
  </div>

  {#if showChat}
    <div
      class="absolute flex flex-col justify-end inset-y-8 right-8 w-80 pointer-events-none z-10"
    >
      <Chat on:close={() => (showChat = false)} />
    </div>
  {/if}

  <div class="py-2">
    {#if exitReason !== null}
      <div class="text-red-400">{exitReason}</div>
    {:else if connected}
      <div class="text-green-400">You are connected!</div>
    {:else}
      <div class="text-yellow-400">Connecting…</div>
    {/if}
  </div>

  <div class="absolute inset-0 overflow-hidden">
    {#each shells as [id, winsize] (id)}
      {@const ws = id === moving ? movingSize : winsize}
      <!--
        The magic numbers "left" and "top" are used to approximately center the
        terminal at the time that it is first created.

        For a default 80x24 terminal, this is half of the width and height on a
        normal screen at 100% scale.
      -->
      <div
        class="absolute left-[calc(50vw-357px)] top-[calc(50vh-258px)]"
        transition:fade|local
        use:slide={{ x: ws.x, y: ws.y }}
      >
        <XTerm
          rows={ws.rows}
          cols={ws.cols}
          bind:write={writers[id]}
          bind:termEl={termElements[id]}
          on:data={({ detail: data }) => srocket?.send({ data: [id, data] })}
          on:startMove={({ detail: event }) => {
            moving = id;
            movingOrigin = [event.pageX - ws.x, event.pageY - ws.y];
            movingSize = ws;
            movingIsDone = false;
          }}
          on:close={() => srocket?.send({ close: id })}
          on:focus={() => srocket?.send({ move: [id, null] })}
        />
        <div
          class="absolute w-5 h-5 -bottom-1 -right-1 cursor-nwse-resize"
          on:mousedown={(event) => {
            const canvasEl = termElements[id].querySelector(
              "canvas.xterm-text-layer",
            );
            if (canvasEl) {
              resizing = id;
              const r = canvasEl.getBoundingClientRect();
              resizingOrigin = [event.pageX - r.width, event.pageY - r.height];
              resizingCell = [r.width / ws.cols, r.height / ws.rows];
              resizingSize = ws;
            }
          }}
        />
      </div>
    {/each}
  </div>
</main>
