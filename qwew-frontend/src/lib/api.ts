const BASE = "http://qwew.space:3000";

export interface AuthResponse {
  user_id: number;
  username: string;
  token: string;
}

export interface Conversation {
  id: number;
  user1_id: number;
  user2_id: number;
  created_at: string;
  other_username: string;
  last_message: string | null;
  last_message_at: string | null;
  unread_count: number;
}

export interface Message {
  id: number;
  conversation_id: number;
  sender_id: number;
  content: string;
  created_at: string;
}

async function request<T>(method: string, path: string, body?: unknown, token?: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });

  const data = await res.json();
  if (!res.ok) throw new Error(data.error ?? "request failed");
  return data;
}

export interface UserResult {
  user_id: number;
  username: string;
}

export const api = {
  register: (username: string, password: string, invite_code: string) =>
    request<AuthResponse>("POST", "/auth/register", { username, password, invite_code }),

  login: (username: string, password: string) =>
    request<AuthResponse>("POST", "/auth/login", { username, password }),

  getConversations: (token: string) =>
    request<Conversation[]>("GET", "/conversations", undefined, token),

  getHistory: (token: string, conversationId: number) =>
    request<Message[]>("GET", `/conversations/${conversationId}/messages`, undefined, token),

  getWsTicket: (token: string) =>
    request<{ ticket: string }>("POST", "/ws/ticket", undefined, token),

  getMe: (token: string) =>
    request<{ user_id: number; username: string }>("GET", "/auth/me", undefined, token),

  searchUsers: (token: string, q: string) =>
    request<UserResult[]>("GET", `/users/search?q=${encodeURIComponent(q)}`, undefined, token),

  createInvite: (token: string) =>
    request<{ code: string; expires_at: string }>("POST", "/invites", undefined, token),
};
