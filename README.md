# xclip

> Desktop clipper that turns social video URLs into pasteable MP4/GIF clips in seconds.

Paste a video URL, trim a clip, and copy it to your clipboard — ready to paste straight into X, Discord, Slack, etc. Built as a small Tauri 2 + Svelte 5 desktop app for Windows.

The headline feature is **Focus Tools**: select a region on the video and either **reframe** the export crop to follow it, or render a **moving overlay** (highlight box, label, blur/censor, pixelate, spotlight) on top of the original framing. Tracking is manual, recorded mouse path, or auto.

## Features

- Paste a video URL — TikTok, Instagram, YouTube, X, Reddit — downloads via `yt-dlp`, plays in-app
- **`yt-dlp` self-updates** weekly from its official GitHub releases, with validation, rollback, and a manual *Check yt-dlp* button. Bundled copy is the immutable fallback.
- Trim with `[` / `]` (or the inputs)
- Export to **MP4** or **GIF** at three quality presets
- Auto-copies the finished file to clipboard; **Copy Last** re-copies the previous export
- Custom output folder + live output-size estimate
- **Focus Tools** for following a subject on export:
  - **Reframe Camera** — animate a crop to follow the subject; lock to 1:1, 4:5, 9:16, 16:9
  - **Highlight Box / Label Box** — moving outline (and text) over the original frame
  - **Blur / Censor / Pixelate** — moving privacy mask
  - **Spotlight** — clear region with the rest dimmed
  - Shape: Rectangle or Circle (export honors the shape via `geq` alpha-mask)
- Three tracking methods: manual keyframes, record-mouse-path (with EMA smoothing + post-record Smooth pass), auto-track (stub returning a friendly "not supported" error)
- Hold-to-record: click+hold the focus box, press **R**, drag to trace, release to stop while video keeps playing
- Timeline with keyframe markers, drag-select range, right-click delete
- Persistable Focus Tools defaults across clips, with a Reset button

## Quick start

Grab the latest from the [Releases page](../../releases/latest):

- **Portable** (`*-portable.zip`) — unzip anywhere, run `xclip.exe`. No installer, no SmartScreen signature prompt. Recommended for most users.
- **Installer** (`*-setup.exe` or `.msi`) — if you want Start Menu integration / uninstaller.

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
- **Media**: bundled `ffmpeg` + `ffprobe` (BtbN GPL build) + `yt-dlp` as sidecar binaries
- **HTTP**: `reqwest` (rustls) for the yt-dlp updater

See [NOTICES.md](./NOTICES.md) for third-party licenses.

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

xclip itself is **MIT**. Bundled third-party binaries (ffmpeg, ffprobe, yt-dlp) keep their own licenses — see [NOTICES.md](./NOTICES.md) for full disclosure.

> yt-dlp self-updates from its [official GitHub releases](https://github.com/yt-dlp/yt-dlp/releases) at most once per 7 days. The bundled copy is the immutable fallback — manual update available via the **Check yt-dlp** button next to the URL field.
