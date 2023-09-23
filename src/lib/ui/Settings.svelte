<script lang="ts">
  import {
    Dialog,
    DialogDescription,
    DialogOverlay,
    DialogTitle,
    Transition,
    TransitionChild,
  } from "@rgossiaux/svelte-headlessui";
  import { XIcon } from "svelte-feather-icons";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{ close: void }>();

  export let open: boolean;
</script>

<Transition show={open}>
  <Dialog on:close class="fixed inset-0 z-50 grid place-items-center">
    <DialogOverlay class="fixed -z-10 inset-0 bg-black/20 backdrop-blur-sm" />

    <TransitionChild
      enter="duration-300 ease-out"
      enterFrom="scale-95 opacity-0"
      enterTo="scale-100 opacity-100"
      leave="duration-75 ease-out"
      leaveFrom="scale-200 opacity-100"
      leaveTo="scale-95 opacity-0"
      class="w-full sm:w-[calc(100%-32px)] max-w-screen-md"
    >
      <div
        class="relative bg-[#111] sm:border border-zinc-800 px-6 py-10 sm:py-6
         h-screen sm:h-auto max-h-screen sm:rounded-lg overflow-y-auto"
      >
        <button
          class="absolute top-4 right-4 p-1 rounded hover:bg-zinc-700 active:bg-indigo-700 transition-colors"
          aria-label="Close settings"
          on:click={() => dispatch("close")}
        >
          <XIcon class="h-5 w-5" />
        </button>

        <div class="mb-8 text-center">
          <DialogTitle class="text-xl font-medium mb-2">
            Terminal Settings
          </DialogTitle>
          <DialogDescription class="text-zinc-400">
            Customize your collaborative terminal.
          </DialogDescription>
        </div>

        <div class="flex flex-col gap-2">
          <div class="item">
            <div class="flex-1">
              <p class="font-medium mb-2">Name</p>
              <p class="text-sm text-zinc-400">
                How you appear to other users online.
              </p>
            </div>
            <div class="text-red-500">TODO</div>
          </div>
          <div class="item">
            <div class="flex-1">
              <p class="font-medium mb-2">Color palette</p>
              <p class="text-sm text-zinc-400">
                Color scheme for text in terminals.
              </p>
            </div>
            <div class="text-red-500">TODO</div>
          </div>
          <div class="item">
            <div class="flex-1">
              <p class="font-medium mb-2">Cursor style</p>
              <p class="text-sm text-zinc-400">
                How live cursors should be displayed.
              </p>
            </div>
            <div class="text-red-500">TODO</div>
          </div>
        </div>

        <!-- svelte-ignore missing-declaration -->
        <p class="mt-6 text-sm text-right text-zinc-400">
          <a
            target="_blank"
            rel="noreferrer"
            href="https://github.com/ekzhang/sshx"
            >sshx-server v{__APP_VERSION__}</a
          >
        </p>
      </div>
    </TransitionChild>
  </Dialog>
</Transition>

<style lang="postcss">
  .item {
    @apply bg-zinc-800/25 rounded-lg p-4 flex gap-4 flex-col sm:flex-row;
  }
</style>
