# WebSocket Protocol

## Connection flow

1. `POST /ws/ticket` with `Authorization: Bearer <jwt>` → `{ "ticket": "<uuid>" }`
2. `WS /ws?ticket=<uuid>` → upgrade
3. Server immediately sends `hello`
4. Ticket is single-use, valid for 30 seconds

---

## Client → Server

### send_message
```json
{
  "type": "send_message",
  "recipient_id": 2,
  "content": "hey"
}
```

### mark_read
```json
{
  "type": "mark_read",
  "conversation_id": 3
}
```

### pong
```json
{
  "type": "pong"
}
```
Must be sent in response to a `ping` within 10 seconds or the server closes the connection.

---

## Server → Client

### hello
Sent immediately on connect. Use `user_id` to confirm identity on reconnect.
Use `server_time` to detect clock drift.
```json
{
  "type": "hello",
  "user_id": 1,
  "server_time": "2026-04-02T12:00:00Z"
}
```

### new_message
Delivered to both sender (confirmation) and recipient (real-time delivery).
```json
{
  "type": "new_message",
  "message": {
    "id": 42,
    "conversation_id": 3,
    "sender_id": 1,
    "content": "hey",
    "created_at": "2026-04-02T12:00:00Z"
  }
}
```

### message_read
Sent to the original sender when the recipient calls `mark_read`.
```json
{
  "type": "message_read",
  "conversation_id": 3,
  "reader_id": 2
}
```

### ping
Server-initiated keepalive, sent every 30 seconds.
```json
{
  "type": "ping"
}
```

### error
Sent when the client does something wrong. Never sent for internal errors.
```json
{
  "type": "error",
  "reason": "invalid event"
}
```

Possible reasons:
- `"invalid event"` — malformed JSON or unknown event type
- `"invalid message length"` — content is empty or exceeds 2000 characters
- `"internal error"` — something broke on the server side

---

## Reconnection

1. On disconnect or error → exponential backoff: 1s → 2s → 4s → ... → cap at 30s
2. `POST /ws/ticket` to get a fresh ticket (requires valid JWT)
3. Reconnect with new ticket
4. On `hello` → re-fetch missed messages via `GET /conversations/:id/messages`
