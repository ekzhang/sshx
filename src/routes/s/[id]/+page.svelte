<script lang="ts">
  import { page } from "$app/stores";
  import SignedOut from "clerk-sveltekit/client/SignedOut.svelte";
  import SignedIn from "clerk-sveltekit/client/SignedIn.svelte";

  import Session from "$lib/Session.svelte";
  import Login from "$lib/ui/Login.svelte";
</script>

<svelte:head>
  <style>
    body {
      overscroll-behavior: none;
    }
  </style>
</svelte:head>

<SignedIn>
  <Session id={$page.params.id} />
</SignedIn>
<SignedOut>
  <Login id={$page.params.id} />
</SignedOut>
