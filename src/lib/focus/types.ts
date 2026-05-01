export type FocusMode =
  | 'off'
  | 'reframe'
  | 'box'
  | 'label'
  | 'blur'
  | 'pixelate'
  | 'spotlight';

export type TrackingMode = 'manual' | 'record_path' | 'auto';

export type AspectRatio = 'original' | '1:1' | '4:5' | '9:16' | '16:9';

export type RegionShape = 'rectangle' | 'circle';

export type Region = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type FocusKeyframe = {
  time: number;
} & Region;

export type LabelPosition = 'top' | 'bottom' | 'left' | 'right';

export type LabelStyle = {
  text: string;
  position: LabelPosition;
  textColor: string;
  backgroundColor: string;
};

export type BoxStyle = {
  strokeColor: string;
  strokeWidth: number;
  fillColor: string;
  fillOpacity: number;
  blurAmount: number;
  pixelSize: number;
  spotlightDim: number;
};

export type AutoTrackingStatus = 'idle' | 'tracking' | 'done' | 'error';

export type FocusToolsState = {
  enabled: boolean;
  mode: FocusMode;
  trackingMode: TrackingMode;
  aspectRatio: AspectRatio;
  shape: RegionShape;
  region: Region;
  keyframes: FocusKeyframe[];
  selectedKeyframeIndex: number | null;
  label: LabelStyle;
  style: BoxStyle;
  recording: {
    active: boolean;
    sampleIntervalMs: number;
    smoothing: number;
    holdMode: boolean;
  };
  // True while the user is mouse-down on the focus region in the overlay.
  interactionHeld: boolean;
  autoTrackingStatus: AutoTrackingStatus;
  autoTrackingError: string | null;
  persistDefaults: boolean;
};

export const ASPECT_RATIOS: AspectRatio[] = ['original', '1:1', '4:5', '9:16', '16:9'];

export function aspectRatioValue(r: AspectRatio, source: { width: number; height: number }): number {
  switch (r) {
    case '1:1':  return 1;
    case '4:5':  return 4 / 5;
    case '9:16': return 9 / 16;
    case '16:9': return 16 / 9;
    case 'original':
    default:     return source.width / source.height;
  }
}

export const DEFAULT_LABEL: LabelStyle = {
  text: 'Label',
  position: 'top',
  textColor: '#ffffff',
  backgroundColor: '#000000',
};

export const DEFAULT_STYLE: BoxStyle = {
  strokeColor: '#ff3b30',
  strokeWidth: 2,
  fillColor: '#ff3b30',
  fillOpacity: 0,
  blurAmount: 12,
  pixelSize: 16,
  spotlightDim: 0.6,
};

export function defaultFocusState(): FocusToolsState {
  return {
    enabled: false,
    mode: 'off',
    trackingMode: 'manual',
    aspectRatio: 'original',
    shape: 'rectangle',
    region: { x: 0, y: 0, width: 0, height: 0 },
    keyframes: [],
    selectedKeyframeIndex: null,
    label: { ...DEFAULT_LABEL },
    style: { ...DEFAULT_STYLE },
    recording: { active: false, sampleIntervalMs: 66, smoothing: 0.4, holdMode: false },
    interactionHeld: false,
    autoTrackingStatus: 'idle',
    autoTrackingError: null,
    persistDefaults: false,
  };
}

const PERSIST_KEY = 'xclip-focus-defaults-v1';

// Subset of FocusToolsState that's safe + meaningful to persist across clips.
type PersistedDefaults = {
  enabled: boolean;
  mode: FocusMode;
  trackingMode: TrackingMode;
  aspectRatio: AspectRatio;
  shape: RegionShape;
  label: LabelStyle;
  style: BoxStyle;
  smoothing: number;
  sampleIntervalMs: number;
  persistDefaults: boolean;
};

export function loadPersistedDefaults(): Partial<FocusToolsState> | null {
  try {
    const raw = typeof localStorage !== 'undefined' ? localStorage.getItem(PERSIST_KEY) : null;
    if (!raw) return null;
    const p = JSON.parse(raw) as PersistedDefaults;
    return {
      enabled: !!p.enabled,
      mode: p.mode ?? 'box',
      trackingMode: p.trackingMode ?? 'manual',
      aspectRatio: p.aspectRatio ?? 'original',
      shape: p.shape ?? 'rectangle',
      label: { ...DEFAULT_LABEL, ...(p.label ?? {}) },
      style: { ...DEFAULT_STYLE, ...(p.style ?? {}) },
      recording: {
        active: false,
        sampleIntervalMs: p.sampleIntervalMs ?? 66,
        smoothing: typeof p.smoothing === 'number' ? p.smoothing : 0.4,
        holdMode: false,
      },
      persistDefaults: !!p.persistDefaults,
    };
  } catch {
    return null;
  }
}

export function savePersistedDefaults(s: FocusToolsState) {
  try {
    if (typeof localStorage === 'undefined') return;
    if (!s.persistDefaults) {
      localStorage.removeItem(PERSIST_KEY);
      return;
    }
    const payload: PersistedDefaults = {
      enabled: s.enabled,
      mode: s.mode,
      trackingMode: s.trackingMode,
      aspectRatio: s.aspectRatio,
      shape: s.shape,
      label: s.label,
      style: s.style,
      smoothing: s.recording.smoothing,
      sampleIntervalMs: s.recording.sampleIntervalMs,
      persistDefaults: true,
    };
    localStorage.setItem(PERSIST_KEY, JSON.stringify(payload));
  } catch {
    // ignore quota / private mode errors
  }
}
