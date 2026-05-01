# xclip

Paste a video URL, trim a clip, and copy it to your clipboard — ready to paste straight into X, Discord, Slack, etc. Built as a small Tauri 2 + Svelte 5 desktop app for Windows.

The headline feature is **Focus Tools**: select a region on the video and either **reframe** the export crop to follow it, or render a **moving overlay** (highlight box, label, blur/censor, pixelate, spotlight) on top of the original framing. Tracking is manual, recorded mouse path, or auto.

## Features

- Paste a video URL → downloads via `yt-dlp`, plays in-app
- Trim with `[` / `]` (or the inputs)
- Export to **MP4** or **GIF** at three quality presets
- Auto-copies the finished file to clipboard
- **Focus Tools** for following a subject on export:
  - **Reframe Camera** — animate a crop to follow the subject; lock to 1:1, 4:5, 9:16, 16:9
  - **Highlight Box / Label Box** — moving outline (and text) over the original frame
  - **Blur / Censor / Pixelate** — moving privacy mask
  - **Spotlight** — clear region with the rest dimmed
  - Shape: Rectangle or Circle
- Three tracking methods: manual keyframes, record-mouse-path (with smoothing), and auto-track (stub on Windows)
- Timeline with keyframe markers, drag-select range, right-click delete
- Persistable Focus Tools defaults across clips
- Custom output folder + size estimate

## Quick start

If you just want the app, grab the latest `.msi` or `.exe` installer from the [Releases page](../../releases/latest).

For development setup see [`INSTALL.md`](./INSTALL.md).

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `Space` | Play / pause |
| `[` | Set trim start to current time |
| `]` | Set trim end to current time |
| `Enter` | Export |
| `K` | Add keyframe at current time (Focus Tools enabled) |
| `Shift+K` | Delete selected keyframe |
| `,` / `.` | Previous / next keyframe |
| `R` | Toggle Record Path (or click+hold the focus box, then press R for hold-to-record) |
| `Delete` / `Backspace` | Delete keyframes in timeline range selection |
| `Esc` | Clear timeline range selection |

## Tech stack

- **Frontend**: Svelte 5 (runes) + TypeScript + Vite
- **Backend**: Tauri 2 (Rust)
- **Media**: bundled `ffmpeg` + `ffprobe` + `yt-dlp` as sidecar binaries

## Project layout

```
src/                  Svelte frontend
src/lib/focus/        Focus Tools utilities (types, interpolation, coords)
src/lib/Focus*.svelte Focus Tools UI (overlay, panel)
src/stores/           App state (clip + focus state with localStorage persistence)
src/bindings/tauri.ts Tauri command bindings
src-tauri/src/        Rust backend
src-tauri/src/focus/  Focus filter graph generator
src-tauri/binaries/   Sidecar binaries (NOT committed — see binaries/README.md)
.github/workflows/    CI: builds + auto-releases on tag push
```

## License

MIT.
