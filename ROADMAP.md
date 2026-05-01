# Roadmap & Known Issues

## Known issues

- **Auto Track is a stub.** Returns a friendly "not yet supported" error. Manual keyframes and Record Path work for all modes. Adding OpenCV CSRT/KCF on Windows is non-trivial; tracked below.
- **Filter expressions are decimated.** Path recordings denser than 30 keyframes are uniformly downsampled before being baked into the ffmpeg filter graph. With 15 Hz path recording over 10+ seconds you won't notice, but very long recordings lose fine timing detail.
- **Spotlight has hard edges.** No soft falloff. Could be improved with a `geq`-based gradient mask, at a frame-rate cost.
- **Pixelate / Blur on circle shape** uses a small bounding-box `geq` for the alpha mask. Performance is fine, but very large regions on long clips can take several seconds during the intermediate render.
- **No undo.** Clear / Delete / Smooth are destructive. Workaround: don't enable Persist Defaults if you're experimenting.
- **GIF size is 3–10× the equivalent MP4.** Inherent to the format (no temporal compression, no CRF). The estimate accounts for this.
- **WebView2 SmartScreen warning** on first launch — the installer is unsigned. Code-signing is on the wishlist.

## Roadmap

### Near-term polish

- [ ] **Recording session boundaries.** Visualize gaps between R-press sessions on the timeline as separate tracks instead of one merged stream.
- [ ] **Right-click context menu on the focus overlay** (delete keyframe at this time, set as the resting region, etc).
- [ ] **Soft-edge spotlight** via a radial gradient `geq` mask.
- [ ] **Per-keyframe size editing.** Currently `width`/`height` is per-keyframe in the data model but the UI doesn't expose distinct sizes well — only the most-recent drag size sticks.
- [ ] **Undo/redo** for keyframe edits (Ctrl+Z / Ctrl+Y).
- [ ] **Drag a clip file directly onto the window** to load it (alongside URL paste).
- [ ] **Configurable max keyframes** for export (currently hardcoded at 30).

### Auto Track

- [ ] Wire up `opencv-rust` (CSRT or KCF) for box-tracking on the trim window.
- [ ] Render keyframe samples at the configured `sampleInterval` (default 0.1 s).
- [ ] Surface tracker confidence in the UI; flag low-confidence segments for manual cleanup.
- [ ] Fall back gracefully if OpenCV isn't installed (current stub behavior).

### Engine

- [ ] **Tests.** Unit test the Rust `piecewise_expr` and `interpolateRegion` (TS) — both are pure and easy to cover. End-to-end tests for export filter graphs.
- [ ] **Faster intermediate renders** for the focus pipeline. Today: full re-encode at `ultrafast crf 18`. Could try `-c:v libx264 -tune fastdecode` or `prores_ks` for the intermediate.
- [ ] **Macros / scripted exports** — apply the same Focus Tools setup to a batch of clips.
- [ ] **Cross-platform.** macOS + Linux builds. Currently Windows-only because of the clipboard implementation and MSVC sidecar naming.

### UX

- [ ] Code-signing for Windows installers (kills the SmartScreen popup).
- [ ] Auto-update via Tauri updater.
- [ ] In-app changelog / first-run hints.
- [ ] Customizable keyboard shortcuts.
- [ ] Light theme toggle.

## Bugs

File new bugs at: https://github.com/biggiesmallcap-blip/xclip/issues

When reporting a focus/export bug, please include:

1. The clip duration, format, quality preset, and focus mode/shape used
2. Number of keyframes (visible in the Focus Tools header)
3. The contents of `E:\xclip\downloads\last-failed-filter.txt` if the export failed
4. The contents of `last-failed-cmd.txt` if present
