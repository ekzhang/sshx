<script lang="ts">
  import { onDestroy, onMount, tick, beforeUpdate, afterUpdate } from "svelte";
  import { fade } from "svelte/transition";
  import { debounce, throttle } from "lodash-es";

  import { Srocket } from "./srocket";
  import type { WsClient, WsServer, WsUser, WsWinsize } from "./protocol";
  import { makeToast } from "./toast";
  import Chat from "./ui/Chat.svelte";
  import Toolbar from "./ui/Toolbar.svelte";
  import XTerm from "./ui/XTerm.svelte";
  import Avatars from "./ui/Avatars.svelte";
  import LiveCursor from "./ui/LiveCursor.svelte";
  import { slide } from "./action/slide";
  import { TouchZoom } from "./action/touchZoom";

  export let id: string;

  // The magic numbers "left" and "top" are used to approximately center the
  // terminal at the time that it is first created.
  //
  // For a default 80x24 terminal, this is half of the width and height on a
  // normal screen at 100% scale.
  const CONSTANT_OFFSET_LEFT = 357;
  const CONSTANT_OFFSET_TOP = 258;

  const OFFSET_LEFT_CSS = `calc(50vw - ${CONSTANT_OFFSET_LEFT}px)`;
  const OFFSET_TOP_CSS = `calc(50vh - ${CONSTANT_OFFSET_TOP}px)`;
  const OFFSET_TRANSFORM_ORIGIN_CSS = `calc(-1 * ${OFFSET_LEFT_CSS}) calc(-1 * ${OFFSET_TOP_CSS})`;

  function getConstantOffset() {
    return [
      0.5 * window.innerWidth - CONSTANT_OFFSET_LEFT,
      0.5 * window.innerHeight - CONSTANT_OFFSET_TOP,
    ];
  }

  let fabricEl: HTMLElement;
  let touchZoom: TouchZoom;
  let center = [0, 0];
  let zoom = 1;

  onMount(() => {
    touchZoom = new TouchZoom(fabricEl);
    touchZoom.onMove(() => {
      center = touchZoom.center;
      zoom = touchZoom.zoom;
    });
  });

  /** Returns the mouse position in infinite grid coordinates, offset transformations and zoom. */
  function normalizePosition(event: MouseEvent): [number, number] {
    const [ox, oy] = getConstantOffset();
    return [
      Math.round(center[0] + event.pageX / zoom - ox),
      Math.round(center[1] + event.pageY / zoom - oy),
    ];
  }

  let srocket: Srocket<WsServer, WsClient> | null = null;

  let connected = false;
  let exitReason: string | null = null;

  let showChat = false; // @hmr:keep

  /** Bound "write" method for each terminal. */
  const writers: Record<number, (data: string) => void> = {};
  const termElements: Record<number, HTMLDivElement> = {};
  const seqnums: Record<number, number> = {};
  let userId = 0;
  let users: [number, WsUser][] = [];
  let shells: [number, WsWinsize][] = [];
  let subscriptions = new Set<number>();

  let moving = -1; // Terminal ID that is being dragged.
  let movingOrigin = [0, 0]; // Coordinates of mouse at origin when drag started.
  let movingSize: WsWinsize; // New [x, y] position of the dragged terminal.
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
            message: `Connected to the server.`,
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
          users = message.users;
        } else if (message.userDiff) {
          const [id, update] = message.userDiff;
          users = users.filter(([uid]) => uid !== id);
          if (update !== null) {
            users = [...users, [id, update]];
          }
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
          console.warn("Server error: " + message.error);
        }
      },

      onConnect() {
        connected = true;
      },

      onDisconnect() {
        connected = false;
        subscriptions.clear();
        users = [];
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
    // 50 milliseconds between successive terminal move updates.
    const sendMove = throttle((message: WsClient) => {
      srocket?.send(message);
    }, 50);

    // 80 milliseconds between successive cursor updates.
    const sendCursor = throttle((message: WsClient) => {
      srocket?.send(message);
    }, 80);

    function handleMouse(event: MouseEvent) {
      if (moving !== -1 && !movingIsDone) {
        const [x, y] = normalizePosition(event);
        movingSize = {
          ...movingSize,
          x: Math.round(x - movingOrigin[0]),
          y: Math.round(y - movingOrigin[1]),
        };
        sendMove({ move: [moving, movingSize] });
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

      sendCursor({ setCursor: normalizePosition(event) });
    }

    function handleMouseEnd(event: MouseEvent) {
      if (moving !== -1) {
        movingIsDone = true;
        sendMove.cancel();
        srocket?.send({ move: [moving, movingSize] });
      }

      if (resizing !== -1) {
        resizing = -1;
      }

      if (event.type === "mouseleave") {
        sendCursor.cancel();
        srocket?.send({ setCursor: null });
      }
    }

    window.addEventListener("mousemove", handleMouse);
    window.addEventListener("mouseup", handleMouseEnd);
    document.body.addEventListener("mouseleave", handleMouseEnd);
    return () => {
      window.removeEventListener("mousemove", handleMouse);
      window.removeEventListener("mouseup", handleMouseEnd);
      document.body.removeEventListener("mouseleave", handleMouseEnd);
    };
  });

  let focused: number[] = [];
  $: setFocus(focused);

  // Wait a small amount of time, since blur events happen before focus events.
  const setFocus = debounce((focused: number[]) => {
    srocket?.send({ setFocus: focused[0] ?? null });
  }, 20);
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

  <div class="absolute inset-0 overflow-hidden touch-none" bind:this={fabricEl}>
    {#each shells as [id, winsize] (id)}
      {@const ws = id === moving ? movingSize : winsize}
      <div
        class="absolute"
        style:left={OFFSET_LEFT_CSS}
        style:top={OFFSET_TOP_CSS}
        style:transform-origin={OFFSET_TRANSFORM_ORIGIN_CSS}
        transition:fade|local
        use:slide={{ x: ws.x, y: ws.y, center, zoom }}
      >
        <XTerm
          rows={ws.rows}
          cols={ws.cols}
          bind:write={writers[id]}
          bind:termEl={termElements[id]}
          on:data={({ detail: data }) => srocket?.send({ data: [id, data] })}
          on:close={() => srocket?.send({ close: id })}
          on:bringToFront={() => srocket?.send({ move: [id, null] })}
          on:startMove={({ detail: event }) => {
            const [x, y] = normalizePosition(event);
            moving = id;
            movingOrigin = [x - ws.x, y - ws.y];
            movingSize = ws;
            movingIsDone = false;
          }}
          on:focus={() => (focused = [...focused, id])}
          on:blur={() => (focused = focused.filter((i) => i !== id))}
        />

        <!-- User avatars -->
        <div class="absolute bottom-2.5 right-2.5 pointer-events-none">
          <Avatars
            users={users.filter(
              ([uid, user]) => uid !== userId && user.focus === id,
            )}
          />
        </div>

        <!-- Interactable element for resizing -->
        <div
          class="absolute w-5 h-5 -bottom-1 -right-1 cursor-nwse-resize"
          on:mousedown={(event) => {
            const canvasEl = termElements[id].querySelector(
              "canvas.xterm-cursor-layer",
            );
            if (canvasEl) {
              resizing = id;
              const r = canvasEl.getBoundingClientRect();
              resizingOrigin = [event.pageX - r.width, event.pageY - r.height];
              resizingCell = [r.width / ws.cols, r.height / ws.rows];
              resizingSize = ws;
            }
          }}
          on:pointerdown={(event) => event.stopPropagation()}
        />
      </div>
    {/each}

    {#each users.filter(([id, user]) => id !== userId && user.cursor !== null) as [id, user] (id)}
      <div
        class="absolute"
        style:left={OFFSET_LEFT_CSS}
        style:top={OFFSET_TOP_CSS}
        style:transform-origin={OFFSET_TRANSFORM_ORIGIN_CSS}
        transition:fade|local={{ duration: 200 }}
        use:slide={{
          x: user.cursor?.[0] ?? 0,
          y: user.cursor?.[1] ?? 0,
          center,
          zoom,
        }}
      >
        <LiveCursor {user} />
      </div>
    {/each}
  </div>
</main>
