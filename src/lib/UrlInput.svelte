<script lang="ts">
  import { clip } from '../stores/clip.svelte';
  import { download, probe } from '../bindings/tauri';

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
</script>

<div class="panel row">
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
