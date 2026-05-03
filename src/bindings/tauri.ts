import { invoke } from '@tauri-apps/api/core';
import type { Region, FocusKeyframe } from '../lib/focus/types';

export type ProbeResult = {
  duration: number;
  width: number;
  height: number;
  fps: number;
};

export type ExportFormat = 'mp4' | 'gif';
export type ExportQuality = 'small' | 'balanced' | 'hq';

export type FocusToolsExport = {
  enabled: boolean;
  mode: string;
  aspectRatio: string;
  shape: string;
  region: Region;
  keyframes: FocusKeyframe[];
  label?: {
    text: string;
    position: string;
    textColor: string;
    backgroundColor: string;
  };
  style?: {
    strokeColor: string;
    strokeWidth: number;
    fillColor: string;
    fillOpacity: number;
    blurAmount: number;
    pixelSize: number;
    spotlightDim: number;
  };
};

export const download = (url: string) =>
  invoke<{ path: string }>('download', { url });

export const probe = (path: string) =>
  invoke<ProbeResult>('probe', { path });

export const exportClip = (args: {
  inputPath: string;
  start: number;
  end: number;
  format: ExportFormat;
  quality: ExportQuality;
  focusTools?: FocusToolsExport;
  outputDir?: string;
}) => invoke<{ outputPath: string }>('export_clip', { args });

export const defaultOutputDir = () => invoke<string>('default_output_dir');
export const revealInFolder = (path: string) => invoke<void>('reveal_in_folder', { path });

export type YtDlpUpdateStatus =
  | { kind: 'skipped'; currentVersion: string | null }
  | { kind: 'upToDate'; version: string }
  | { kind: 'updated'; from: string | null; to: string }
  | { kind: 'failed'; reason: string };

export const ytDlpCheckUpdate = (force = false) =>
  invoke<YtDlpUpdateStatus>('yt_dlp_check_update', { force });

export const ytDlpUpdateNow = () =>
  invoke<YtDlpUpdateStatus>('yt_dlp_update_now');

export const copyFile = (path: string) =>
  invoke<void>('copy_file_to_clipboard', { path });

export type AutoTrackResult = {
  keyframes: Array<FocusKeyframe & { confidence?: number }>;
};

export const autoTrack = (args: {
  inputPath: string;
  start: number;
  end: number;
  targetBox: Region;
  sampleInterval: number;
}) => invoke<AutoTrackResult>('auto_track', { args });
