type Sid = number; // u32
type Uid = number; // u32

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
  focus: number | null;
};

/** Server message type, see the Rust version. */
export type WsServer = {
  hello?: Uid;
  users?: [Uid, WsUser][];
  userDiff?: [Uid, WsUser | null];
  shells?: [Sid, WsWinsize][];
  chunks?: [Sid, string[]];
  hear?: [Uid, string, string];
  terminated?: [];
  error?: string;
};

/** Client message type, see the Rust version. */
export type WsClient = {
  setName?: string;
  setCursor?: [number, number] | null;
  setFocus?: number | null;
  create?: [];
  close?: Sid;
  move?: [Sid, WsWinsize | null];
  data?: [Sid, Uint8Array];
  subscribe?: [Sid, number];
  chat?: string;
};
