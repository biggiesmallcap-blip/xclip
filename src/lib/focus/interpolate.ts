import type { FocusKeyframe, Region } from './types';

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

export function sortedKeyframes(kfs: FocusKeyframe[]): FocusKeyframe[] {
  return [...kfs].sort((a, b) => a.time - b.time);
}

export function interpolateRegion(kfs: FocusKeyframe[], time: number): Region | null {
  if (kfs.length === 0) return null;
  const sorted = sortedKeyframes(kfs);
  if (time <= sorted[0].time) {
    const k = sorted[0];
    return { x: k.x, y: k.y, width: k.width, height: k.height };
  }
  const last = sorted[sorted.length - 1];
  if (time >= last.time) {
    return { x: last.x, y: last.y, width: last.width, height: last.height };
  }
  for (let i = 0; i < sorted.length - 1; i++) {
    const a = sorted[i];
    const b = sorted[i + 1];
    if (time >= a.time && time <= b.time) {
      const span = b.time - a.time;
      const t = span === 0 ? 0 : (time - a.time) / span;
      return {
        x: lerp(a.x, b.x, t),
        y: lerp(a.y, b.y, t),
        width: lerp(a.width, b.width, t),
        height: lerp(a.height, b.height, t),
      };
    }
  }
  return { x: last.x, y: last.y, width: last.width, height: last.height };
}

export function findKeyframeAtTime(
  kfs: FocusKeyframe[],
  time: number,
  toleranceSec = 0.05,
): number {
  for (let i = 0; i < kfs.length; i++) {
    if (Math.abs(kfs[i].time - time) <= toleranceSec) return i;
  }
  return -1;
}

export function nextKeyframeIndex(kfs: FocusKeyframe[], time: number): number {
  const sorted = sortedKeyframes(kfs);
  if (sorted.length === 0) return -1;
  for (let i = 0; i < sorted.length; i++) {
    if (sorted[i].time > time + 1e-4) {
      return kfs.indexOf(sorted[i]);
    }
  }
  return -1;
}

export function prevKeyframeIndex(kfs: FocusKeyframe[], time: number): number {
  const sorted = sortedKeyframes(kfs);
  if (sorted.length === 0) return -1;
  for (let i = sorted.length - 1; i >= 0; i--) {
    if (sorted[i].time < time - 1e-4) {
      return kfs.indexOf(sorted[i]);
    }
  }
  return -1;
}
