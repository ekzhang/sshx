<script lang="ts">
  import { page } from "$app/stores";

  import Session from "$lib/Session.svelte";
</script>

<svelte:head>
  <style>
    body {
      overscroll-behavior: none;
    }
  </style>
</svelte:head>

<Session id={$page.params.id} />
