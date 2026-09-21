<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import {
    XIcon,
    UploadIcon,
    DownloadIcon,
    Edit3Icon,
    Trash2Icon,
    HomeIcon,
    RefreshCwIcon,
  } from "svelte-feather-icons";
  import FileTree from "./FileTree.svelte";
  import type { FileEntry } from "../protocol";

  export let open: boolean = false;
  export let entries: FileEntry[] = [];
  export let entriesMap: Record<string, FileEntry[]> = { "/": [] };
  export let currentPath: string = "/";
  export let sessionId: string = "";
  export let encryptedZerosB64: string = "";

  let expandedDirs: Set<string> = new Set();
  let selectedPath: string | null = null;
  let fileInput: HTMLInputElement;
  let panelWidth = 300;
  let resizing = false;
  let dragOver = false;
  let dragCounter = 0;

  const MIN_WIDTH = 200;
  const MAX_WIDTH = 600;

  const dispatch = createEventDispatcher<{
    close: void;
    listFiles: string;
    deleteFile: string;
    renameFile: { path: string; newName: string };
  }>();

  function joinPath(parent: string, name: string): string {
    return parent === "/" ? `/${name}` : `${parent}/${name}`;
  }

  $: selectedEntry = entries.find(
    (e) => joinPath(currentPath, e.name) === selectedPath,
  );

  function goRoot() {
    currentPath = "/";
    dispatch("listFiles", "/");
  }

  function refresh() {
    dispatch("listFiles", currentPath);
  }

  function handleDownload() {
    if (!selectedPath) return;
    const url = `/api/s/${sessionId}/files?path=${encodeURIComponent(selectedPath)}`;
    fetch(url, { headers: { "X-SSHX-Key": encryptedZerosB64 } })
      .then((resp) => {
        if (!resp.ok) throw new Error(`Download failed: ${resp.status}`);
        return resp.blob();
      })
      .then((blob) => {
        const filename = selectedPath!.split("/").pop() || "download";
        const a = document.createElement("a");
        a.href = URL.createObjectURL(blob);
        a.download = filename;
        a.click();
        URL.revokeObjectURL(a.href);
      })
      .catch((err) => console.error("Download failed:", err));
  }

  function handleUpload() {
    fileInput?.click();
  }

  async function uploadFile(file: File) {
    const uploadPath = joinPath(currentPath, file.name);
    try {
      const resp = await fetch(
        `/api/s/${sessionId}/files?path=${encodeURIComponent(uploadPath)}`,
        {
          method: "POST",
          headers: { "X-SSHX-Key": encryptedZerosB64 },
          body: await file.arrayBuffer(),
        },
      );
      if (!resp.ok) console.error(`Upload ${file.name} failed: ${resp.status}`);
      return resp.ok;
    } catch (err) {
      console.error("Upload failed:", err);
      return false;
    }
  }

  async function handleFileSelected(event: Event) {
    const input = event.target as HTMLInputElement;
    const files = input.files;
    if (!files || files.length === 0) return;
    for (const file of Array.from(files)) {
      await uploadFile(file);
    }
    dispatch("listFiles", currentPath);
    input.value = "";
  }

  function onDragEnter(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    dragCounter++;
    if (event.dataTransfer?.types.includes("Files")) {
      dragOver = true;
    }
  }

  function onDragOver(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
  }

  function onDragLeave(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    dragCounter--;
    if (dragCounter === 0) {
      dragOver = false;
    }
  }

  async function onDrop(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    dragOver = false;
    dragCounter = 0;
    const files = event.dataTransfer?.files;
    if (!files || files.length === 0) return;
    for (const file of Array.from(files)) {
      await uploadFile(file);
    }
    dispatch("listFiles", currentPath);
  }

  function handleDelete() {
    if (!selectedPath) return;
    if (confirm(`Delete ${selectedPath}?`)) {
      dispatch("deleteFile", selectedPath);
    }
  }

  function handleRename() {
    if (!selectedPath) return;
    const name = selectedPath.split("/").pop() || "";
    const newName = prompt("New name:", name);
    if (newName && newName !== name) {
      const parent =
        selectedPath.substring(0, selectedPath.lastIndexOf("/")) || "/";
      dispatch("renameFile", {
        path: selectedPath,
        newName: `${parent}/${newName}`,
      });
    }
  }

  function goUp() {
    const parent =
      currentPath.substring(0, currentPath.lastIndexOf("/")) || "/";
    currentPath = parent;
    dispatch("listFiles", parent);
  }

  function handleToggle(path: string) {
    const newExpanded = new Set(expandedDirs);
    if (newExpanded.has(path)) {
      newExpanded.delete(path);
    } else {
      newExpanded.add(path);
    }
    expandedDirs = newExpanded;
    dispatch("listFiles", path);
  }

  function startResize(event: PointerEvent) {
    resizing = true;
    (event.target as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onResize(event: PointerEvent) {
    if (!resizing) return;
    const newWidth = window.innerWidth - event.clientX;
    panelWidth = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, Math.round(newWidth)));
  }

  function stopResize() {
    resizing = false;
  }
</script>

{#if open}
  <div
    class="file-manager"
    style="width: {panelWidth}px"
    class:resizing
    class:drag-over={dragOver}
    on:dragenter={onDragEnter}
    on:dragover={onDragOver}
    on:dragleave={onDragLeave}
    on:drop={onDrop}
  >
    <div
      class="resize-handle"
      on:pointerdown={startResize}
      on:pointermove={onResize}
      on:pointerup={stopResize}
      on:pointercancel={stopResize}
      role="separator"
      aria-label="Resize file manager"
    ></div>

    <div class="header">
      <span class="title">Files</span>
      <button class="close-btn" on:click={() => dispatch("close")}>
        <XIcon size="16" />
      </button>
    </div>

    <div class="breadcrumb">
      <button class="nav-btn" on:click={goRoot} title="Root directory">
        <HomeIcon size="13" />
      </button>
      <button
        class="nav-btn"
        on:click={goUp}
        disabled={currentPath === "/"}
        title="Parent directory"
      >
        &uarr;
      </button>
      <button class="nav-btn" on:click={refresh} title="Refresh">
        <RefreshCwIcon size="13" />
      </button>
      <span class="path">{currentPath}</span>
    </div>

    <div class="upload-area">
      <button class="upload-btn" on:click={handleUpload}>
        <UploadIcon size="14" />
        Upload
      </button>
      <span class="hint">or drop files here</span>
      <input
        type="file"
        bind:this={fileInput}
        class="hidden"
        on:change={handleFileSelected}
      />
    </div>

    <div class="tree-container">
      {#if entries.length === 0}
        <p class="empty">Empty directory</p>
      {:else}
        <FileTree
          {entries}
          depth={0}
          basePath={currentPath}
          {selectedPath}
          {expandedDirs}
          on:select={(e) => (selectedPath = e.detail)}
          on:toggle={(e) => handleToggle(e.detail)}
          on:doubleClick={() => handleDownload()}
        />
      {/if}
    </div>

    {#if dragOver}
      <div class="drop-overlay">
        <UploadIcon size="32" />
        <p>Drop files to upload</p>
      </div>
    {/if}

    {#if selectedPath}
      <div class="actions">
        <button on:click={handleDownload} title="Download">
          <DownloadIcon size="14" />
          Download
        </button>
        <button on:click={handleRename} title="Rename">
          <Edit3Icon size="14" />
          Rename
        </button>
        <button on:click={handleDelete} title="Delete">
          <Trash2Icon size="14" />
          Delete
        </button>
      </div>
    {/if}
  </div>
{/if}

<style lang="postcss">
  .file-manager {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    background: #18181b;
    border-left: 1px solid #27272a;
    display: flex;
    flex-direction: column;
    z-index: 20;
  }
  .file-manager.drag-over {
    border-left-color: #3b82f6;
  }
  .file-manager.resizing {
    user-select: none;
  }
  .drop-overlay {
    position: absolute;
    inset: 0;
    background: rgba(59, 130, 246, 0.15);
    border: 2px dashed #3b82f6;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: #93c5fd;
    pointer-events: none;
    z-index: 40;
  }
  .drop-overlay p {
    font-size: 14px;
    font-weight: 500;
  }
  .resize-handle {
    position: absolute;
    left: -3px;
    top: 0;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    z-index: 30;
  }
  .resize-handle:hover,
  .file-manager.resizing .resize-handle {
    background: rgba(59, 130, 246, 0.5);
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid #27272a;
  }
  .title {
    font-weight: 500;
    font-size: 14px;
  }
  .close-btn {
    padding: 2px;
    border-radius: 4px;
    color: #a1a1aa;
    border: none;
    background: none;
    cursor: pointer;
  }
  .close-btn:hover {
    background: #27272a;
    color: #fff;
  }
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    font-size: 12px;
    color: #a1a1aa;
    border-bottom: 1px solid #27272a;
    overflow: hidden;
  }
  .nav-btn {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 4px;
    font-size: 14px;
    border: none;
    background: none;
    color: #a1a1aa;
    cursor: pointer;
    padding: 0;
  }
  .nav-btn:hover:not(:disabled) {
    background: #27272a;
    color: #fff;
  }
  .nav-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }
  .path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-left: 4px;
  }
  .upload-area {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-bottom: 1px solid #27272a;
  }
  .upload-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    border-radius: 4px;
    background: #3b82f6;
    color: white;
    font-size: 12px;
    font-weight: 500;
    border: none;
    cursor: pointer;
  }
  .upload-btn:hover {
    background: #2563eb;
  }
  .hint {
    font-size: 11px;
    color: #71717a;
  }
  .hidden {
    display: none;
  }
  .tree-container {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }
  .empty {
    padding: 16px;
    text-align: center;
    font-size: 13px;
    color: #71717a;
  }
  .actions {
    display: flex;
    gap: 4px;
    padding: 8px 12px;
    border-top: 1px solid #27272a;
  }
  .actions button {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 12px;
    color: #a1a1aa;
    border: none;
    background: none;
    cursor: pointer;
  }
  .actions button:hover {
    background: #27272a;
    color: #fff;
  }
</style>
