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
    const { Argon2, Argon2Mode } = await import(
      "https://esm.sh/@sphereon/isomorphic-argon2@1.0.1" as any
    );
    const result = await Argon2.hash(key, SALT, {
      mode: Argon2Mode.Argon2id,
      memory: 19 * 1024,
      iterations: 2,
      parallelism: 1,
      hashLength: 16,
    });
    const aesKey = await crypto.subtle.importKey(
      "raw",
      Uint8Array.from(
        result.hex.match(/.{1,2}/g).map((byte: string) => parseInt(byte, 16)),
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
    offset: number,
    data: Uint8Array,
  ): Promise<Uint8Array> {
    if (streamNum === 0n) throw new Error("stream number must be nonzero"); // security check)

    const blockNum = offset >> 4;
    const iv = new Uint8Array(16);
    new DataView(iv.buffer).setBigUint64(0, streamNum);
    new DataView(iv.buffer).setBigUint64(8, BigInt(blockNum));

    const padBytes = offset % 16;
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
