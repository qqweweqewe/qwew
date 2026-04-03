<script lang="ts">
  import { app } from "../store.svelte";

  let mode: "login" | "register" = $state("login");
  let username = $state("");
  let password = $state("");
  let invite_code = $state("");
  let error = $state("");
  let loading = $state(false);

  async function submit() {
    error = "";
    loading = true;
    try {
      if (mode === "login") {
        await app.login(username, password);
      } else {
        await app.register(username, password, invite_code);
      }
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  }
</script>

<div class="auth">
  <div class="card">
    <div class="logo">
<pre class="logo-ascii"> _____
/\  __`\
\ \ \/\ \
 \ \ \ \ \
  \ \ \\'\\
   \ \___\_\
    \/__//_/</pre>
    </div>

    <div class="tabs">
      <button class:active={mode === "login"} onclick={() => { mode = "login"; error = ""; }}>
        {mode === "login" ? "» login" : "  login"}
      </button>
      <button class:active={mode === "register"} onclick={() => { mode = "register"; error = ""; }}>
        {mode === "register" ? "» register" : "  register"}
      </button>
    </div>

    <form onsubmit={(e) => { e.preventDefault(); submit(); }}>
      <div class="field">
        <span class="field-prefix">·</span>
        <input type="text" placeholder="username" bind:value={username}
          autocomplete="username" required />
      </div>
      <div class="field">
        <span class="field-prefix">·</span>
        <input type="password" placeholder="password" bind:value={password}
          autocomplete={mode === "login" ? "current-password" : "new-password"} required />
      </div>
      {#if mode === "register"}
        <div class="field">
          <span class="field-prefix">·</span>
          <input type="text" placeholder="invite code" bind:value={invite_code} required />
        </div>
      {/if}

      {#if error}
        <p class="error">! {error}</p>
      {/if}

      <button type="submit" class="submit" disabled={loading}>
        {loading ? "· · ·" : `[ ${mode} ]`}
      </button>
    </form>
  </div>
</div>

<style>
  .auth {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
  }

  .card {
    width: 340px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .logo-ascii {
    color: var(--accent);
    font-family: inherit;
    font-size: 0.75rem;
    line-height: 1.3;
    text-align: left;
    display: inline-block;
    white-space: pre;
    overflow: visible;
  }

  .logo {
    display: flex;
    justify-content: center;
  }

  .tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    gap: 4px;
  }

  .tabs button {
    flex: 1;
    padding: 7px 4px;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.85rem;
    font-family: inherit;
    border-bottom: 1px solid transparent;
    margin-bottom: -1px;
    text-align: left;
    padding-left: 8px;
  }

  .tabs button.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .field {
    display: flex;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--border);
    background: var(--surface);
    padding: 0 10px;
    transition: border-color 0.15s;
  }

  .field:focus-within {
    border-color: var(--accent);
  }

  .field-prefix {
    color: var(--accent);
    font-size: 1rem;
    line-height: 1;
    flex-shrink: 0;
  }

  input {
    flex: 1;
    padding: 9px 0;
    background: none;
    border: none;
    color: var(--text);
    font-size: 0.9rem;
    font-family: inherit;
    outline: none;
  }

  input::placeholder { color: var(--text-muted); }

  .error {
    font-size: 0.82rem;
    color: var(--error);
  }

  .submit {
    padding: 9px;
    background: none;
    color: var(--accent);
    border: 1px solid var(--accent);
    font-size: 0.9rem;
    font-family: inherit;
    cursor: pointer;
    letter-spacing: 0.1em;
    transition: background 0.15s, color 0.15s;
  }

  .submit:hover:not(:disabled) {
    background: var(--accent);
    color: var(--bg);
  }

  .submit:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
