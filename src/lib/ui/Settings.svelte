<script lang="ts">
  import { settings } from "$lib/settings";
  import { ChevronDownIcon } from "svelte-feather-icons";
  import OverlayMenu from "./OverlayMenu.svelte";
  import themes, { defaultTheme, type ThemeName } from "./themes";

  export let open: boolean;

  let nameValue = "";
  let initialized = false;

  $: open, (initialized = false);
  $: if (open && !initialized) {
    initialized = true;
    nameValue = $settings.name;
  }

  let selectedTheme: ThemeName; // Bound to the settings input.
  if (Object.hasOwn(themes, $settings.theme)) {
    selectedTheme = $settings.theme;
  } else {
    selectedTheme = defaultTheme;
  }

  function handleThemeChange() {
    settings.update((curSettings) => ({
      ...curSettings,
      theme: selectedTheme,
    }));
  }
</script>

<OverlayMenu
  title="Terminal Settings"
  description="Customize your collaborative terminal."
  showCloseButton
  {open}
  on:close
>
  <div class="flex flex-col gap-4">
    <div class="item">
      <div class="flex-1">
        <p class="font-medium mb-2">Name</p>
        <p class="text-sm text-zinc-400">
          Choose how you appear to other users.
        </p>
      </div>
      <div>
        <input
          class="input-common"
          placeholder="Your name"
          bind:value={nameValue}
          maxlength="50"
          on:input={() => {
            if (nameValue.length >= 2) {
              settings.update((curSettings) => ({
                ...curSettings,
                name: nameValue,
              }));
            }
          }}
        />
      </div>
    </div>
    <div class="item">
      <div class="flex-1">
        <p class="font-medium mb-2">Color palette</p>
        <p class="text-sm text-zinc-400">Color theme for text in terminals.</p>
      </div>
      <div class="relative">
        <ChevronDownIcon
          class="absolute top-[11px] right-2.5 w-4 h-4 text-zinc-400"
        />
        <select
          class="input-common !pr-5"
          bind:value={selectedTheme}
          on:change={handleThemeChange}
        >
          {#each Object.keys(themes) as themeName (themeName)}
            <option value={themeName}>{themeName}</option>
          {/each}
        </select>
      </div>
    </div>
    <!-- <div class="item">
      <div class="flex-1">
        <p class="font-medium mb-2">Cursor style</p>
        <p class="text-sm text-zinc-400">Style of live cursors.</p>
      </div>
      <div class="text-red-500">Coming soon</div>
    </div> -->
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
    @apply bg-zinc-800/25 rounded-lg p-4 flex gap-4 flex-col sm:flex-row items-start;
  }

  .input-common {
    @apply w-52 px-3 py-2 text-sm rounded-md bg-transparent hover:bg-white/5;
    @apply border border-zinc-700 outline-none focus:ring-2 focus:ring-indigo-500/50;
    @apply appearance-none transition-colors;
  }
</style>
