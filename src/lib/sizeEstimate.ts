import type { ExportFormat, ExportQuality } from '../bindings/tauri';

// Rough average bitrates from typical encodes. These are heuristics — actual size depends
// on motion, content complexity, and whether focus filters are active.
const MP4_KBPS: Record<ExportQuality, number> = {
  small: 700,
  balanced: 1600,
  hq: 3800,
};

const GIF_KBPS: Record<ExportQuality, number> = {
  small: 3500,
  balanced: 6500,
  hq: 11000,
};

/** Returns estimated file size in bytes for the given trim duration / format / quality. */
export function estimateExportBytes(
  durationSec: number,
  format: ExportFormat,
  quality: ExportQuality,
): number {
  if (durationSec <= 0) return 0;
  const kbps = format === 'gif' ? GIF_KBPS[quality] : MP4_KBPS[quality];
  return Math.round((kbps * 1000 * durationSec) / 8);
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
