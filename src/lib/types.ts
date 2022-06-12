/** Server message type, see the Rust version. */
export type WsServer = {
  shells?: number[];
  chunks?: [number, [number, string][]];
};

/** Client message type, see the Rust version. */
export type WsClient = {
  create?: null;
  close?: number;
  data?: [number, string];
  subscribe?: [number, number];
};
