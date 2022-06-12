/**
 * @file Internal library for sshx, providing real-time communication.
 *
 * The contents of this file are technically general, not sshx-specific, but it
 * is not open-sourced as its own library because it's not ready for that.
 */

/** How long to wait between reconnections. */
const RECONNECT_DELAY = 500;

/** Number of messages to queue  */
const BUFFER_SIZE = 64;

export type SrocketOptions<T> = {
  /** Handle a message received from the server. */
  onMessage(message: T): void;

  /** Called when the socket connects to the server. */
  onConnect?(): void;

  /** Called when a connected socket is closed. */
  onDisconnect?(): void;

  /** Called when an incoming or existing connection is closed due to error. */
  onError?(event: Event): void;
};

/** A reconnecting WebSocket client for real-time communication. */
export class Srocket<T, U> {
  private ws: WebSocket | null;
  private connected: boolean;
  private buffer: string[];
  private disposed: boolean;

  constructor(
    private readonly url: string,
    private readonly options: SrocketOptions<T>,
  ) {
    this.ws = null;
    this.connected = false;
    this.buffer = [];
    this.disposed = false;
    this.reconnect();
  }

  /** Queue a message to send to the server, with "at-most-once" semantics. */
  send(message: U) {
    const data = JSON.stringify(message);
    if (this.connected && this.ws) {
      this.ws.send(data);
    } else {
      if (this.buffer.length < BUFFER_SIZE) {
        this.buffer.push(data);
      }
    }
  }

  /** Dispose of this WebSocket permanently. */
  dispose() {
    this.stateChange(false);
    this.disposed = true;
    this.ws?.close();
  }

  private reconnect() {
    if (this.disposed) return;
    if (this.ws !== null) {
      throw new Error("invariant violation: reconnecting while connected");
    }
    this.ws = new WebSocket(this.url);
    this.ws.onopen = () => {
      this.stateChange(true);
    };
    this.ws.onclose = () => {
      this.ws = null;
      this.stateChange(false);
      setTimeout(() => this.reconnect(), RECONNECT_DELAY);
    };
    this.ws.onerror = (event) => {
      this.options.onError?.(event);
    };
    this.ws.onmessage = (event) => {
      if (typeof event.data === "string") {
        const message: T = JSON.parse(event.data);
        this.options.onMessage(message);
      } else {
        console.warn("unexpected non-string message, ignoring");
      }
    };
  }

  private stateChange(connected: boolean) {
    if (!this.disposed && connected !== this.connected) {
      this.connected = connected;
      if (connected) {
        this.options.onConnect?.();

        if (!this.ws) {
          throw new Error("invariant violation: connected but ws is null");
        }
        // Send any queued messages.
        for (const message of this.buffer) {
          this.ws.send(message);
        }
        this.buffer = [];
      } else {
        this.options.onDisconnect?.();
      }
    }
  }
}
