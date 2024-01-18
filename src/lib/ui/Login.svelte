<script lang="ts">
  import SignIn from "clerk-sveltekit/client/SignIn.svelte";
  export let id: string;
</script>

<div class="full-screen">
  <SignIn redirectUrl="/s/{id}" />
</div>

<style>
  .full-screen {
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
  }
</style>
