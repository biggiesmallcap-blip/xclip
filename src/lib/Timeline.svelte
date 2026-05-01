<script lang="ts">
  import { clip } from '../stores/clip.svelte';
  import { playerRef } from './playerRef.svelte';

  let trackEl: HTMLDivElement | null = $state(null);
  let currentTime = $state(0);
  let rangeStart = $state<number | null>(null);
  let rangeEnd = $state<number | null>(null);
  let dragging = $state(false);

  $effect(() => {
    const v = playerRef.video;
    if (!v) return;
    const onTime = () => { currentTime = v.currentTime; };
    v.addEventListener('timeupdate', onTime);
    v.addEventListener('seeked', onTime);
    return () => {
      v.removeEventListener('timeupdate', onTime);
      v.removeEventListener('seeked', onTime);
    };
  });

  const duration = $derived(clip.probe?.duration ?? 0);
  const ft = $derived(clip.focusTools);

  const hasRange = $derived(rangeStart !== null && rangeEnd !== null && Math.abs((rangeEnd ?? 0) - (rangeStart ?? 0)) > 1e-3);

  function pct(t: number) {
    if (duration <= 0) return 0;
    return Math.max(0, Math.min(100, (t / duration) * 100));
  }

  function timeAt(e: MouseEvent | PointerEvent): number {
    if (!trackEl) return 0;
    const rect = trackEl.getBoundingClientRect();
    const f = (e.clientX - rect.left) / rect.width;
    return Math.max(0, Math.min(duration, f * duration));
  }

  function onPointerDown(e: PointerEvent) {
    if (!trackEl || duration <= 0) return;
    if (e.button !== 0) return; // left click only for range
    e.preventDefault();
    dragging = true;
    const t = timeAt(e);
    rangeStart = t;
    rangeEnd = t;
    trackEl.setPointerCapture(e.pointerId);
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    rangeEnd = timeAt(e);
  }

  function onPointerUp(e: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    if (trackEl?.hasPointerCapture(e.pointerId)) trackEl.releasePointerCapture(e.pointerId);
    // No drag (just a click) → seek to that point and clear range.
    if (rangeStart !== null && rangeEnd !== null && Math.abs(rangeEnd - rangeStart) < 1e-3) {
      const v = playerRef.video;
      if (v) v.currentTime = rangeStart;
      rangeStart = null;
      rangeEnd = null;
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Delete' || e.key === 'Backspace') {
      if (hasRange && rangeStart !== null && rangeEnd !== null) {
        e.preventDefault();
        clip.deleteKeyframesInRange(rangeStart, rangeEnd);
        rangeStart = null;
        rangeEnd = null;
      }
    } else if (e.key === 'Escape') {
      rangeStart = null;
      rangeEnd = null;
    }
  }

  function clearRange() {
    rangeStart = null;
    rangeEnd = null;
  }

  function onMarkerContextMenu(e: MouseEvent, idx: number) {
    e.preventDefault();
    e.stopPropagation();
    clip.deleteKeyframe(idx);
  }

  function onMarkerClick(e: MouseEvent, k: { time: number }, i: number) {
    e.stopPropagation();
    if (playerRef.video) playerRef.video.currentTime = k.time;
    clip.selectKeyframe(i);
  }

  const lo = $derived(rangeStart !== null && rangeEnd !== null ? Math.min(rangeStart, rangeEnd) : 0);
  const hi = $derived(rangeStart !== null && rangeEnd !== null ? Math.max(rangeStart, rangeEnd) : 0);
</script>

{#if duration > 0}
  <div
    class="timeline"
    bind:this={trackEl}
    role="slider"
    tabindex="0"
    aria-valuemin={0}
    aria-valuemax={duration}
    aria-valuenow={currentTime}
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onkeydown={onKeyDown}
  >
    <div class="tl-trim" style="left: {pct(clip.start)}%; right: {100 - pct(clip.end)}%;"></div>

    {#if hasRange}
      <div class="tl-range" style="left: {pct(lo)}%; right: {100 - pct(hi)}%;"></div>
    {/if}

    {#each ft.keyframes as k, i}
      {@const inRange = hasRange && k.time >= lo && k.time <= hi}
      <button
        type="button"
        class="kf-marker"
        class:selected={ft.selectedKeyframeIndex === i}
        class:in-range={inRange}
        style="left: {pct(k.time)}%;"
        title={`Keyframe ${i + 1} @ ${k.time.toFixed(2)}s — right-click to delete`}
        onclick={(e) => onMarkerClick(e, k, i)}
        oncontextmenu={(e) => onMarkerContextMenu(e, i)}
      ></button>
    {/each}

    <div class="tl-cursor" style="left: {pct(currentTime)}%;"></div>
  </div>

  {#if hasRange}
    <div class="row" style="gap: 6px; margin-top: 4px; font-size: 11px;">
      <span class="muted">Selection {lo.toFixed(2)}s → {hi.toFixed(2)}s</span>
      <button onclick={() => { clip.deleteKeyframesInRange(lo, hi); clearRange(); }}>
        Delete keyframes in range
      </button>
      <button onclick={clearRange}>Clear selection</button>
    </div>
  {/if}
{/if}

<style>
  .timeline {
    position: relative;
    height: 22px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: crosshair;
    margin-top: 6px;
    user-select: none;
  }
  .timeline:focus { outline: 1px solid var(--accent); outline-offset: 1px; }
  .tl-trim {
    position: absolute;
    top: 0;
    bottom: 0;
    background: color-mix(in srgb, var(--accent) 22%, transparent);
    border-left: 2px solid var(--accent);
    border-right: 2px solid var(--accent);
    pointer-events: none;
  }
  .tl-range {
    position: absolute;
    top: 0;
    bottom: 0;
    background: color-mix(in srgb, var(--danger, #ff3b30) 28%, transparent);
    border-left: 1px dashed var(--danger, #ff3b30);
    border-right: 1px dashed var(--danger, #ff3b30);
    pointer-events: none;
  }
  .tl-cursor {
    position: absolute;
    top: -2px;
    bottom: -2px;
    width: 2px;
    background: var(--fg);
    pointer-events: none;
  }
  .kf-marker {
    position: absolute;
    top: 2px;
    width: 10px;
    height: 18px;
    margin-left: -5px;
    background: var(--danger, #ff3b30);
    border: 1px solid var(--fg);
    border-radius: 2px;
    padding: 0;
    cursor: pointer;
  }
  .kf-marker.selected {
    background: var(--accent);
  }
  .kf-marker.in-range {
    outline: 2px solid var(--fg);
    outline-offset: 1px;
  }
</style>
