# Third-party notices

xclip bundles and depends on the projects listed below. Each remains under its own license; this file is the disclosure required when redistributing them as part of an installer or portable build.

The xclip application code itself is MIT-licensed — see [README.md](./README.md).

---

## ffmpeg + ffprobe

xclip's installers and portable build include `ffmpeg.exe` and `ffprobe.exe` from the **GPL Windows builds** published by [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds). These are unmodified upstream binaries.

- **License**: GNU General Public License v3 (GPL-3.0). Full text: <https://www.gnu.org/licenses/gpl-3.0.html>
- **Upstream**: <https://ffmpeg.org/>
- **Source code**: corresponding source is available from the upstream project at <https://ffmpeg.org/download.html> and from the build provider at <https://github.com/BtbN/FFmpeg-Builds>. We do not modify these binaries; the upstream source corresponds to the bundled version.
- **Legal page**: <https://ffmpeg.org/legal.html>

> Per GPL §6, anyone receiving a copy of xclip including these binaries has the right to receive the corresponding ffmpeg source code under the same license terms. The upstream links above satisfy this for unmodified bundles.

## yt-dlp

xclip ships an initial copy of `yt-dlp.exe` and auto-updates it from the [yt-dlp/yt-dlp](https://github.com/yt-dlp/yt-dlp) GitHub Releases at most once every 7 days.

- **License**: [The Unlicense](https://github.com/yt-dlp/yt-dlp/blob/master/LICENSE) (public domain dedication)
- **Upstream**: <https://github.com/yt-dlp/yt-dlp>

## Rust crates and JavaScript packages

xclip's source tree depends on a number of open-source libraries via [Cargo](./src-tauri/Cargo.toml) and [npm](./package.json). Their licenses (Apache-2.0, MIT, BSD-3-Clause, ISC, MPL-2.0, etc.) are listed in the respective lockfiles. None require additional in-app disclosure beyond their inclusion in source distribution.

Notable major dependencies:

- [Tauri 2](https://tauri.app/) — Apache-2.0 / MIT
- [Svelte 5](https://svelte.dev/) — MIT
- [Vite](https://vitejs.dev/) — MIT
- [reqwest](https://github.com/seanmonstar/reqwest) — Apache-2.0 / MIT (used for the yt-dlp update fetch)

## Reporting

If you believe xclip distributes a third-party component without proper attribution, please open an issue at <https://github.com/biggiesmallcap-blip/xclip/issues> and we'll fix it.
