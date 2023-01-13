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
  cursor: [number, number] | null;
};

/** Server message type, see the Rust version. */
export type WsServer = {
  hello?: number;
  users?: [number, WsUser][];
  userDiff?: [number, WsUser | null];
  shells?: [number, WsWinsize][];
  chunks?: [number, [number, string][]];
  terminated?: [];
  error?: string;
};

/** Client message type, see the Rust version. */
export type WsClient = {
  setName?: string;
  setCursor?: [number, number] | null;
  create?: [];
  close?: number;
  move?: [number, WsWinsize | null];
  data?: [number, Uint8Array];
  subscribe?: [number, number];
};
