<script lang="ts">
  import { clip } from '../stores/clip.svelte';

  $effect(() => {
    if (clip.toast) {
      const t = clip.toast;
      const handle = setTimeout(() => {
        if (clip.toast === t) clip.toast = '';
      }, 2500);
      return () => clearTimeout(handle);
    }
  });

  async function copyError() {
    try {
      await navigator.clipboard.writeText(clip.errorMsg);
      clip.toast = 'Error copied';
    } catch {
      clip.toast = 'Copy failed';
    }
  }

  function dismissError() {
    clip.errorMsg = '';
    clip.status = clip.srcPath ? 'ready' : 'idle';
  }
</script>

{#if clip.toast}
  <div class="toast">{clip.toast}</div>
{/if}

{#if clip.status === 'error' && clip.errorMsg}
  <div class="error-panel">
    <div class="error-actions">
      <button class="primary" onclick={copyError} title="Copy">Copy</button>
      <button onclick={dismissError} title="Dismiss">×</button>
    </div>
    <pre class="error-text">{clip.errorMsg}</pre>
  </div>
{/if}

<style>
  .error-panel {
    position: fixed;
    bottom: 16px;
    left: 16px;
    right: 16px;
    max-width: 720px;
    margin: 0 auto;
    background: var(--panel);
    border: 1px solid var(--danger);
    border-radius: 8px;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.4);
    z-index: 10;
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 50vh;
  }
  .error-actions {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
  }
  .error-text {
    margin: 0;
    color: var(--danger);
    font: 12px/1.45 ui-monospace, "Cascadia Mono", Menlo, Consolas, monospace;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
    -webkit-user-select: text;
    overflow: auto;
    max-height: 40vh;
    padding: 4px 6px;
    background: rgba(0, 0, 0, 0.25);
    border-radius: 4px;
  }
</style>
