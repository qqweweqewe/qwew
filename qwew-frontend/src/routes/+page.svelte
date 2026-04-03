<script lang="ts">
  import { onMount } from "svelte";
  import { app } from "../lib/store.svelte";
  import Auth from "../lib/components/Auth.svelte";
  import Sidebar from "../lib/components/Sidebar.svelte";
  import ChatWindow from "../lib/components/ChatWindow.svelte";
  import Settings from "../lib/components/Settings.svelte";

  let settingsOpen = $state(false);

  onMount(() => app.init());
</script>

{#if app.initializing}
  <div class="loading">
    <span class="dot">· · ·</span>
  </div>
{:else if !app.auth}
  <Auth />
{:else}
  <div class="layout">
    {#if settingsOpen}
      <Settings onClose={() => settingsOpen = false} />
    {/if}
    <Sidebar onOpenSettings={() => settingsOpen = true} />
    <ChatWindow />
  </div>
{/if}

<style>
  .layout {
    display: flex;
    height: 100vh;
    overflow: hidden;
  }

  .loading {
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dot {
    font-family: monospace;
    font-size: 1.2rem;
    color: var(--accent);
    letter-spacing: 0.4em;
    animation: pulse 1.2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.2; }
    50%       { opacity: 1; }
  }
</style>
