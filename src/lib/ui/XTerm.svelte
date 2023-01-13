<!-- @component Interactive terminal rendered with xterm.js -->
<script lang="ts" context="module">
  import { makeToast } from "$lib/toast";

  // Deduplicated terminal font loading.
  const waitForFonts = (() => {
    let state: "initial" | "loading" | "loaded" = "initial";
    const waitlist: (() => void)[] = [];

    return async function waitForFonts() {
      if (state === "loaded") return;
      else if (state === "initial") {
        const FontFaceObserver = (await import("fontfaceobserver")).default;
        state = "loading";
        try {
          await new FontFaceObserver("Fira Code VF").load();
        } catch (error) {
          makeToast({
            kind: "error",
            message: "Could not load terminal font.",
          });
        }
        state = "loaded";
        for (const fn of waitlist) fn();
      } else {
        await new Promise<void>((resolve) => {
          if (state === "loaded") resolve();
          else waitlist.push(resolve);
        });
      }
    };
  })();

  // Patch xterm to remove data requests triggering spurious messages when replayed.
  //
  // This removes support for several commands, which is not great for full feature support, but
  // without the patch the requests cause problems because they cause the terminal to send data
  // before any user interactions, so the data is duplicated with multiple connections.
  //
  // Search the xterm.js source for calls to "triggerDataEvent" to understand why these specific
  // functions were patched.
  //
  // I'm so sorry about this. In the future we should parse ANSI sequences correctly on the server
  // side and pass them through a state machine that filters such status requests and replies to
  // them exactly once, while being transparent to the sshx client.
  const patchXTerm = (() => {
    let patched = false;

    /* eslint-disable @typescript-eslint/no-empty-function */
    return function patchXTerm(term: any) {
      if (patched) return;
      patched = true;

      // Hack: This requires monkey-patching internal XTerm methods.
      const Terminal = term._core.constructor;
      const InputHandler = term._core._inputHandler.constructor;

      Terminal.prototype._handleColorEvent = () => {};
      Terminal.prototype._reportFocus = () => {};
      InputHandler.prototype.unhook = function () {
        this._data = new Uint32Array(0);
        return true;
      };
      InputHandler.prototype.sendDeviceAttributesPrimary = () => true;
      InputHandler.prototype.sendDeviceAttributesSecondary = () => true;
      InputHandler.prototype.requestMode = () => true;
      InputHandler.prototype.deviceStatus = () => true;
      InputHandler.prototype.deviceStatusPrivate = () => true;
      InputHandler.prototype.requestStatusString = () => true;
      const windowOptions = InputHandler.prototype.windowOptions;
      InputHandler.prototype.windowOptions = function (params: any): boolean {
        if (params.params[0] === 18) {
          return true; // GetWinSizeChars
        } else {
          return windowOptions.call(this, params);
        }
      };
    };
    /* eslint-enable @typescript-eslint/no-empty-function */
  })();
</script>

<script lang="ts">
  import { browser } from "$app/environment";

  import { createEventDispatcher, onDestroy, onMount } from "svelte";
  import type { Terminal } from "xterm";
  import { Buffer } from "buffer";

  import themes from "./themes";
  import CircleButton from "./CircleButton.svelte";
  import CircleButtons from "./CircleButtons.svelte";

  const theme = themes.defaultDark;

  /** Used to determine Cmd versus Ctrl keyboard shortcuts. */
  const isMac = browser && navigator.platform.startsWith("Mac");

  const dispatch = createEventDispatcher<{
    data: Uint8Array;
    close: void;
    bringToFront: void;
    startMove: MouseEvent;
    focus: void;
    blur: void;
  }>();

  export let rows: number, cols: number;
  export let write: (data: string) => void; // bound function prop

  export let termEl: HTMLDivElement = null as any; // suppress "missing prop" warning
  let term: Terminal | null = null;

  let loaded = false;
  let currentTitle = "Remote Terminal";

  const preloadBuffer: string[] = [];

  write = (data: string) => {
    if (!term) {
      // Before the terminal is loaded, push data into a buffer.
      preloadBuffer.push(data);
    } else {
      term.write(data);
    }
  };

  $: term?.resize(cols, rows);

  onMount(async () => {
    const { Terminal } = await import("xterm");
    const { WebLinksAddon } = await import("xterm-addon-web-links");
    const { WebglAddon } = await import("xterm-addon-webgl");

    await waitForFonts();

    term = new Terminal({
      allowTransparency: false,
      cursorBlink: false,
      cursorStyle: "block",
      // This is the monospace font family configured in Tailwind.
      fontFamily:
        '"Fira Code VF", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
      fontSize: 14,
      fontWeight: 400,
      fontWeightBold: 500,
      lineHeight: 1.06,
      scrollback: 5000,
      theme,
    });
    patchXTerm(term);

    // Keyboard shortcuts for natural text editing.
    term.attachCustomKeyEventHandler((event) => {
      if (
        (isMac && event.metaKey && !event.ctrlKey && !event.altKey) ||
        (!isMac && !event.metaKey && event.ctrlKey && !event.altKey)
      ) {
        if (event.key === "ArrowLeft") {
          dispatch("data", new Uint8Array([0x01]));
          return false;
        } else if (event.key === "ArrowRight") {
          dispatch("data", new Uint8Array([0x05]));
          return false;
        } else if (event.key === "Backspace") {
          dispatch("data", new Uint8Array([0x15]));
          return false;
        }
      }
      return true;
    });

    term.loadAddon(new WebLinksAddon());
    term.loadAddon(new WebglAddon());

    term.open(termEl);
    term.resize(cols, rows);
    term.onTitleChange((title) => {
      currentTitle = title;
    });

    let currentlyFocused = false;
    const focusObserver = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (
          mutation.type === "attributes" &&
          mutation.attributeName === "class"
        ) {
          // The "focus class is set directly by xterm.js, but there isn't any way to listen for it.
          const target = mutation.target as HTMLElement;
          const isFocused = target.classList.contains("focus");
          if (isFocused && !currentlyFocused) {
            currentlyFocused = isFocused;
            dispatch("focus");
          } else if (!isFocused && currentlyFocused) {
            currentlyFocused = isFocused;
            dispatch("blur");
          }
        }
      }
    });
    focusObserver.observe(term.element!, { attributeFilter: ["class"] });

    loaded = true;
    for (const data of preloadBuffer) {
      term.write(data);
    }

    const utf8 = new TextEncoder();
    term.onData((data: string) => {
      dispatch("data", utf8.encode(data));
    });
    term.onBinary((data: string) => {
      dispatch("data", Buffer.from(data, "binary"));
    });
  });

  onDestroy(() => term?.dispose());
</script>

<div
  class="term-container opacity-95"
  style:background={theme.background}
  on:mousedown={() => dispatch("bringToFront")}
>
  <div
    class="flex select-none"
    on:mousedown={(event) => dispatch("startMove", event)}
  >
    <div class="flex-1 flex items-center px-3">
      <CircleButtons>
        <CircleButton kind="red" on:click={() => dispatch("close")} />
        <CircleButton kind="yellow" />
        <CircleButton kind="green" />
      </CircleButtons>
    </div>
    <div
      class="p-2 text-sm text-gray-300 font-bold overflow-hidden text-ellipsis min-w-0"
    >
      {currentTitle}
    </div>
    <div class="flex-1" />
  </div>
  <div
    class="inline-block px-4 py-2 transition-opacity duration-500"
    bind:this={termEl}
    style:opacity={loaded ? 1.0 : 0.0}
  />
</div>

<style lang="postcss">
  .term-container {
    @apply inline-block rounded-lg border border-gray-600 transition-transform duration-200;
  }
</style>
