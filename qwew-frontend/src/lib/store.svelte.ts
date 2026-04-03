import { api, type Conversation, type Message, type UserResult } from "./api";
import { WsClient } from "./ws";
import { saveSession, loadSession, clearSession } from "./session";

class AppState {
  auth = $state<{ user_id: number; username: string; token: string } | null>(null);
  conversations = $state<Conversation[]>([]);
  activeConversationId = $state<number | null>(null);
  messages = $state<Record<number, Message[]>>({});
  pendingRecipient = $state<UserResult | null>(null);
  // true while we're checking a saved session on startup
  initializing = $state(true);

  private ws: WsClient | null = null;

  async init() {
    const stored = await loadSession();
    if (stored) {
      try {
        // validate token is still good
        await api.getMe(stored.token);
        await this.initSession(stored);
      } catch {
        await clearSession();
      }
    }
    this.initializing = false;
  }

  async login(username: string, password: string) {
    const res = await api.login(username, password);
    await this.initSession(res);
  }

  async register(username: string, password: string, invite_code: string) {
    const res = await api.register(username, password, invite_code);
    await this.initSession(res);
  }

  logout() {
    this.ws?.disconnect();
    this.ws = null;
    this.auth = null;
    this.conversations = [];
    this.activeConversationId = null;
    this.messages = {};
    this.pendingRecipient = null;
    clearSession();
  }

  async loadConversations() {
    if (!this.auth) return;
    this.conversations = await api.getConversations(this.auth.token);
  }

  async selectConversation(id: number) {
    this.activeConversationId = id;
    if (!this.messages[id] && this.auth) {
      this.messages[id] = await api.getHistory(this.auth.token, id);
    }
    this.ws?.send({ type: "mark_read", conversation_id: id });
    const convo = this.conversations.find(c => c.id === id);
    if (convo) convo.unread_count = 0;
  }

  sendMessage(recipient_id: number, content: string) {
    this.ws?.send({ type: "send_message", recipient_id, content });
  }

  private async initSession(res: { user_id: number; username: string; token: string }) {
    this.auth = res;
    await saveSession(res);
    await this.loadConversations();

    this.ws = new WsClient(res.token);
    this.ws.on((event) => {
      if (event.type === "new_message") {
        const msg = event.message;
        if (!this.messages[msg.conversation_id]) {
          this.messages = { ...this.messages, [msg.conversation_id]: [] };
        }
        this.messages[msg.conversation_id] = [...this.messages[msg.conversation_id], msg];

        const convo = this.conversations.find(c => c.id === msg.conversation_id);
        if (convo) {
          convo.last_message = msg.content;
          convo.last_message_at = msg.created_at;
          if (msg.sender_id !== res.user_id && this.activeConversationId !== msg.conversation_id) {
            convo.unread_count++;
          }
        } else {
          this.loadConversations();
        }
      }

      if (event.type === "hello" && this.activeConversationId && this.auth) {
        api.getHistory(this.auth.token, this.activeConversationId).then(msgs => {
          this.messages = { ...this.messages, [this.activeConversationId!]: msgs };
        });
      }
    });

    await this.ws.connect();
  }
}

export const app = new AppState();
