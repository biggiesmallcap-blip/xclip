<script lang="ts">
  import { clip } from '../stores/clip.svelte';
  import { exportClip, copyFile, defaultOutputDir, revealInFolder, type FocusToolsExport } from '../bindings/tauri';
  import { exportRef } from './exportRef.svelte';
  import FocusToolsPanel from './FocusToolsPanel.svelte';
  import { estimateExportBytes, formatBytes } from './sizeEstimate';

  let defaultDir = $state('');
  $effect(() => {
    defaultOutputDir().then((d) => { defaultDir = d; }).catch(() => {});
  });

  const effectiveDir = $derived(clip.outputDir || defaultDir);
  const duration = $derived(Math.max(0, clip.end - clip.start));
  const estimate = $derived(estimateExportBytes(duration, clip.format, clip.quality));

  const canExport = $derived(
    clip.status === 'ready' && clip.srcPath !== '' && clip.end > clip.start
  );

  function buildFocusPayload(): FocusToolsExport | undefined {
    const ft = clip.focusTools;
    if (!ft.enabled || ft.mode === 'off') return undefined;
    return {
      enabled: true,
      mode: ft.mode,
      aspectRatio: ft.aspectRatio,
      shape: ft.shape,
      region: { ...ft.region },
      keyframes: ft.keyframes.map((k) => ({ ...k })),
      label: { ...ft.label },
      style: { ...ft.style },
    };
  }

  async function triggerExport() {
    if (!canExport) return;
    clip.errorMsg = '';
    clip.status = 'exporting';
    try {
      const { outputPath } = await exportClip({
        inputPath: clip.srcPath,
        start: clip.start,
        end: clip.end,
        format: clip.format,
        quality: clip.quality,
        focusTools: buildFocusPayload(),
        outputDir: clip.outputDir || undefined,
      });
      clip.lastExportPath = outputPath;
      await copyFile(outputPath);
      clip.toast = 'Copied — paste into X';
      clip.status = 'ready';
    } catch (e) {
      clip.status = 'error';
      clip.errorMsg = String(e);
    }
  }

  async function copyLastExport() {
    if (!clip.lastExportPath) return;
    try {
      await copyFile(clip.lastExportPath);
      clip.toast = 'Copied — paste into X';
    } catch (e) {
      clip.toast = `Copy failed: ${String(e)}`;
    }
  }

  $effect(() => {
    exportRef.trigger = triggerExport;
    return () => { exportRef.trigger = null; };
  });
</script>

<FocusToolsPanel />

<div class="panel" style="display: flex; flex-direction: column; gap: 8px;">
  <div class="row" style="gap: 8px; font-size: 11px; flex-wrap: wrap;">
    <span class="muted" style="min-width: 38px;">Save to</span>
    <input
      type="text"
      placeholder={defaultDir || 'default'}
      value={clip.outputDir}
      oninput={(e) => clip.outputDir = (e.currentTarget as HTMLInputElement).value}
      style="flex: 1; min-width: 240px; font-family: ui-monospace, monospace;"
    />
    <button
      type="button"
      style="padding: 2px 8px;"
      onclick={() => revealInFolder(effectiveDir).catch((err) => clip.toast = `Open failed: ${err}`)}
      disabled={!effectiveDir}
      title="Open the output folder"
    >Open</button>
    <button
      type="button"
      style="padding: 2px 8px;"
      onclick={() => clip.outputDir = ''}
      disabled={!clip.outputDir}
      title="Use the default output folder"
    >Default</button>
  </div>

  <div class="row" style="gap: 16px; flex-wrap: wrap;">
    <div class="row">
      <span class="muted">Format</span>
      <label class="row"><input type="radio" bind:group={clip.format} value="mp4" /> MP4</label>
      <label class="row"><input type="radio" bind:group={clip.format} value="gif" /> GIF</label>
    </div>

    <div class="row">
      <span class="muted">Quality</span>
      <label class="row"><input type="radio" bind:group={clip.quality} value="small" /> Small</label>
      <label class="row"><input type="radio" bind:group={clip.quality} value="balanced" /> Balanced</label>
      <label class="row"><input type="radio" bind:group={clip.quality} value="hq" /> HQ</label>
    </div>

    <span
      class="muted"
      style="margin-left: auto; font-variant-numeric: tabular-nums; font-size: 12px;"
      title="Estimated output size — heuristic, ±30%"
    >~{formatBytes(estimate)} · {duration.toFixed(2)}s</span>
    <button
      onclick={copyLastExport}
      disabled={!clip.lastExportPath || clip.status === 'exporting'}
      title="Copy the last exported file to clipboard again"
    >
      Copy Last
    </button>
    <button
      class="primary"
      onclick={triggerExport}
      disabled={!canExport || clip.status === 'exporting'}
      title="Enter key"
    >
      {clip.status === 'exporting' ? 'Exporting…' : 'Export'}
    </button>
  </div>
</div>
