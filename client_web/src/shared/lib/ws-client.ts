import { ClientMessage, WsEvent } from "./ws-types";

const PING_INTERVAL = 30_000;
const MAX_BACKOFF = 30_000;
const STABLE_CONNECTION_THRESHOLD = 60_000;

export type WsMessageHandler = (event: WsEvent) => void;

export class WsClient {
  private ws: WebSocket | null = null;
  private handlers: WsMessageHandler[] = [];
  private pendingMessages: ClientMessage[] = [];
  private attemptCount = 0;
  private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  private stableTimeout: ReturnType<typeof setTimeout> | null = null;
  private pingInterval: ReturnType<typeof setInterval> | null = null;
  private stopped = false;

  constructor(private readonly getToken: () => string | null) {}

  connect(): void {
    this.stopped = false;
    this.openSocket();
  }

  disconnect(): void {
    this.stopped = true;
    this.clearTimers();
    this.closeSocket();
  }

  send(msg: ClientMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    } else {
      this.pendingMessages.push(msg);
    }
  }

  onMessage(handler: WsMessageHandler): () => void {
    this.handlers.push(handler);
    return () => {
      this.handlers = this.handlers.filter((h) => h !== handler);
    };
  }

  private openSocket(): void {
    if (this.stopped) return;

    this.closeSocket();
    this.clearReconnectTimeout();

    const token = this.getToken();
    if (!token) return;

    const WS_BASE_URL = process.env.NEXT_PUBLIC_WS_URL ?? "ws://localhost:8080";
    this.ws = new WebSocket(`${WS_BASE_URL}/ws?token=${token}`);

    this.ws.onopen = () => {
      this.stableTimeout = setTimeout(() => {
        this.attemptCount = 0;
      }, STABLE_CONNECTION_THRESHOLD);

      for (const msg of this.pendingMessages) {
        this.ws!.send(JSON.stringify(msg));
      }
      this.pendingMessages = [];

      this.pingInterval = setInterval(() => {
        this.send({ type: "ping" });
      }, PING_INTERVAL);
    };

    this.ws.onmessage = (event) => {
      try {
        const wsEvent: WsEvent = JSON.parse(event.data as string);
        for (const handler of this.handlers) {
          handler(wsEvent);
        }
      } catch {}
    };

    this.ws.onerror = () => {};

    this.ws.onclose = () => {
      this.clearTimers();
      this.scheduleReconnect();
    };
  }

  private scheduleReconnect(): void {
    if (this.stopped) return;
    const delay = Math.min(1_000 * Math.pow(2, this.attemptCount), MAX_BACKOFF);
    this.attemptCount += 1;
    this.reconnectTimeout = setTimeout(() => {
      this.openSocket();
    }, delay);
  }

  private closeSocket(): void {
    if (!this.ws) return;
    this.ws.onopen = null;
    this.ws.onmessage = null;
    this.ws.onerror = null;
    this.ws.onclose = null;
    this.ws.close();
    this.ws = null;
  }

  private clearTimers(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
    if (this.stableTimeout) {
      clearTimeout(this.stableTimeout);
      this.stableTimeout = null;
    }
  }

  private clearReconnectTimeout(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
  }
}
