<script lang="ts">
  import { app } from "../store.svelte";
  import { api, type UserResult } from "../api";

  let { onOpenSettings }: { onOpenSettings: () => void } = $props();

  let query = $state("");
  let searchResults = $state<UserResult[]>([]);
  let searching = $state(false);
  let searchError = $state("");

  const isSearching = $derived(query.trim().length > 0);

  async function search(e: KeyboardEvent) {
    if (e.key !== "Enter" || !query.trim() || !app.auth) return;
    searching = true;
    searchError = "";
    try {
      searchResults = await api.searchUsers(app.auth.token, query.trim());
      if (searchResults.length === 0) searchError = "no users found";
    } catch {
      searchError = "search failed";
    } finally {
      searching = false;
    }
  }

  function clearSearch() {
    query = "";
    searchResults = [];
    searchError = "";
  }

  async function openDm(user: UserResult) {
    clearSearch();
    const existing = app.conversations.find(c => c.other_username === user.username);
    if (existing) {
      app.selectConversation(existing.id);
    } else {
      app.pendingRecipient = { user_id: user.user_id, username: user.username };
    }
  }

  function formatTime(iso: string | null): string {
    if (!iso) return "";
    const d = new Date(iso);
    const now = new Date();
    if (d.toDateString() === now.toDateString()) {
      return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    }
    return d.toLocaleDateString([], { month: "short", day: "numeric" });
  }
</script>

<aside class="sidebar">
  <div class="header">
    <button class="icon-btn" onclick={onOpenSettings} title="settings">
      ·/·
    </button>
    <div class="search-wrap">
      <span class="search-icon">/</span>
      <input
        type="text"
        placeholder="search..."
        bind:value={query}
        onkeydown={search}
        autocomplete="off"
      />
      {#if isSearching}
        <button class="clear" onclick={clearSearch}>×</button>
      {/if}
    </div>
  </div>

  <div class="conversations">
    {#if isSearching}
      {#if searching}
        <p class="empty">· · ·</p>
      {:else if searchError}
        <p class="empty">! {searchError}</p>
      {:else}
        {#each searchResults as user (user.user_id)}
          <button class="convo" onclick={() => openDm(user)}>
            <div class="convo-top">
              <span class="convo-icon">+</span>
              <span class="convo-name">{user.username}</span>
            </div>
            <div class="convo-bottom">
              <span class="convo-preview">· new conversation</span>
            </div>
          </button>
        {/each}
      {/if}
    {:else}
      {#if app.conversations.length === 0}
        <p class="empty">· no conversations yet ·</p>
      {/if}
      {#each app.conversations as convo (convo.id)}
        <button
          class="convo"
          class:active={app.activeConversationId === convo.id}
          onclick={() => app.selectConversation(convo.id)}
        >
          <div class="convo-top">
            <span class="convo-icon">{app.activeConversationId === convo.id ? "»" : "·"}</span>
            <span class="convo-name">{convo.other_username}</span>
            <span class="convo-time">{formatTime(convo.last_message_at)}</span>
          </div>
          <div class="convo-bottom">
            <span class="convo-preview">{convo.last_message ?? ""}</span>
            {#if convo.unread_count > 0}
              <span class="badge">+{convo.unread_count}</span>
            {/if}
          </div>
        </button>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    min-width: 240px;
    background: var(--surface);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .header {
    padding: 8px 10px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .icon-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-family: inherit;
    font-size: 0.85rem;
    padding: 4px 6px;
    letter-spacing: 0.05em;
    flex-shrink: 0;
    transition: color 0.1s;
  }

  .icon-btn:hover { color: var(--accent); }

  .search-wrap {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 4px;
    border: 1px solid var(--border);
    padding: 4px 8px;
    background: var(--bg);
    transition: border-color 0.15s;
  }

  .search-wrap:focus-within { border-color: var(--accent); }

  .search-icon {
    color: var(--text-muted);
    font-size: 0.85rem;
    flex-shrink: 0;
  }

  .search-wrap input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text);
    font-size: 0.85rem;
    font-family: inherit;
    outline: none;
    min-width: 0;
  }

  .search-wrap input::placeholder { color: var(--text-muted); }

  .clear {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0;
    line-height: 1;
    font-family: inherit;
  }

  .clear:hover { color: var(--error); }

  .conversations { flex: 1; overflow-y: auto; }

  .empty {
    padding: 20px 16px;
    color: var(--text-muted);
    font-size: 0.82rem;
    text-align: center;
  }

  .convo {
    width: 100%;
    padding: 10px 12px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    text-align: left;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 3px;
    transition: background 0.1s;
    font-family: inherit;
  }

  .convo:hover { background: var(--surface-hi); }

  .convo.active {
    background: var(--surface-hi);
    border-left: 2px solid var(--accent);
  }

  .convo-top {
    display: flex;
    align-items: baseline;
    gap: 5px;
  }

  .convo-icon {
    color: var(--accent);
    font-size: 0.8rem;
    flex-shrink: 0;
    width: 10px;
  }

  .convo-name {
    flex: 1;
    font-weight: 600;
    font-size: 0.88rem;
    color: var(--text);
  }

  .convo-time {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .convo-bottom {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-left: 15px;
  }

  .convo-preview {
    font-size: 0.8rem;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 170px;
  }

  .badge {
    color: var(--accent);
    font-size: 0.72rem;
    font-weight: 700;
    white-space: nowrap;
  }
</style>
