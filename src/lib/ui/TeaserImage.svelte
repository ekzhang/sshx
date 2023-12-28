<script lang="ts">
  import logo from "$lib/assets/logo.svg";
  import {
    ArrowLeftIcon,
    ArrowRightIcon,
    InfoIcon,
    MoreVerticalIcon,
    UserIcon,
    XIcon,
  } from "svelte-feather-icons";
</script>

<div class="rounded-lg border border-white/20 overflow-hidden">
  <div class="flex bg-zinc-900 items-end">
    <div class="px-4 py-3 flex gap-1.5">
      <div class="w-2.5 h-2.5 rounded-full bg-red-500" />
      <div class="w-2.5 h-2.5 rounded-full bg-yellow-500" />
      <div class="w-2.5 h-2.5 rounded-full bg-green-500" />
    </div>
    <div class="flex ml-2">
      <div class="rounded-t-lg bg-zinc-800 h-7 w-44 px-2 flex items-center">
        <img src={logo} alt="sshx logo" class="h-5 w-5" />
        <p class="ml-1.5 text-xs">sshx</p>
        <XIcon class="w-3.5 h-3.5 ml-auto text-zinc-400" />
      </div>
    </div>
  </div>
  <div class="flex py-1 bg-zinc-800 gap-2">
    <div class="flex px-2 py-1 gap-2">
      <ArrowLeftIcon class="w-5 h-5 text-zinc-500" />
      <ArrowRightIcon class="w-5 h-5 text-zinc-500" />
    </div>
    <div class="rounded-full flex-1 bg-zinc-900/60 flex items-center px-2">
      <InfoIcon class="w-4 h-4 text-zinc-400" />
      <p class="text-sm ml-2 select-none text-zinc-300">
        sshx.io/s/gzN0WHsm6r#tiOAVOLsNXEZxJ
      </p>
    </div>
    <div class="flex items-center gap-3 px-2">
      <UserIcon class="w-4 h-4 text-zinc-300" />
      <MoreVerticalIcon class="w-4 h-4 text-zinc-300" />
    </div>
  </div>
  <video muted autoplay loop>
    <source
      src="https://sshx.s3.amazonaws.com/media/teaser-video.mov"
      type="video/mp4"
    />
    <track kind="captions" />
  </video>
</div>
