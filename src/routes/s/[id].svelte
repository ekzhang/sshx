<script lang="ts">
  import { page } from "$app/stores";

  import Session from "$lib/Session.svelte";
</script>

<Session id={$page.params.id} />
