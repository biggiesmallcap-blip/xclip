<script lang="ts">
  import { clip } from '../stores/clip.svelte';
  import { playerRef } from './playerRef.svelte';
  import { autoTrack } from '../bindings/tauri';
  import { defaultRegion } from './focus/coords';
  import { aspectRatioValue, type FocusMode, type AspectRatio, ASPECT_RATIOS } from './focus/types';
  import { nextKeyframeIndex, prevKeyframeIndex, sortedKeyframes } from './focus/interpolate';

  const ft = $derived(clip.focusTools);

  function sourceSize() {
    return clip.probe ? { width: clip.probe.width, height: clip.probe.height } : null;
  }

  function lockAspect(): number | null {
    const s = sourceSize();
    if (!s) return null;
    if (ft.mode === 'reframe') return aspectRatioValue(ft.aspectRatio, s);
    if (ft.shape === 'circle') return 1;
    return null;
  }

  function setMode(m: FocusMode) {
    ft.mode = m;
    if (m === 'off') {
      ft.enabled = false;
    } else {
      ft.enabled = true;
    }
    const s = sourceSize();
    if (s && (ft.region.width === 0 || ft.region.height === 0)) {
      ft.region = defaultRegion(s, lockAspect());
    }
  }

  function setAspect(a: AspectRatio) {
    ft.aspectRatio = a;
    const s = sourceSize();
    if (!s) return;
    if (ft.mode === 'reframe') {
      ft.region = defaultRegion(s, aspectRatioValue(a, s));
    }
  }

  function addKeyframe() {
    const v = playerRef.video;
    if (!v) return;
    clip.addKeyframe(v.currentTime, ft.region);
    clip.sortKeyframes();
  }

  function deleteKeyframe() {
    if (ft.selectedKeyframeIndex === null) return;
    clip.deleteKeyframe(ft.selectedKeyframeIndex);
  }

  function gotoKeyframe(idx: number) {
    if (idx < 0) return;
    const k = ft.keyframes[idx];
    const v = playerRef.video;
    if (v) v.currentTime = k.time;
    clip.selectKeyframe(idx);
  }

  function prevKf() {
    const v = playerRef.video;
    if (!v) return;
    gotoKeyframe(prevKeyframeIndex(ft.keyframes, v.currentTime));
  }

  function nextKf() {
    const v = playerRef.video;
    if (!v) return;
    gotoKeyframe(nextKeyframeIndex(ft.keyframes, v.currentTime));
  }

  async function recordPath() {
    const v = playerRef.video;
    if (!v) return;
    ft.trackingMode = 'record_path';
    ft.recording.active = true;
    try { await v.play(); } catch {}
  }

  function stopRecord() {
    ft.recording.active = false;
    clip.sortKeyframes();
    const v = playerRef.video;
    if (v && !v.paused) v.pause();
  }

  async function runAutoTrack() {
    const v = playerRef.video;
    if (!v) return;
    ft.autoTrackingStatus = 'tracking';
    ft.autoTrackingError = null;
    try {
      const res = await autoTrack({
        inputPath: clip.srcPath,
        start: clip.start,
        end: clip.end,
        targetBox: ft.region,
        sampleInterval: 0.1,
      });
      ft.keyframes = res.keyframes.map((k) => ({
        time: k.time, x: k.x, y: k.y, width: k.width, height: k.height,
      }));
      clip.sortKeyframes();
      ft.autoTrackingStatus = 'done';
    } catch (e) {
      ft.autoTrackingStatus = 'error';
      ft.autoTrackingError = String(e);
      clip.toast = `Auto track unavailable: ${String(e)}`;
    }
  }

  function clearTrack() {
    clip.clearKeyframes();
  }

  // Auto-persist when the toggle is on and any persisted-relevant field changes.
  $effect(() => {
    if (!ft.persistDefaults) return;
    // touch the fields we persist so the effect tracks them
    void ft.enabled; void ft.mode; void ft.trackingMode; void ft.aspectRatio; void ft.shape;
    void ft.label.text; void ft.label.position; void ft.label.textColor; void ft.label.backgroundColor;
    void ft.style.strokeColor; void ft.style.strokeWidth; void ft.style.fillColor; void ft.style.fillOpacity;
    void ft.style.blurAmount; void ft.style.pixelSize; void ft.style.spotlightDim;
    void ft.recording.smoothing; void ft.recording.sampleIntervalMs;
    clip.persistDefaultsNow();
  });

  const sortedKfs = $derived(sortedKeyframes(ft.keyframes));
</script>

<div class="panel" style="display: flex; flex-direction: column; gap: 10px;">
  <div class="row" style="gap: 10px; align-items: center;">
    <label class="row" style="gap: 6px;">
      <input
        type="checkbox"
        checked={ft.enabled}
        onchange={(e) => {
          const target = e.currentTarget as HTMLInputElement;
          ft.enabled = target.checked;
          if (target.checked && (ft.mode === 'off')) ft.mode = 'box';
          if (target.checked) {
            const s = sourceSize();
            if (s) ft.region = defaultRegion(s, lockAspect());
          }
        }}
      />
      <strong>Focus Tools</strong>
    </label>
    <span class="muted" style="font-size: 12px;">{ft.keyframes.length} keyframe{ft.keyframes.length === 1 ? '' : 's'}</span>
    {#if ft.recording.active}
      <span class="rec-indicator" title="Press R to stop">● REC</span>
    {/if}
    <label class="row" style="gap: 4px; margin-left: auto; font-size: 11px;" title="Remember these settings across clips">
      <input
        type="checkbox"
        checked={ft.persistDefaults}
        onchange={(e) => {
          ft.persistDefaults = (e.currentTarget as HTMLInputElement).checked;
          clip.persistDefaultsNow();
        }}
      />
      <span class="muted">Persist defaults</span>
    </label>
    <button
      type="button"
      style="font-size: 11px; padding: 2px 8px;"
      onclick={() => clip.resetFocus()}
      title="Reset Focus Tools to factory defaults"
    >Reset</button>
  </div>

  {#if ft.enabled}
    <div class="row" style="gap: 6px; flex-wrap: wrap;">
      <span class="muted">Mode</span>
      {#each [['reframe','Reframe Camera'], ['box','Highlight Box'], ['label','Label Box'], ['blur','Blur / Censor'], ['pixelate','Pixelate'], ['spotlight','Spotlight']] as [val, label]}
        <label class="row" style="gap: 4px;">
          <input
            type="radio"
            name="focusmode"
            value={val}
            checked={ft.mode === val}
            onchange={() => setMode(val as FocusMode)}
          />
          {label}
        </label>
      {/each}
    </div>

    {#if ft.mode === 'reframe'}
      <div class="row" style="gap: 6px; flex-wrap: wrap;">
        <span class="muted">Aspect</span>
        {#each ASPECT_RATIOS as a}
          <label class="row" style="gap: 4px;">
            <input
              type="radio"
              name="focusaspect"
              checked={ft.aspectRatio === a}
              onchange={() => setAspect(a)}
            />
            {a}
          </label>
        {/each}
      </div>
    {:else}
      <div class="row" style="gap: 6px; flex-wrap: wrap;">
        <span class="muted">Shape</span>
        <label class="row" style="gap: 4px;">
          <input type="radio" name="focusshape" checked={ft.shape === 'rectangle'} onchange={() => ft.shape = 'rectangle'} /> Rectangle
        </label>
        <label class="row" style="gap: 4px;">
          <input type="radio" name="focusshape" checked={ft.shape === 'circle'} onchange={() => ft.shape = 'circle'} /> Circle
        </label>
      </div>
    {/if}

    <div class="row" style="gap: 6px; flex-wrap: wrap;">
      <span class="muted">Tracking</span>
      <label class="row" style="gap: 4px;">
        <input type="radio" name="focustrack" checked={ft.trackingMode === 'manual'} onchange={() => ft.trackingMode = 'manual'} /> Manual
      </label>
      <label class="row" style="gap: 4px;">
        <input type="radio" name="focustrack" checked={ft.trackingMode === 'record_path'} onchange={() => ft.trackingMode = 'record_path'} /> Record path
      </label>
      <label class="row" style="gap: 4px;">
        <input type="radio" name="focustrack" checked={ft.trackingMode === 'auto'} onchange={() => ft.trackingMode = 'auto'} /> Auto
      </label>
    </div>

    <div class="row" style="gap: 6px; flex-wrap: wrap;">
      <button onclick={addKeyframe}>+ Keyframe</button>
      <button onclick={deleteKeyframe} disabled={ft.selectedKeyframeIndex === null}>Delete</button>
      <button onclick={prevKf} disabled={ft.keyframes.length === 0}>‹ Prev</button>
      <button onclick={nextKf} disabled={ft.keyframes.length === 0}>Next ›</button>
      {#if ft.recording.active}
        <button onclick={stopRecord} class="primary" title="Stop (R)">■ Stop Recording</button>
      {:else}
        <button onclick={recordPath} title="Record (R)">● Record Path</button>
      {/if}
      <button onclick={runAutoTrack} disabled={ft.autoTrackingStatus === 'tracking'}>
        {ft.autoTrackingStatus === 'tracking' ? 'Tracking…' : 'Auto Track'}
      </button>
      <button onclick={clearTrack} disabled={ft.keyframes.length === 0}>Clear</button>
      <button
        onclick={() => { const s = sourceSize(); if (s) ft.region = defaultRegion(s, lockAspect()); }}
        title="Recenter the focus region"
      >Center</button>
    </div>

    <div class="row" style="gap: 10px; flex-wrap: wrap; align-items: center;">
      <label class="row" style="gap: 6px;">
        <span class="muted">Smoothing</span>
        <input type="range" min="0" max="1" step="0.05" bind:value={ft.recording.smoothing} style="width: 120px;" />
        <span class="muted" style="font-variant-numeric: tabular-nums; min-width: 28px; text-align: right;">{ft.recording.smoothing.toFixed(2)}</span>
      </label>
      <button onclick={() => clip.smoothKeyframes()} disabled={ft.keyframes.length < 3 || ft.recording.smoothing <= 0} title="Smooth existing keyframes — strength controlled by Smoothing slider; click again for more">Smooth</button>
    </div>

    {#if ft.mode === 'box' || ft.mode === 'label' || ft.mode === 'reframe'}
      <details>
        <summary class="muted">Box style</summary>
        <div class="row" style="gap: 10px; flex-wrap: wrap; margin-top: 6px;">
          <label class="row" style="gap: 4px;">Stroke <input type="color" bind:value={ft.style.strokeColor} /></label>
          <label class="row" style="gap: 4px;">Width <input type="number" min="0" max="40" bind:value={ft.style.strokeWidth} style="width: 60px;" /></label>
          <label class="row" style="gap: 4px;">Fill <input type="color" bind:value={ft.style.fillColor} /></label>
          <label class="row" style="gap: 4px;">Opacity <input type="number" min="0" max="1" step="0.05" bind:value={ft.style.fillOpacity} style="width: 70px;" /></label>
        </div>
      </details>
    {/if}

    {#if ft.mode === 'label'}
      <details open>
        <summary class="muted">Label</summary>
        <div class="row" style="gap: 10px; flex-wrap: wrap; margin-top: 6px;">
          <label class="row" style="gap: 4px;">Text <input type="text" bind:value={ft.label.text} style="width: 160px;" /></label>
          <label class="row" style="gap: 4px;">Position
            <select bind:value={ft.label.position}>
              <option value="top">Top</option>
              <option value="bottom">Bottom</option>
              <option value="left">Left</option>
              <option value="right">Right</option>
            </select>
          </label>
          <label class="row" style="gap: 4px;">Text <input type="color" bind:value={ft.label.textColor} /></label>
          <label class="row" style="gap: 4px;">Bg <input type="color" bind:value={ft.label.backgroundColor} /></label>
        </div>
      </details>
    {/if}

    {#if ft.mode === 'blur'}
      <div class="row" style="gap: 10px;">
        <label class="row" style="gap: 4px;">Blur amount
          <input type="number" min="1" max="100" bind:value={ft.style.blurAmount} style="width: 70px;" />
        </label>
      </div>
    {/if}

    {#if ft.mode === 'pixelate'}
      <div class="row" style="gap: 10px;">
        <label class="row" style="gap: 4px;">Pixel size
          <input type="number" min="2" max="64" bind:value={ft.style.pixelSize} style="width: 70px;" />
        </label>
      </div>
    {/if}

    {#if ft.mode === 'spotlight'}
      <div class="row" style="gap: 10px;">
        <label class="row" style="gap: 4px;">Dim
          <input type="number" min="0" max="1" step="0.05" bind:value={ft.style.spotlightDim} style="width: 70px;" />
        </label>
      </div>
    {/if}

    {#if sortedKfs.length > 0}
      <div class="muted" style="font-size: 11px;">
        Keyframes:
        {#each sortedKfs as k, i}
          <button
            type="button"
            class="kf-pill"
            class:selected={ft.selectedKeyframeIndex === ft.keyframes.indexOf(k)}
            onclick={() => gotoKeyframe(ft.keyframes.indexOf(k))}
          >{k.time.toFixed(2)}s</button>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .kf-pill {
    background: var(--panel);
    border: 1px solid var(--border);
    color: var(--fg);
    border-radius: 999px;
    padding: 1px 8px;
    margin: 2px;
    cursor: pointer;
    font-size: 11px;
  }
  .kf-pill.selected {
    border-color: var(--accent);
    color: var(--accent);
  }
  details > summary {
    cursor: pointer;
  }
  .rec-indicator {
    color: var(--danger, #ff3b30);
    font-weight: 600;
    font-size: 12px;
    letter-spacing: 0.04em;
    margin-left: auto;
    animation: rec-pulse 1.2s ease-in-out infinite;
  }
  @keyframes rec-pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.45; }
  }
</style>
