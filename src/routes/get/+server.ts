import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ url }) => {
  const script = `#!/bin/bash

# This is a short script to install the latest version of the sshx binary.
#
# It's meant to be as simple as possible, so if you're not happy hardcoding a
# \`curl | sh\` pipe in your application, you can just download the binary
# directly with the appropriate URL for your architecture.

set +e

target="$(uname -s)-$(uname -m)"
curl -sSf ${url.origin}/get/$target -o /tmp/sshx
chmod +x /tmp/sshx
sudo mv -v /tmp/sshx /usr/local/bin
`;
  return new Response(script, { headers: { "Content-Type": "text/plain" } });
};
