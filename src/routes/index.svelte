<script lang="ts" context="module">
  export const prerender = true;
</script>

<script lang="ts">
  import logotypeDark from "$lib/assets/logotype-dark.svg";
  import landingGraphic from "$lib/assets/landing-graphic.svg";
  import landingBackground from "$lib/assets/landing-background.svg";

  let installationEl: HTMLDivElement;

  const installs = [
    {
      title: "macOS",
      steps: `brew tap ekzhang/sshx
brew install sshx`,
    },
    {
      title: "Linux",
      steps: `curl -sSf $DOWNLOAD_URL -o /tmp/sshx
sudo mv -v /tmp/sshx /usr/local/bin/sshx`,
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
  <main class="max-w-screen-xl mx-auto px-4 md:px-8 lg:px-16">
    <header class="my-12">
      <img class="h-16 -mx-1" src={logotypeDark} alt="sshx logo" />
    </header>
    <h1>
      Web-based, <span class="title-gradient">real-time collaboration</span> for
      your remote terminal
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

    <section class="my-12 space-y-6 text-xl md:max-w-[460px] text-gray-300">
      <p>
        <code>sshx</code> lets you share your terminal by link with anyone on the
        web.
      </p>
      <p>
        It supports <b>live presence</b> and <b>chat</b> so you can see who’s
        online and seamlessly work with them. Plus, it’s <b>fast</b>: just run a
        single Rust-based CLI tool to start your session.
      </p>
      <p>
        Use <code>sshx</code> for pair programming, demos, remote access, or even
        just as a fashionable web interface for your terminal.
      </p>
    </section>

    <div class="pb-12 md:pb-36">
      <button
        class="bg-pink-600 hover:bg-pink-500 active:ring-4 active:ring-pink-500/50 text-lg font-semibold px-8 py-2 rounded-full"
        on:click={() => installationEl.scrollIntoView({ behavior: "smooth" })}
      >
        Get Started
      </button>
    </div>

    <hr />

    <h2 bind:this={installationEl}>
      Get started in <span class="title-gradient">two quick steps</span>
    </h2>

    <div
      class="grid md:grid-cols-2 gap-12 md:text-center mb-12 text-lg text-gray-300"
    >
      <div class="space-y-5">
        <h3 class="step-heading">
          <span class="pill mr-3">1</span> Install the CLI
        </h3>
        <p>
          Download the tiny <code>sshx</code> binary to connect to our servers. It's
          just 6 MB, and you can install it in a couple different ways.
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

        <p>
          You can also see <a
            class="underline"
            href="https://github.com/ekzhang/sshx"
            >other methods of installation</a
          >.
        </p>
      </div>
      <div class="space-y-5">
        <h3 class="step-heading">
          <span class="pill mr-3">2</span> Start a session
        </h3>

        <p>Just run a single command in your shell.</p>

        <pre class="rounded">sshx</pre>

        <p>
          It's that easy! This kicks off a live session with your unique link at <code
            >https://sshx.io/s/$ID</code
          >, and you can invite anyone to collaborate together in your terminal
          on the web.
        </p>
      </div>
    </div>

    <hr />

    <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-4 md:gap-6 mb-6">
      {#each socials as social}
        <a
          href={social.href}
          class="border border-gray-700 hover:bg-gray-900 transition-colors p-4 text-center text-lg font-medium rounded-lg"
        >
          {social.title}
        </a>
      {/each}
    </div>

    <p class="mb-12 text-center text-gray-400">
      open source, &copy; Eric Zhang 2022
    </p>
  </main>
</div>

<style lang="postcss">
  h1 {
    @apply font-extrabold text-4xl sm:text-5xl max-w-[26ch] py-2;
    line-height: 1.15;
  }

  h2 {
    @apply font-extrabold text-3xl sm:text-4xl mb-12 md:text-center scroll-mt-8;
  }

  code {
    @apply text-[0.9em] text-white font-medium bg-gray-700 px-1 py-0.5 rounded;
  }

  pre {
    @apply bg-gray-900 p-3 whitespace-pre-wrap border border-gray-500;
  }

  hr {
    @apply mx-auto md:w-1/2 border-gray-800 my-12;
  }

  .title-gradient {
    @apply text-transparent bg-clip-text bg-gradient-to-r from-fuchsia-400 to-blue-500;
  }

  .step-heading {
    @apply text-2xl text-gray-200 font-bold flex items-center md:justify-center;
  }

  .pill {
    @apply flex justify-center items-center w-[1.5em] h-[1.5em] bg-gray-700 rounded-full;
  }
</style>
