/** Position and size of a window, see the Rust version. */
export type WsWinsize = {
  x: number;
  y: number;
  rows: number;
  cols: number;
};

/** Server message type, see the Rust version. */
export type WsServer = {
  shells?: [number, WsWinsize][];
  chunks?: [number, [number, string][]];
  terminated?: [];
  error?: string;
};

/** Client message type, see the Rust version. */
export type WsClient = {
  create?: [];
  close?: number;
  move?: [number, WsWinsize];
  data?: [number, Uint8Array];
  subscribe?: [number, number];
};
