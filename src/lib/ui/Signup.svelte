<script lang="ts">
  import SignUp from "clerk-sveltekit/client/SignUp.svelte";
  export let id: string;
</script>

<div class="full-screen">
  <SignUp redirectUrl="/sign-in" />
</div>

<style>
  .full-screen {
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: center;
  }
</style>
