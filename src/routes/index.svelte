<script lang="ts" context="module">
  export const prerender = true;
</script>

<script lang="ts">
  import logotypeDark from "$lib/assets/logotype-dark.svg";
  import landingGraphic from "$lib/assets/landing-graphic.svg";
  import landingBackground from "$lib/assets/landing-background.svg";
</script>

<main class="max-w-screen-xl mx-auto px-16">
  <header class="my-12">
    <img class="h-16 -mx-1" src={logotypeDark} alt="sshx logo" />
  </header>
  <h1 class="font-extrabold text-5xl max-w-[26ch] leading-[1.15] py-2">
    Web-based, <span class="title-gradient">real-time collaboration</span> for your
    remote terminal
  </h1>

  <div class="relative">
    <div class="absolute left-[180px] top-[-200px] -z-10">
      <img class="select-none" src={landingBackground} alt="" />
    </div>
    <div class="absolute left-[500px] w-[1000px]">
      <img
        class="mt-5 mb-8 w-[720px]"
        width={813}
        height={623}
        src={landingGraphic}
        alt="two terminal windows running sshx and three live cursors"
      />
    </div>
  </div>

  <section class="my-12 space-y-6 text-xl max-w-[460px] text-gray-200">
    <p>
      <code>sshx</code> lets you share your terminal by link with anyone on the web.
    </p>
    <p>
      It supports <b>live presence</b> and <b>chat</b> so you can see who’s
      online and seamlessly work with them. Plus, it’s <b>fast</b>: just run a
      single Rust-based CLI tool to start your session.
    </p>
    <p>
      Use <code>sshx</code> for pair programming, demos, remote access, or even just
      as a fashionable web interface for your terminal.
    </p>
  </section>

  <button
    class="bg-pink-600 hover:bg-pink-500 active:ring-4 active:ring-pink-500/50 text-lg font-semibold px-8 py-2 rounded-full my-2"
    on:click={() => alert("this site is still under construction!")}
  >
    Get Started
  </button>
</main>

<style lang="postcss">
  .title-gradient {
    @apply text-transparent bg-clip-text bg-gradient-to-r from-fuchsia-400 to-blue-500;
  }

  code {
    @apply text-[0.9em] text-white font-medium bg-gray-600 px-1 py-0.5 rounded;
  }
</style>
