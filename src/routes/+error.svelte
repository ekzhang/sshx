<script lang="ts">
  import { page } from "$app/stores";

  import logotypeDark from "$lib/assets/logotype-dark.svg";
</script>

<main class="p-4 max-w-xl mx-auto my-6 md:my-12 lg:my-24">
  <img class="h-16 -mx-2" src={logotypeDark} alt="sshx logo" />

  <div class="space-y-4 mt-6 mb-8 text-zinc-300">
    <p>
      {#if $page.status === 404}
        <b class="text-white">404 Not Found.</b> We couldn't find this page, sorry!
      {:else}
        <b class="text-white">Error {$page.status}.</b> An unexpected error occurred.
      {/if}
    </p>
    {#if $page.status !== 404}
      <pre class="whitespace-pre-wrap break-all p-3 rounded bg-zinc-900">
{JSON.stringify($page.error, null, 2)}
</pre>
    {/if}
    <p>
      Perhaps try coming back later? If you have any feedback, please feel free
      to reach out at
      <a class="underline text-white" href="mailto:ekzhang1@gmail.com"
        >ekzhang1@gmail.com</a
      >.
    </p>
  </div>

  <a
    href="/"
    class="inline-block font-semibold px-6 py-2 rounded-full bg-indigo-900 hover:bg-indigo-700"
    >Return home</a
  >
</main>
