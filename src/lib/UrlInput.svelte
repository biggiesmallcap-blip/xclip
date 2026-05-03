<script lang="ts">
  import { clip } from '../stores/clip.svelte';
  import { download, probe, ytDlpUpdateNow } from '../bindings/tauri';

  let updating = $state(false);

  async function handleDownload() {
    if (!clip.url.trim() || clip.status === 'downloading') return;
    clip.errorMsg = '';
    clip.status = 'downloading';
    try {
      const { path } = await download(clip.url.trim());
      const info = await probe(path);
      clip.srcPath = path;
      clip.probe = info;
      clip.start = 0;
      clip.end = info.duration;
      clip.status = 'ready';
    } catch (e) {
      clip.status = 'error';
      clip.errorMsg = String(e);
    }
  }

  async function handleUpdateYtDlp() {
    if (updating) return;
    updating = true;
    try {
      const res = await ytDlpUpdateNow();
      switch (res.kind) {
        case 'updated':
          clip.toast = `yt-dlp updated → ${res.to}`;
          break;
        case 'upToDate':
          clip.toast = `yt-dlp up-to-date (${res.version})`;
          break;
        case 'skipped':
          clip.toast = 'yt-dlp checked recently';
          break;
        case 'failed':
          clip.toast = `yt-dlp update failed: ${res.reason}`;
          break;
      }
    } catch (e) {
      clip.toast = `yt-dlp update error: ${String(e)}`;
    } finally {
      updating = false;
    }
  }
</script>

<div class="panel" style="display: flex; flex-direction: column; gap: 6px;">
  <div class="row">
    <input
      type="url"
      placeholder="Paste a video URL (x.com, youtube.com, reddit.com, tiktok.com…)"
      bind:value={clip.url}
      onkeydown={(e) => { if (e.key === 'Enter') handleDownload(); }}
      style="flex: 1"
    />
    <button
      class="primary"
      onclick={handleDownload}
      disabled={clip.status === 'downloading' || !clip.url.trim()}
    >
      {clip.status === 'downloading' ? 'Downloading…' : 'Download'}
    </button>
  </div>
  <div class="row" style="gap: 8px; font-size: 11px;">
    <span class="muted">Downloader auto-updates weekly.</span>
    <button
      type="button"
      onclick={handleUpdateYtDlp}
      disabled={updating}
      style="padding: 1px 8px; font-size: 11px; margin-left: auto;"
      title="Check GitHub for the latest yt-dlp.exe and replace the local copy if newer"
    >
      {updating ? 'Checking…' : 'Check yt-dlp'}
    </button>
  </div>
</div>
