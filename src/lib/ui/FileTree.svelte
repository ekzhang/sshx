<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { FileIcon, FolderIcon, ChevronRightIcon } from "svelte-feather-icons";
  import type { FileEntry } from "../protocol";

  export let entries: FileEntry[] = [];
  export let depth: number = 0;
  export let basePath: string = "";
  export let selectedPath: string | null = null;
  export let expandedDirs: Set<string> = new Set();

  const dispatch = createEventDispatcher<{
    select: string;
    toggle: string;
    doubleClick: string;
  }>();

  function getFullPath(name: string): string {
    if (basePath === "/") return `/${name}`;
    return basePath ? `${basePath}/${name}` : `/${name}`;
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return "";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB`;
  }
</script>

{#each entries as entry (entry.name)}
  {@const fullPath = getFullPath(entry.name)}
  <div
    class="file-entry"
    class:selected={fullPath === selectedPath}
    style="padding-left: {depth * 16 + 8}px"
    on:click={() => entry.isDir ? dispatch("toggle", fullPath) : dispatch("select", fullPath)}
    on:dblclick={() => !entry.isDir && dispatch("doubleClick", fullPath)}
    role="treeitem"
    aria-selected={fullPath === selectedPath}
    tabindex="0"
    on:keydown={(e) => {
      if (e.key === 'Enter') {
        entry.isDir ? dispatch("toggle", fullPath) : dispatch("select", fullPath);
      }
    }}
  >
    <span class="icon">
      {#if entry.isDir}
        <span class="chevron" class:rotated={expandedDirs.has(fullPath)}>
          <ChevronRightIcon size="14" />
        </span>
        <FolderIcon size="14" />
      {:else}
        <FileIcon size="14" />
      {/if}
    </span>
    <span class="name">{entry.name}</span>
    {#if !entry.isDir}
      <span class="size">{formatSize(entry.size)}</span>
    {/if}
  </div>
{/each}

<style lang="postcss">
  .file-entry {
    display: flex;
    align-items: center;
    gap: 4px;
    padding-top: 2px;
    padding-bottom: 2px;
    padding-right: 8px;
    cursor: pointer;
    user-select: none;
    font-size: 13px;
    color: #d1d5db;
  }
  .file-entry:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .file-entry.selected {
    background: rgba(99, 102, 241, 0.2);
  }
  .icon {
    display: flex;
    align-items: center;
    gap: 1px;
    flex-shrink: 0;
    width: 32px;
    color: #9ca3af;
  }
  .chevron {
    transition: transform 100ms;
  }
  .chevron.rotated {
    transform: rotate(90deg);
  }
  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .size {
    flex-shrink: 0;
    font-size: 11px;
    color: #6b7280;
  }
</style>
