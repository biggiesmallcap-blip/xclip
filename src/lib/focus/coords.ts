import type { Region } from './types';

export type Size = { width: number; height: number };

export const MIN_REGION_WIDTH = 24;

export function videoDisplayRect(videoEl: HTMLVideoElement, source: Size): {
  left: number;
  top: number;
  width: number;
  height: number;
} {
  const rect = videoEl.getBoundingClientRect();
  const elAspect = rect.width / rect.height;
  const srcAspect = source.width / source.height;
  let w = rect.width;
  let h = rect.height;
  let left = rect.left;
  let top = rect.top;
  if (elAspect > srcAspect) {
    w = rect.height * srcAspect;
    left = rect.left + (rect.width - w) / 2;
  } else if (elAspect < srcAspect) {
    h = rect.width / srcAspect;
    top = rect.top + (rect.height - h) / 2;
  }
  return { left, top, width: w, height: h };
}

export function cssToSource(
  cssRegion: Region,
  cssOrigin: { left: number; top: number; width: number; height: number },
  source: Size,
): Region {
  const sx = source.width / cssOrigin.width;
  const sy = source.height / cssOrigin.height;
  return {
    x: (cssRegion.x - cssOrigin.left) * sx,
    y: (cssRegion.y - cssOrigin.top) * sy,
    width: cssRegion.width * sx,
    height: cssRegion.height * sy,
  };
}

export function sourceToCss(
  region: Region,
  cssOrigin: { left: number; top: number; width: number; height: number },
  source: Size,
): Region {
  const sx = cssOrigin.width / source.width;
  const sy = cssOrigin.height / source.height;
  return {
    x: cssOrigin.left + region.x * sx,
    y: cssOrigin.top + region.y * sy,
    width: region.width * sx,
    height: region.height * sy,
  };
}

export function clampRegion(
  region: Region,
  source: Size,
  opts: { minWidth?: number; lockAspect?: number | null } = {},
): Region {
  const minWidth = opts.minWidth ?? MIN_REGION_WIDTH;
  const lockAspect = opts.lockAspect ?? null;

  let { x, y, width, height } = region;

  width = Math.max(minWidth, Math.min(width, source.width));
  if (lockAspect && lockAspect > 0) {
    height = width / lockAspect;
  } else {
    const minHeight = Math.max(1, Math.round(minWidth * (height / Math.max(width, 1))));
    height = Math.max(minHeight, Math.min(height, source.height));
  }

  if (width > source.width) {
    width = source.width;
    if (lockAspect) height = width / lockAspect;
  }
  if (height > source.height) {
    height = source.height;
    if (lockAspect) width = height * lockAspect;
  }

  x = Math.max(0, Math.min(x, source.width - width));
  y = Math.max(0, Math.min(y, source.height - height));

  return { x, y, width, height };
}

export function defaultRegion(source: Size, lockAspect: number | null = null): Region {
  let width = Math.max(MIN_REGION_WIDTH, Math.round(source.width / 3));
  let height = lockAspect ? width / lockAspect : Math.round(source.height / 3);
  if (height > source.height) {
    height = source.height;
    if (lockAspect) width = height * lockAspect;
  }
  return clampRegion(
    {
      x: (source.width - width) / 2,
      y: (source.height - height) / 2,
      width,
      height,
    },
    source,
    { lockAspect },
  );
}
