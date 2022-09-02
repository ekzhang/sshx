/** Position and size of a window, see the Rust version. */
export type WsWinsize = {
  x: number;
  y: number;
  rows: number;
  cols: number;
};

/** Information about a user, see the Rust version */
export type WsUser = {
  name: string;
  cursorPos: [number, number] | null;
};

/** Server message type, see the Rust version. */
export type WsServer = {
  hello?: number;
  users?: [number, WsUser][];
  shells?: [number, WsWinsize][];
  chunks?: [number, [number, string][]];
  terminated?: [];
  error?: string;
};

/** Client message type, see the Rust version. */
export type WsClient = {
  create?: [];
  close?: number;
  move?: [number, WsWinsize | null];
  data?: [number, Uint8Array];
  subscribe?: [number, number];
};
