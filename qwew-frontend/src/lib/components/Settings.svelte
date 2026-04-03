<script lang="ts">
  import { app } from "../store.svelte";
  import { api } from "../api";

  let { onClose }: { onClose: () => void } = $props();

  let inviteCode = $state("");
  let inviteExpiry = $state("");
  let inviteLoading = $state(false);
  let inviteCopied = $state(false);
  let inviteError = $state("");

  async function generateInvite() {
    if (!app.auth) return;
    inviteLoading = true;
    inviteError = "";
    inviteCopied = false;
    try {
      const res = await api.createInvite(app.auth.token);
      inviteCode = res.code;
      inviteExpiry = new Date(res.expires_at).toLocaleDateString([], {
        month: "short", day: "numeric", year: "numeric"
      });
    } catch (e: any) {
      inviteError = e.message;
    } finally {
      inviteLoading = false;
    }
  }

  async function copyInvite() {
    await navigator.clipboard.writeText(inviteCode);
    inviteCopied = true;
    setTimeout(() => inviteCopied = false, 2000);
  }
</script>

<div class="overlay" onclick={onClose} role="presentation"></div>

<div class="panel">
  <div class="panel-header">
    <span class="header-mark">·--</span>
    <span>settings</span>
    <button class="close" onclick={onClose}>×</button>
  </div>

  <div class="section">
    <div class="section-title">-- account --</div>
    <div class="row">
      <span class="label">· user</span>
      <span class="value">{app.auth?.username}</span>
    </div>
  </div>

  <div class="section">
    <div class="section-title">-- invite --</div>
    <p class="hint">· single-use · expires in 3 days</p>
    <button class="btn" onclick={generateInvite} disabled={inviteLoading}>
      {inviteLoading ? "· · ·" : "[ generate ]"}
    </button>
    {#if inviteError}
      <p class="error">! {inviteError}</p>
    {/if}
    {#if inviteCode}
      <div class="invite-row">
        <code class="invite-code">{inviteCode}</code>
        <button class="btn-copy" onclick={copyInvite}>
          {inviteCopied ? "·ok·" : "[cp]"}
        </button>
      </div>
      <p class="hint">· expires {inviteExpiry}</p>
    {/if}
  </div>

  <div class="section muted">
    <div class="section-title">-- appearance -- <span class="tag">soon</span></div>
    <p class="hint">· themes · font · density</p>
  </div>

  <div class="section muted">
    <div class="section-title">-- notifications -- <span class="tag">soon</span></div>
    <p class="hint">· sound · desktop · dnd</p>
  </div>

  <div class="section muted">
    <div class="section-title">-- api access -- <span class="tag">soon</span></div>
    <p class="hint">· build your own client</p>
  </div>

  <div class="section">
    <button class="btn danger" onclick={() => { onClose(); app.logout(); }}>
      [ log out ]
    </button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 10;
  }

  .panel {
    position: fixed;
    top: 0;
    left: 0;
    width: 260px;
    height: 100vh;
    background: var(--surface);
    border-right: 1px solid var(--border);
    z-index: 11;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    animation: slide-in 0.15s ease;
  }

  @keyframes slide-in {
    from { transform: translateX(-100%); }
    to   { transform: translateX(0); }
  }

  .panel-header {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 700;
    font-size: 0.88rem;
    color: var(--text);
  }

  .header-mark { color: var(--accent); }

  .close {
    margin-left: auto;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 1rem;
    font-family: inherit;
    padding: 2px 6px;
    transition: color 0.1s;
  }

  .close:hover { color: var(--error); }

  .section {
    padding: 14px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section.muted { opacity: 0.45; }

  .section-title {
    font-size: 0.75rem;
    color: var(--accent);
    letter-spacing: 0.04em;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .tag {
    font-size: 0.65rem;
    color: var(--text-muted);
    border: 1px solid var(--border);
    padding: 0 4px;
  }

  .row {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
  }

  .label { color: var(--text-muted); }
  .value { color: var(--text); }

  .hint {
    font-size: 0.78rem;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .btn {
    padding: 6px 12px;
    background: none;
    color: var(--accent);
    border: 1px solid var(--accent);
    font-family: inherit;
    font-size: 0.82rem;
    cursor: pointer;
    align-self: flex-start;
    letter-spacing: 0.05em;
    transition: background 0.15s, color 0.15s;
  }

  .btn:hover:not(:disabled) {
    background: var(--accent);
    color: var(--bg);
  }

  .btn:disabled { opacity: 0.4; cursor: default; }

  .btn.danger {
    color: var(--error);
    border-color: var(--error);
    width: 100%;
    text-align: center;
  }

  .btn.danger:hover {
    background: var(--error);
    color: var(--bg);
  }

  .invite-row {
    display: flex;
    align-items: center;
    gap: 8px;
    border: 1px solid var(--border);
    padding: 7px 10px;
    background: var(--bg);
  }

  .invite-code {
    flex: 1;
    font-family: inherit;
    font-size: 0.85rem;
    letter-spacing: 0.08em;
    color: var(--green);
  }

  .btn-copy {
    background: none;
    border: 1px solid var(--border);
    color: var(--text-muted);
    font-family: inherit;
    font-size: 0.75rem;
    padding: 2px 6px;
    cursor: pointer;
    transition: color 0.1s, border-color 0.1s;
  }

  .btn-copy:hover {
    color: var(--accent);
    border-color: var(--accent);
  }

  .error { font-size: 0.8rem; color: var(--error); }
</style>
