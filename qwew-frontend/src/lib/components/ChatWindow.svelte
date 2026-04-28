<script lang="ts">
  import { app } from "../store.svelte";
  import { tick } from "svelte";

  let input = $state("");
  let messagesEl = $state<HTMLDivElement | null>(null);

  const convo = $derived(app.conversations.find(c => c.id === app.activeConversationId));
  const msgs = $derived(app.activeConversationId ? (app.messages[app.activeConversationId] ?? []) : []);
  const chatName = $derived(convo?.other_username ?? app.pendingRecipient?.username ?? null);
  const recipientId = $derived(
    convo
      ? (convo.user1_id === app.auth?.user_id ? convo.user2_id : convo.user1_id)
      : app.pendingRecipient?.user_id ?? null
  );

  $effect(() => {
    msgs.length;
    tick().then(() => {
      if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
    });
  });

  function send() {
    const content = input.trim();
    if (!content || recipientId === null) return;
    app.sendMessage(recipientId, content);
    input = "";
  }

  function formatTime(iso: string): string {
    return new Date(iso).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
</script>

<div class="chat">
  {#if !chatName}
    <div class="empty">
      <span>· · ·</span>
    </div>
  {:else}
    <div class="chat-header">
      <span class="header-arrow">»</span>
      <span class="chat-name">{chatName}</span>
    </div>

    <div class="messages" bind:this={messagesEl}>
      {#each msgs as msg (msg.id)}
        {@const mine = msg.sender_id === app.auth?.user_id}
        <div class="msg" class:mine>
          <div class="bubble">
            {#if !mine}
              <span class="msg-author">{chatName}</span>
            {/if}
            <span class="msg-content">{msg.content}</span>
          </div>
          <span class="time">{formatTime(msg.created_at)}</span>
        </div>
      {/each}
    </div>

    <form class="input-row" onsubmit={(e) => { e.preventDefault(); send(); }}>
      <span class="input-prefix">»</span>
      <input
        type="text"
        placeholder="message {chatName}..."
        bind:value={input}
        autocomplete="off"
      />
      <button type="submit" disabled={!input.trim()}>--&gt;</button>
    </form>
  {/if}
</div>

<style>
  .chat {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--bg);
  }

  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--border);
    font-size: 1.2rem;
    letter-spacing: 0.3em;
  }

  .chat-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface);
  }

  .header-arrow {
    color: var(--accent);
    font-size: 0.9rem;
  }

  .chat-name {
    font-weight: 700;
    font-size: 0.9rem;
    color: var(--text);
    letter-spacing: 0.03em;
  }

  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .msg {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    max-width: 72%;
  }

  .msg.mine {
    align-self: flex-end;
    align-items: flex-end;
  }

  .bubble {
    padding: 7px 11px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-left: 2px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .msg.mine .bubble {
    background: var(--surface-hi);
    border-left-color: var(--accent);
    border-color: var(--border);
  }

  .msg-author {
    font-size: 0.72rem;
    color: var(--accent);
    font-weight: 700;
  }

  .msg-content {
    font-size: 0.88rem;
    color: var(--text);
    line-height: 1.45;
    word-break: break-word;
    white-space: pre-wrap;
  }

  .time {
    font-size: 0.68rem;
    color: var(--text-muted);
    padding: 0 2px;
  }

  .input-row {
    padding: 10px 14px;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--surface);
  }

  .input-prefix {
    color: var(--accent);
    font-size: 0.9rem;
    flex-shrink: 0;
  }

  .input-row input {
    flex: 1;
    padding: 7px 0;
    background: none;
    border: none;
    color: var(--text);
    font-size: 0.88rem;
    font-family: inherit;
    outline: none;
  }

  .input-row input::placeholder { color: var(--text-muted); }

  .input-row button {
    padding: 5px 10px;
    background: none;
    color: var(--accent);
    border: 1px solid var(--accent);
    font-family: inherit;
    font-size: 0.82rem;
    cursor: pointer;
    letter-spacing: 0.05em;
    transition: background 0.15s, color 0.15s;
    flex-shrink: 0;
  }

  .input-row button:hover:not(:disabled) {
    background: var(--accent);
    color: var(--bg);
  }

  .input-row button:disabled {
    opacity: 0.3;
    cursor: default;
  }
</style>
