<script lang="ts">
  import { clip } from '../stores/clip.svelte';
  import { playerRef } from './playerRef.svelte';

  function setStart() {
    if (playerRef.video) clip.start = Math.max(0, playerRef.video.currentTime);
  }
  function setEnd() {
    if (playerRef.video) clip.end = playerRef.video.currentTime;
  }

  const delta = $derived(Math.max(0, clip.end - clip.start));
</script>

<div class="panel">
  <div class="row" style="gap: 12px; flex-wrap: wrap;">
    <label class="row">
      <span class="muted" style="min-width: 36px;">Start</span>
      <input type="number" min="0" step="0.1" bind:value={clip.start} style="width: 90px;" />
    </label>
    <button onclick={setStart} title="[ key">Set Start</button>

    <label class="row">
      <span class="muted" style="min-width: 28px;">End</span>
      <input type="number" min="0" step="0.1" bind:value={clip.end} style="width: 90px;" />
    </label>
    <button onclick={setEnd} title="] key">Set End</button>

    <span class="muted" style="margin-left: auto;">
      Δ {delta.toFixed(2)}s
    </span>
  </div>
</div>
