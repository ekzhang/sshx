<script lang="ts">
  import type { WsUser } from "$lib/protocol";

  export let user: WsUser;
</script>

<div class="flex items-center">
  <svg width="27" height="27" viewBox="0 0 27 27">
    <path d="M14 26L2 2L26 14H14V26Z" fill="#76FF84" stroke="#111111" />
  </svg>
  <p class="ml-1 bg-zinc-600 text-sm px-1.5 py-[1px] rounded-sm font-bold">
    {user.name}
  </p>
</div>
