<script lang="ts">
  import {
    CastIcon,
    DownloadIcon,
    GitBranchIcon,
    HardDriveIcon,
    ImageIcon,
    LockIcon,
    PackageIcon,
    RefreshCwIcon,
    Share2Icon,
  } from "svelte-feather-icons";

  import logotypeDark from "$lib/assets/logotype-dark.svg";
  import landingGraphic from "$lib/assets/landing-graphic.svg";
  import landingBackground from "$lib/assets/landing-background.svg";
  import TeaserImage from "$lib/ui/TeaserImage.svelte";
  import CopyableCode from "$lib/ui/CopyableCode.svelte";
  import DownloadLink from "$lib/ui/DownloadLink.svelte";

  let installationEl: HTMLDivElement;

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

  function scrollToInstallation() {
    installationEl.scrollIntoView({ behavior: "smooth" });
  }
</script>

<main
  class="max-w-screen-xl mx-auto px-4 md:px-8 lg:px-16 text-zinc-100 overflow-x-hidden"
>
  <header class="mt-6 mb-4 sm:my-8 md:my-12">
    <img class="h-12 sm:h-16 -mx-1" src={logotypeDark} alt="sshx logo" />
  </header>
  <h1
    class="font-medium text-3xl sm:text-4xl md:text-5xl max-w-[26ch] py-2 mb-6 md:mb-0 sm:tracking-tight leading-[1.15]"
  >
    A secure web-based,
    <span class="title-gradient">collaborative</span> terminal
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

  <section class="my-12 space-y-6 sm:text-lg md:max-w-[460px] text-zinc-400">
    <p>
      <code class="name">sshx</code> lets you share your terminal with anyone by
      link, on a
      <b>multiplayer infinite canvas</b>.
    </p>
    <p>
      It has <b>real-time collaboration</b>, with remote cursors and chat. It's
      also <b>fast</b> and <b>end-to-end encrypted</b>, with a lightweight
      server written in Rust.
    </p>
    <p>
      Install <code class="name">sshx</code> with a single command. Use it for teaching,
      debugging, or cloud access.
    </p>
  </section>

  <div class="pb-12 md:pb-36">
    <button
      class="bg-pink-700 hover:bg-pink-600 active:ring-4 active:ring-pink-500/50 text-lg font-medium px-8 py-2 rounded-full"
      on:click={scrollToInstallation}
    >
      Get Started
    </button>
  </div>

  <div class="mt-8 md:mt-32 grid md:grid-cols-2 xl:grid-cols-3 gap-4 md:gap-6">
    <div class="feature-block">
      <div class="feature-icon">
        <CastIcon size="14" />
      </div>
      <h3>Collaborative</h3>
      <p>Invite people by sharing a secure, unique browser link.</p>
    </div>
    <div class="feature-block">
      <div class="feature-icon">
        <LockIcon size="14" />
      </div>
      <h3>End-to-end encrypted</h3>
      <p>Send data securely; the server never sees what you're typing.</p>
    </div>
    <div class="feature-block">
      <div class="feature-icon">
        <HardDriveIcon size="14" />
      </div>
      <h3>Cross-platform</h3>
      <p>Use the command-line tool on macOS, Linux, and Windows.</p>
    </div>
    <div class="feature-block">
      <div class="feature-icon">
        <ImageIcon size="14" />
      </div>
      <h3>Infinite canvas</h3>
      <p>Move and resize multiple terminals at once, in any arrangement.</p>
    </div>
    <div class="feature-block">
      <div class="feature-icon">
        <RefreshCwIcon size="14" />
      </div>
      <h3>Live presence</h3>
      <p>See other people's names and cursors within the app.</p>
    </div>
    <div class="feature-block">
      <div class="feature-icon">
        <Share2Icon size="14" />
      </div>
      <h3>Ultra-fast mesh networking</h3>
      <p>
        Connect from anywhere to the nearest distributed peer in a global
        network.
      </p>
    </div>
  </div>

  <div class="my-48 hidden md:block">
    <TeaserImage />
  </div>

  <h2
    bind:this={installationEl}
    class="mt-32 mb-12 font-medium text-3xl sm:text-4xl md:text-center scroll-mt-16"
  >
    Installation
  </h2>

  <section class="installation-section">
    <h3 class="text-xl sm:text-lg">
      <DownloadIcon size="20" class="text-zinc-400 inline-block mr-1 mb-0.5" />
      macOS / Linux
    </h3>
    <div class="text-sm text-zinc-400 md:text-base md:pt-0.5">
      <p class="mb-3">Run the following in your terminal:</p>
      <CopyableCode value="curl -sSf https://sshx.io/get | sh" />

      <p class="mt-8 mb-3">Or, download the binary for your platform.</p>
      <div class="flex flex-wrap gap-2 mb-2">
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-aarch64-apple-darwin.tar.gz"
          >macOS ARM64 (Apple Silicon)</DownloadLink
        >
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-x86_64-apple-darwin.tar.gz"
          >macOS x86-64 (Intel)</DownloadLink
        >
      </div>
      <div class="flex flex-wrap gap-2 mb-2">
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-aarch64-unknown-linux-musl.tar.gz"
          >Linux ARM64</DownloadLink
        >
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-x86_64-unknown-linux-musl.tar.gz"
          >Linux x86-64</DownloadLink
        >
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-arm-unknown-linux-musleabihf.tar.gz"
          >Linux ARMv6</DownloadLink
        >
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-armv7-unknown-linux-musleabihf.tar.gz"
          >Linux ARMv7</DownloadLink
        >
      </div>
      <div class="flex flex-wrap gap-2">
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-x86_64-unknown-freebsd.tar.gz"
          >FreeBSD x86-64</DownloadLink
        >
      </div>
    </div>
  </section>

  <section class="installation-section">
    <h3 class="text-xl sm:text-lg">
      <DownloadIcon size="20" class="text-zinc-400 inline-block mr-1 mb-0.5" />
      Windows
    </h3>
    <div class="text-sm text-zinc-400 md:text-base md:pt-0.5">
      <p class="mb-3">Download the executable for your platform.</p>

      <div class="flex flex-wrap gap-2">
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-x86_64-pc-windows-msvc.zip"
          >Windows x86-64</DownloadLink
        >
        <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-i686-pc-windows-msvc.zip"
          >Windows x86</DownloadLink
        >
        <!-- <DownloadLink
          href="https://sshx.s3.amazonaws.com/sshx-aarch64-pc-windows-msvc.zip"
          >Windows ARM64</DownloadLink
        > -->
      </div>
    </div>
  </section>

  <section class="installation-section">
    <h3 class="text-xl sm:text-lg">
      <PackageIcon size="20" class="text-zinc-400 inline-block mr-1 mb-0.5" />
      Build from source
    </h3>
    <div class="text-sm text-zinc-400 md:text-base md:pt-0.5">
      <p class="mb-3">
        Ensure you have up-to-date versions of Rust and protoc. Compile sshx and
        add it to the system path.
      </p>
      <CopyableCode value="cargo install sshx" />
    </div>
  </section>

  <section class="installation-section">
    <h3 class="text-xl sm:text-lg">
      <GitBranchIcon size="20" class="text-zinc-400 inline-block mr-1 mb-0.5" />
      GitHub Actions
    </h3>
    <div class="text-sm text-zinc-400 md:text-base md:pt-0.5">
      <p class="mb-3">
        On GitHub Actions or other CI providers, run this command. It pauses
        your workflow and starts a collaborative session.
      </p>
      <CopyableCode value="curl -sSf https://sshx.io/get | sh -s run" />
    </div>
  </section>

  <hr class="mt-32 mb-12" />

  <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 md:gap-6 mb-6">
    {#each socials as social}
      <a
        target="_blank"
        rel="noreferrer"
        href={social.href}
        class="border border-white/10 hover:border-white/30 transition-colors p-4 text-center text-lg font-medium rounded-lg"
      >
        {social.title}
      </a>
    {/each}
  </div>

  <p class="mb-12 text-center text-zinc-400">
    open source, &copy; Eric Zhang 2023
  </p>
</main>

<style lang="postcss">
  b {
    @apply text-zinc-300 font-medium;
  }

  code.name {
    @apply text-[0.9em] text-zinc-100 border border-white/25 px-1 py-0.5 rounded;
  }

  hr {
    @apply mx-auto md:w-1/2 border-zinc-800;
  }

  .title-gradient {
    @apply text-transparent bg-clip-text bg-gradient-to-r from-fuchsia-400 to-blue-500;
  }

  .feature-block {
    @apply relative border rounded-lg border-transparent p-6 sm:p-8;
    background: #111111 padding-box;
  }

  .feature-block::before {
    content: "";
    @apply absolute inset-0;
    z-index: -1;
    margin: -1px;
    border-radius: inherit;
    background: linear-gradient(
      160deg,
      rgba(255, 255, 255, 0.36) 5%,
      rgba(255, 255, 255, 0.08) 25%,
      rgba(255, 255, 255, 0.24) 50%,
      rgba(255, 255, 255, 0.08) 75%,
      rgba(255, 255, 255, 0.28) 95%
    );
    opacity: 0.5;
    transition: opacity 200ms;
  }

  .feature-block:hover::before {
    opacity: 1;
  }

  .feature-block h3 {
    @apply font-medium mb-2;
  }

  .feature-block p {
    @apply text-zinc-400;
  }

  .feature-icon {
    @apply inline-block p-3 rounded-full mb-3 shadow-md border border-zinc-600;
  }

  .installation-section {
    @apply grid sm:grid-cols-[200px,1fr] gap-x-10 gap-y-4 max-w-4xl mx-auto sm:border-t sm:border-white/10 sm:py-6 lg:px-2;
    @apply mb-16 lg:mb-8;
  }
</style>
