<script lang="ts">
  import logotypeDark from "$lib/assets/logotype-dark.svg";
  import landingGraphic from "$lib/assets/landing-graphic.svg";
  import landingBackground from "$lib/assets/landing-background.svg";

  let installationEl: HTMLDivElement;

  const installs = [
    {
      title: "macOS / Linux",
      steps: `curl -sSf https://sshx.io/get | sh`,
    },
    {
      title: "Rust (from source)",
      steps: `cargo install sshx`,
    },
  ];

  const socials = [
    {
      title: "🤖\xa0 GitHub",
      href: "https://github.com/ekzhang/sshx",
    },
    {
      title: "🌸\xa0 Twitter",
      href: "https://twitter.com/ekzhang1",
    },
    {
      title: "💌\xa0 Email",
      href: "mailto:ekzhang1@gmail.com",
    },
    {
      title: "🌎\xa0 Website",
      href: "https://www.ekzhang.com",
    },
  ];

  let selectedInstall = installs[0];
</script>

<div class="overflow-x-hidden absolute inset-0">
  <div class="w-full bg-pink-700 text-white text-xl px-4 py-8 text-center">
    <strong>Warning:</strong> sshx is not ready or publicly released yet. Use at
    your own risk.
  </div>

  <main class="max-w-screen-xl mx-auto px-4 md:px-8 lg:px-16">
    <header class="my-12">
      <img class="h-16 -mx-1" src={logotypeDark} alt="sshx logo" />
    </header>
    <h1>
      A secure web-based,
      <span class="title-gradient">collaborative</span> terminal.
    </h1>

    <div class="relative">
      <div
        class="absolute scale-150 md:scale-100 md:left-[180px] md:top-[-200px] md:w-[1000px] -z-10"
      >
        <img class="select-none" src={landingBackground} alt="" />
      </div>
      <div class="md:absolute md:left-[500px] md:w-[1000px]">
        <img
          class="mt-5 mb-8 w-[720px]"
          width={813}
          height={623}
          src={landingGraphic}
          alt="two terminal windows running sshx and three live cursors"
        />
      </div>
    </div>

    <section class="my-12 space-y-6 text-lg md:max-w-[460px] text-gray-400">
      <p>
        <code>sshx</code> gives you a link to share your terminal with anyone,
        on a <b>multiplayer infinite canvas</b>.
      </p>
      <p>
        It has <b>real-time collaboration</b>, with remote cursors and chat.
        Also, it's <b>fast</b> and <b>end-to-end encrypted</b>, with client and
        server binaries written in Rust.
      </p>
      <p>
        Install <code>sshx</code> with just one command. Use it for teaching, debugging,
        or cloud access.
      </p>
    </section>

    <div class="pb-12 md:pb-36">
      <button
        class="bg-pink-700 hover:bg-pink-600 active:ring-4 active:ring-pink-500/50 text-lg font-semibold px-8 py-2 rounded-full"
        on:click={() => installationEl.scrollIntoView({ behavior: "smooth" })}
      >
        Get Started
      </button>
    </div>

    <h2 bind:this={installationEl} class="mt-40 mb-16">
      Get started in <span class="title-gradient">two quick steps</span>
    </h2>

    <div
      class="grid lg:grid-cols-2 gap-16 lg:text-center mb-12 text-lg text-gray-300"
    >
      <div class="space-y-6">
        <h3 class="step-heading">
          <span class="pill mr-3">1</span> Install the CLI
        </h3>
        <p class="text-gray-400">
          Get the <code>sshx</code> binary by running the following in your terminal.
          It's tiny and downloads in seconds (3 MB).
        </p>
        <div class="flex flex-col items-start text-base">
          <div class="flex rounded-t text-sm bg-gray-900">
            {#each installs as method}
              <button
                class="px-2 py-1 border-l border-t last:border-r border-gray-500 first:rounded-tl last:rounded-tr"
                class:bg-gray-700={selectedInstall === method}
                on:click={() => (selectedInstall = method)}
              >
                {method.title}
              </button>
            {/each}
          </div>
          <pre class="rounded-b rounded-r w-full">{selectedInstall.steps}</pre>
        </div>

        <p class="text-gray-400">
          You can also build it <a
            target="_blank"
            rel="noreferrer"
            class="underline underline-offset-2 text-gray-300"
            href="https://github.com/ekzhang/sshx">from source</a
          >, if you'd like.
        </p>
      </div>
      <div class="space-y-6">
        <h3 class="step-heading">
          <span class="pill mr-3">2</span> Share your terminal
        </h3>

        <p class="text-gray-400">Run this command in your favorite terminal.</p>

        <pre class="rounded">sshx</pre>

        <p class="text-gray-400">
          This kicks off a live, encrypted session with a unique link. Open the
          link in a web browser to join.
        </p>
      </div>
    </div>

    <hr class="mt-40 mb-12" />

    <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-4 md:gap-6 mb-6">
      {#each socials as social}
        <a
          target="_blank"
          rel="noreferrer"
          href={social.href}
          class="border border-gray-700 hover:bg-gray-900 transition-colors p-4 text-center text-lg font-medium rounded-lg"
        >
          {social.title}
        </a>
      {/each}
    </div>

    <p class="mb-12 text-center text-gray-400">
      open source, &copy; Eric Zhang 2023
    </p>
  </main>
</div>

<style lang="postcss">
  h1 {
    @apply font-bold text-4xl sm:text-5xl max-w-[26ch] py-2;
    line-height: 1.15;
  }

  h2 {
    @apply font-bold text-3xl sm:text-4xl md:text-center scroll-mt-16;
  }

  b {
    @apply text-gray-300 font-semibold;
  }

  code {
    @apply text-[0.9em] text-gray-200 font-medium bg-gray-700 px-1 py-0.5 rounded;
  }

  pre {
    @apply bg-gray-900 p-3 whitespace-pre-wrap border border-gray-500;
  }

  hr {
    @apply mx-auto md:w-1/2 border-gray-800;
  }

  .title-gradient {
    @apply text-transparent bg-clip-text bg-gradient-to-r from-fuchsia-400 to-blue-500;
  }

  .step-heading {
    @apply text-2xl text-gray-200 font-semibold flex items-center md:justify-center;
  }

  .pill {
    @apply flex justify-center items-center w-7 h-7 rounded-full;
    @apply text-base font-normal border border-current;
  }
</style>
