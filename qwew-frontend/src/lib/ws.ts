import { api } from "./api";

export type ServerEvent =
  | { type: "hello"; user_id: number; server_time: string }
  | { type: "new_message"; message: { id: number; conversation_id: number; sender_id: number; content: string; created_at: string } }
  | { type: "message_read"; conversation_id: number; reader_id: number }
  | { type: "ping" }
  | { type: "error"; reason: string };

type EventHandler = (event: ServerEvent) => void;

const BACKOFF_INITIAL = 1000;
const BACKOFF_MAX = 30000;

export class WsClient {
  private ws: WebSocket | null = null;
  private token: string;
  private handlers: EventHandler[] = [];
  private backoff = BACKOFF_INITIAL;
  private stopped = false;

  constructor(token: string) {
    this.token = token;
  }

  on(handler: EventHandler) {
    this.handlers.push(handler);
    return () => { this.handlers = this.handlers.filter(h => h !== handler); };
  }

  async connect() {
    this.stopped = false;
    await this.tryConnect();
  }

  disconnect() {
    this.stopped = true;
    this.ws?.close();
    this.ws = null;
  }

  send(event: object) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(event));
    }
  }

  private async tryConnect() {
    if (this.stopped) return;

    try {
      const { ticket } = await api.getWsTicket(this.token);
      const ws = new WebSocket(`ws://qwew.space:3000/ws?ticket=${ticket}`);
      this.ws = ws;

      ws.onopen = () => { this.backoff = BACKOFF_INITIAL; };

      ws.onmessage = (e) => {
        const event: ServerEvent = JSON.parse(e.data);
        if (event.type === "ping") {
          this.send({ type: "pong" });
          return;
        }
        this.handlers.forEach(h => h(event));
      };

      ws.onclose = () => this.scheduleReconnect();
      ws.onerror = () => ws.close();

    } catch {
      this.scheduleReconnect();
    }
  }

  private scheduleReconnect() {
    if (this.stopped) return;
    setTimeout(() => this.tryConnect(), this.backoff);
    this.backoff = Math.min(this.backoff * 2, BACKOFF_MAX);
  }
}
