<script lang="ts">
  import UrlInput from './lib/UrlInput.svelte';
  import Player from './lib/Player.svelte';
  import TrimControls from './lib/TrimControls.svelte';
  import Timeline from './lib/Timeline.svelte';
  import ExportPanel from './lib/ExportPanel.svelte';
  import Toast from './lib/Toast.svelte';
  import { clip } from './stores/clip.svelte';
  import { playerRef } from './lib/playerRef.svelte';
  import { exportRef } from './lib/exportRef.svelte';
  import { nextKeyframeIndex, prevKeyframeIndex } from './lib/focus/interpolate';

  function onKey(e: KeyboardEvent) {
    const t = e.target;
    if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement) return;
    const v = playerRef.video;
    const ft = clip.focusTools;

    if (e.code === 'Space') {
      e.preventDefault();
      const ae = document.activeElement;
      if (ae instanceof HTMLButtonElement) ae.blur();
      if (!v || e.repeat) return;
      v.paused ? v.play() : v.pause();
      return;
    }

    if ((e.key === 'r' || e.key === 'R') && !e.ctrlKey && !e.metaKey && !e.altKey) {
      e.preventDefault();
      if (!v || e.repeat) return;
      if (!ft.enabled) return;
      if (ft.trackingMode !== 'record_path') ft.trackingMode = 'record_path';

      // Box-hold + R: hold-to-record. While the user is mouse-holding the focus box,
      // pressing R starts play+record. Releasing R (or releasing the box) stops record
      // but keeps playing — so they can press R again later for the next bit.
      if (ft.interactionHeld) {
        if (!ft.recording.active) {
          ft.recording.holdMode = true;
          ft.recording.active = true;
          v.play().catch((err) => { clip.toast = `Couldn't start playback: ${err}`; });
        }
        return;
      }

      // Plain R (no box held): toggle. Stop also pauses.
      if (ft.recording.active) {
        ft.recording.active = false;
        ft.recording.holdMode = false;
        clip.sortKeyframes();
        if (!v.paused) v.pause();
      } else {
        ft.recording.active = true;
        ft.recording.holdMode = false;
        v.play().catch((err) => { clip.toast = `Couldn't start playback: ${err}`; });
      }
      return;
    }
    if (e.key === '[' && v) { clip.start = Math.max(0, v.currentTime); return; }
    if (e.key === ']' && v) { clip.end = v.currentTime; return; }
    if (e.key === 'Enter') { void exportRef.trigger?.(); return; }

    if (ft.enabled && v) {
      if (e.key === 'k' || e.key === 'K') {
        if (e.shiftKey) {
          if (ft.selectedKeyframeIndex !== null) clip.deleteKeyframe(ft.selectedKeyframeIndex);
        } else {
          clip.addKeyframe(v.currentTime, ft.region);
          clip.sortKeyframes();
        }
        return;
      }
      if (e.key === ',') {
        const i = prevKeyframeIndex(ft.keyframes, v.currentTime);
        if (i >= 0) { v.currentTime = ft.keyframes[i].time; clip.selectKeyframe(i); }
        return;
      }
      if (e.key === '.') {
        const i = nextKeyframeIndex(ft.keyframes, v.currentTime);
        if (i >= 0) { v.currentTime = ft.keyframes[i].time; clip.selectKeyframe(i); }
        return;
      }
    }
  }

  function onKeyUp(e: KeyboardEvent) {
    const t = e.target;
    if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement) return;
    const ft = clip.focusTools;
    // Hold-mode release: stop recording but keep playback running.
    if ((e.key === 'r' || e.key === 'R') && ft.recording.holdMode && ft.recording.active) {
      e.preventDefault();
      ft.recording.active = false;
      ft.recording.holdMode = false;
      clip.sortKeyframes();
    }
  }

</script>

<svelte:document onkeydowncapture={onKey} onkeyupcapture={onKeyUp} />

<main class="app">
  <header style="display:flex; align-items:baseline; gap:8px;">
    <h1 style="margin:0; font-size:18px; letter-spacing:-0.01em;">xclip</h1>
    <span class="muted" style="font-size:12px;">paste · trim · paste into X</span>
  </header>

  <UrlInput />

  {#if clip.srcPath}
    <Player />
  {/if}

  {#if clip.probe}
    <Timeline />
    <TrimControls />
    <ExportPanel />
  {/if}

  <Toast />
</main>
