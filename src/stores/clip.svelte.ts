import type { ProbeResult, ExportFormat, ExportQuality } from '../bindings/tauri';
import {
  defaultFocusState,
  loadPersistedDefaults,
  savePersistedDefaults,
  type FocusToolsState,
  type FocusKeyframe,
  type Region,
} from '../lib/focus/types';
import { findKeyframeAtTime, sortedKeyframes } from '../lib/focus/interpolate';

export type Status = 'idle' | 'downloading' | 'ready' | 'exporting' | 'error';

function createClip() {
  let url = $state('');
  let srcPath = $state('');
  let probe = $state<ProbeResult | null>(null);
  let start = $state(0);
  let end = $state(0);
  let status = $state<Status>('idle');
  let toast = $state('');
  let errorMsg = $state('');
  let format = $state<ExportFormat>('mp4');
  let quality = $state<ExportQuality>('balanced');
  let focusTools = $state<FocusToolsState>({ ...defaultFocusState(), ...(loadPersistedDefaults() ?? {}) });
  let lastExportPath = $state('');
  let outputDir = $state<string>(typeof localStorage !== 'undefined' ? (localStorage.getItem('xclip-output-dir') ?? '') : '');

  function resetFocus() {
    const persistOn = focusTools.persistDefaults;
    focusTools = { ...defaultFocusState(), persistDefaults: false };
    // If persistence was on, also clear the saved snapshot.
    if (persistOn) savePersistedDefaults(focusTools);
  }

  function persistDefaultsNow() {
    savePersistedDefaults(focusTools);
  }

  function addKeyframe(time: number, region: Region) {
    const existing = findKeyframeAtTime(focusTools.keyframes, time);
    const kf: FocusKeyframe = { time, ...region };
    if (existing >= 0) {
      focusTools.keyframes[existing] = kf;
      focusTools.selectedKeyframeIndex = existing;
    } else {
      focusTools.keyframes.push(kf);
      focusTools.selectedKeyframeIndex = focusTools.keyframes.length - 1;
    }
  }

  function deleteKeyframe(index: number) {
    if (index < 0 || index >= focusTools.keyframes.length) return;
    focusTools.keyframes.splice(index, 1);
    if (focusTools.selectedKeyframeIndex === index) {
      focusTools.selectedKeyframeIndex = null;
    } else if (
      focusTools.selectedKeyframeIndex !== null &&
      focusTools.selectedKeyframeIndex > index
    ) {
      focusTools.selectedKeyframeIndex -= 1;
    }
  }

  function selectKeyframe(index: number | null) {
    if (index === null) {
      focusTools.selectedKeyframeIndex = null;
      return;
    }
    if (index < 0 || index >= focusTools.keyframes.length) return;
    focusTools.selectedKeyframeIndex = index;
    const k = focusTools.keyframes[index];
    focusTools.region = { x: k.x, y: k.y, width: k.width, height: k.height };
  }

  function clearKeyframes() {
    focusTools.keyframes = [];
    focusTools.selectedKeyframeIndex = null;
  }

  function deleteKeyframesInRange(t0: number, t1: number) {
    const lo = Math.min(t0, t1);
    const hi = Math.max(t0, t1);
    const remaining = focusTools.keyframes.filter((k) => k.time < lo || k.time > hi);
    if (remaining.length !== focusTools.keyframes.length) {
      focusTools.keyframes = remaining;
      focusTools.selectedKeyframeIndex = null;
    }
  }

  function upsertKeyframeAtTime(time: number, region: Region, toleranceSec = 0.05) {
    const existing = findKeyframeAtTime(focusTools.keyframes, time, toleranceSec);
    if (existing >= 0) {
      focusTools.keyframes[existing] = { time, ...region };
    } else {
      focusTools.keyframes.push({ time, ...region });
    }
  }

  function sortKeyframes() {
    focusTools.keyframes = sortedKeyframes(focusTools.keyframes);
  }

  // Moving-average smoother. window=1 → no smoothing. Larger = smoother but laggier.
  function smoothPass(ks: FocusKeyframe[], window: number): FocusKeyframe[] {
    if (window < 2) return ks;
    const half = Math.floor(window / 2);
    const out: FocusKeyframe[] = [];
    for (let i = 0; i < ks.length; i++) {
      let sx = 0, sy = 0, sw = 0, sh = 0, n = 0;
      for (let j = Math.max(0, i - half); j <= Math.min(ks.length - 1, i + half); j++) {
        sx += ks[j].x; sy += ks[j].y; sw += ks[j].width; sh += ks[j].height; n++;
      }
      out.push({ time: ks[i].time, x: sx / n, y: sy / n, width: sw / n, height: sh / n });
    }
    return out;
  }

  function smoothKeyframes(intensity?: number) {
    let ks = sortedKeyframes(focusTools.keyframes);
    if (ks.length < 3) return;
    // Map intensity ∈ [0..1] to (window, passes). Default to slider value.
    const s = Math.max(0, Math.min(1, intensity ?? focusTools.recording.smoothing));
    if (s <= 0) return;
    // Strong by default so the user sees a real effect.
    // s=0.2  → window 7, 1 pass
    // s=0.4  → window 9, 2 passes
    // s=0.7  → window 13, 3 passes
    // s=1.0  → window 17, 4 passes
    const window = Math.max(3, Math.round(5 + s * 12) | 1); // odd window >=3
    const passes = Math.max(1, Math.round(1 + s * 3));
    for (let p = 0; p < passes; p++) {
      ks = smoothPass(ks, window);
    }
    focusTools.keyframes = ks;
  }

  return {
    get url() { return url; }, set url(v) { url = v; },
    get srcPath() { return srcPath; }, set srcPath(v) { srcPath = v; },
    get probe() { return probe; }, set probe(v) { probe = v; },
    get start() { return start; }, set start(v) { start = v; },
    get end() { return end; }, set end(v) { end = v; },
    get status() { return status; }, set status(v) { status = v; },
    get toast() { return toast; }, set toast(v) { toast = v; },
    get errorMsg() { return errorMsg; }, set errorMsg(v) { errorMsg = v; },
    get format() { return format; }, set format(v) { format = v; },
    get quality() { return quality; }, set quality(v) { quality = v; },
    get focusTools() { return focusTools; },
    get lastExportPath() { return lastExportPath; }, set lastExportPath(v) { lastExportPath = v; },
    get outputDir() { return outputDir; },
    set outputDir(v) {
      outputDir = v;
      try {
        if (typeof localStorage !== 'undefined') {
          if (v) localStorage.setItem('xclip-output-dir', v);
          else localStorage.removeItem('xclip-output-dir');
        }
      } catch {}
    },
    addKeyframe,
    deleteKeyframe,
    selectKeyframe,
    clearKeyframes,
    upsertKeyframeAtTime,
    sortKeyframes,
    smoothKeyframes,
    deleteKeyframesInRange,
    resetFocus,
    persistDefaultsNow,
    reset() {
      srcPath = '';
      probe = null;
      start = 0;
      end = 0;
      status = 'idle';
      toast = '';
      errorMsg = '';
      // Preserve focus settings between clips when the user has opted in; otherwise wipe to defaults.
      if (focusTools.persistDefaults) {
        focusTools.keyframes = [];
        focusTools.selectedKeyframeIndex = null;
        focusTools.recording = { ...focusTools.recording, active: false, holdMode: false };
        focusTools.region = { x: 0, y: 0, width: 0, height: 0 };
        focusTools.autoTrackingStatus = 'idle';
        focusTools.autoTrackingError = null;
      } else {
        focusTools = defaultFocusState();
      }
    },
  };
}

export const clip = createClip();
