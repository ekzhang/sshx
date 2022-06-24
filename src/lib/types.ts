/** Server message type, see the Rust version. */
export type WsServer = {
  shells?: number[];
  chunks?: [number, [number, string][]];
  terminated?: [];
};

/** Client message type, see the Rust version. */
export type WsClient = {
  create?: [];
  close?: number;
  data?: [number, Uint8Array];
  subscribe?: [number, number];
};
