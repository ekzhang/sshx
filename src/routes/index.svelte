<script lang="ts">
  import Seo from "$lib/components/Seo.svelte";
</script>

<Seo
  title="sshx"
  ogTitle="sshx — share collaborative, live terminals on the web"
  description="Blazing-fast, collaborative live terminal in your browser, with real-time chat, cursors, and activity tracking."
/>

<main class="p-8 max-w-screen-sm">
  <h1 class="text-6xl">sshx</h1>
  <p><em>Site under construction.</em></p>

  <h2 class="text-2xl mt-8">Installation instructions</h2>

  <p>Run this command to install the CLI tool.</p>

  <pre>$ <code>cargo install sshx</code></pre>

  <p>
    You should be able to run the <code>sshx</code> command to start a new live,
    collaborative session from your computer. Invite others to join the link from
    their browser.
  </p>

  <p>
    Trying using for pair development, presentations, or debugging sessions.
    Have fun programming together!
  </p>

  <h2 class="text-2xl mt-8">Credits</h2>

  <p>
    Source is available on GitHub at <a href="https://github.com/ekzhang/sshx"
      >ekzhang/sshx</a
    >. This software was developed and designed by
    <a href="https://www.ekzhang.com/">Eric Zhang</a>.
  </p>
</main>

<style lang="postcss">
  p,
  pre {
    @apply my-4 text-gray-600;
  }

  pre {
    @apply p-2 bg-gray-100;
  }

  code {
    @apply text-black;
  }

  a {
    @apply text-blue-700 font-medium underline decoration-dotted hover:decoration-solid;
  }
</style>
