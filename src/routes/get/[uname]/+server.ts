import { error } from "@sveltejs/kit";

import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ params }) => {
  const idx = params.uname.search("-");
  if (idx === -1) {
    throw error(404);
  }

  const os = params.uname.substring(0, idx).toLowerCase();
  const arch = params.uname.substring(idx + 1).toLowerCase();

  if (os !== "linux" && os !== "darwin") {
    throw error(404);
  }

  const archMapping: Record<string, string> = {
    aarch64_be: "aarch64",
    aarch64: "aarch64",
    arm64: "aarch64",
    armv8b: "aarch64",
    armv8l: "aarch64",
    x86_64: "x86_64",
    x64: "x86_64",
    amd64: "x86_64",
  };

  if (!Object.hasOwn(archMapping, arch)) {
    throw error(404);
  }

  let triple = archMapping[arch];
  if (os === "linux") {
    triple += "-unknown-linux-musl";
  } else {
    triple += "-apple-darwin";
  }

  const proxy = await fetch("https://s3.amazonaws.com/sshx/sshx-" + triple);
  const resp = new Response(proxy.body);
  resp.headers.set("Content-Disposition", `attachment; filename="sshx"`);
  return resp;
};
