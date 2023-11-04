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

  export let title: string;
  export let description: string;
  export let showCloseButton = false;
  export let maxWidth: number = 768; // screen-md
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
      class="w-full sm:w-[calc(100%-32px)]"
      style="max-width: {maxWidth}px"
    >
      <div
        class="relative bg-[#111] sm:border border-zinc-800 px-6 py-10 sm:py-6
         h-screen sm:h-auto max-h-screen sm:rounded-lg overflow-y-auto"
      >
        {#if showCloseButton}
          <button
            class="absolute top-4 right-4 p-1 rounded hover:bg-zinc-700 active:bg-indigo-700 transition-colors"
            aria-label="Close {title}"
            on:click={() => dispatch("close")}
          >
            <XIcon class="h-5 w-5" />
          </button>
        {/if}

        <div class="mb-8 text-center">
          <DialogTitle class="text-xl font-medium mb-2">
            {title}
          </DialogTitle>
          <DialogDescription class="text-zinc-400">
            {description}
          </DialogDescription>
        </div>

        <slot />
      </div>
    </TransitionChild>
  </Dialog>
</Transition>
