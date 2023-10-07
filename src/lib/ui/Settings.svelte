<script lang="ts">
  import OverlayMenu from "./OverlayMenu.svelte";

  export let open: boolean;
</script>

<OverlayMenu
  title="Terminal Settings"
  description="Customize your collaborative terminal."
  showCloseButton
  {open}
  on:close
>
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
        <p class="text-sm text-zinc-400">Color scheme for text in terminals.</p>
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
    <a target="_blank" rel="noreferrer" href="https://github.com/ekzhang/sshx"
      >sshx-server v{__APP_VERSION__}</a
    >
  </p>
</OverlayMenu>

<style lang="postcss">
  .item {
    @apply bg-zinc-800/25 rounded-lg p-4 flex gap-4 flex-col sm:flex-row;
  }
</style>
