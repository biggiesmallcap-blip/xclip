<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { clip } from '../stores/clip.svelte';
  import { playerRef } from './playerRef.svelte';
  import FocusOverlay from './FocusOverlay.svelte';

  let videoEl: HTMLVideoElement | null = $state(null);

  $effect(() => {
    playerRef.video = videoEl;
    return () => { playerRef.video = null; };
  });

  const src = $derived(clip.srcPath ? convertFileSrc(clip.srcPath) : '');
</script>

{#if src}
  <div class="panel">
    <div class="player-stage">
      <video bind:this={videoEl} controls {src}>
        <track kind="captions" />
      </video>
      <FocusOverlay />
    </div>
    {#if clip.probe}
      <div class="muted" style="margin-top: 6px; font-size: 12px;">
        {clip.probe.width}×{clip.probe.height} · {clip.probe.fps.toFixed(2)} fps · {clip.probe.duration.toFixed(2)}s
      </div>
    {/if}
  </div>
{/if}
