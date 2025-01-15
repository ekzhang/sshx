/**
 * @file Encryption of byte streams based on a random key.
 *
 * This is used for end-to-end encryption between the terminal source and its
 * client. Keep this file consistent with the Rust implementation.
 */

const SALT: string =
  "This is a non-random salt for sshx.io, since we want to stretch the security of 83-bit keys!";

export class Encrypt {
  private constructor(private aesKey: CryptoKey) {}

  static async new(key: string): Promise<Encrypt> {
    const argon2 = await import(
      "argon2-browser/dist/argon2-bundled.min.js" as any
    );
    const result = await argon2.hash({
      pass: key,
      salt: SALT,
      type: argon2.ArgonType.Argon2id,
      mem: 19 * 1024, // Memory cost in KiB
      time: 2, // Number of iterations
      parallelism: 1,
      hashLen: 16, // Hash length in bytes
    });
    const aesKey = await crypto.subtle.importKey(
      "raw",
      Uint8Array.from(
        result.hashHex
          .match(/.{1,2}/g)
          .map((byte: string) => parseInt(byte, 16)),
      ),
      { name: "AES-CTR" },
      false,
      ["encrypt"],
    );
    return new Encrypt(aesKey);
  }

  async zeros(): Promise<Uint8Array> {
    const zeros = new Uint8Array(16);
    const cipher = await crypto.subtle.encrypt(
      { name: "AES-CTR", counter: zeros, length: 64 },
      this.aesKey,
      zeros,
    );
    return new Uint8Array(cipher);
  }

  async segment(
    streamNum: bigint,
    offset: bigint,
    data: Uint8Array,
  ): Promise<Uint8Array> {
    if (streamNum === 0n) throw new Error("stream number must be nonzero"); // security check)

    const blockNum = offset >> 4n;
    const iv = new Uint8Array(16);
    new DataView(iv.buffer).setBigUint64(0, streamNum);
    new DataView(iv.buffer).setBigUint64(8, blockNum);

    const padBytes = Number(offset % 16n);
    const paddedData = new Uint8Array(padBytes + data.length);
    paddedData.set(data, padBytes);

    const encryptedData = await crypto.subtle.encrypt(
      {
        name: "AES-CTR",
        counter: iv,
        length: 64,
      },
      this.aesKey,
      paddedData,
    );
    return new Uint8Array(encryptedData, padBytes, data.length);
  }
}
