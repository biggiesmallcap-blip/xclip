# Installation

## End users

Download the latest installer from the [Releases page](../../releases/latest):

- `xclip_<version>_x64-setup.exe` (NSIS, smaller, recommended)
- `xclip_<version>_x64_en-US.msi` (MSI, for managed environments)

Run the installer. The app self-contains `ffmpeg`, `ffprobe`, and `yt-dlp` — no extra dependencies.

> **Windows SmartScreen** may flag the unsigned binary on first run. Click *More info* → *Run anyway*.

## Developers

### Prerequisites

| Tool | Version |
|------|---------|
| Node.js | ≥ 18 |
| npm | ≥ 9 |
| Rust | stable, ≥ 1.78 (install via [rustup](https://rustup.rs)) |
| MSVC build tools | "Desktop development with C++" workload from Visual Studio Installer |
| WebView2 Runtime | usually pre-installed on Windows 11 |

### 1. Clone & install

```powershell
git clone https://github.com/biggiesmallcap-blip/xclip.git
cd xclip
npm install
```

### 2. Populate sidecar binaries

The build expects three executables under `src-tauri/binaries/`. They are NOT committed.

```powershell
mkdir src-tauri\binaries -Force | Out-Null

# yt-dlp
Invoke-WebRequest `
  -Uri https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe `
  -OutFile src-tauri\binaries\yt-dlp-x86_64-pc-windows-msvc.exe

# ffmpeg + ffprobe (BtbN gpl latest win64)
$tmp = New-TemporaryFile
Invoke-WebRequest `
  -Uri https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip `
  -OutFile "$tmp.zip"
Expand-Archive -Path "$tmp.zip" -DestinationPath "$tmp-extracted" -Force
$bin = Get-ChildItem "$tmp-extracted" -Recurse -Filter "bin" | Select-Object -First 1
Copy-Item "$($bin.FullName)\ffmpeg.exe" src-tauri\binaries\ffmpeg-x86_64-pc-windows-msvc.exe
Copy-Item "$($bin.FullName)\ffprobe.exe" src-tauri\binaries\ffprobe-x86_64-pc-windows-msvc.exe
Remove-Item "$tmp.zip", "$tmp-extracted" -Recurse -Force
```

The CI workflow does the same automatically (see `.github/workflows/release.yml`).

### 3. Run

```powershell
npm run tauri dev
```

First build takes 1–3 minutes (Rust compilation + Tauri toolchain). Subsequent dev runs are fast — Vite HMR for the frontend, incremental Rust for the backend.

### 4. Build a release locally

```powershell
npm run tauri build
```

Outputs land in `src-tauri/target/release/bundle/`:

- `nsis/xclip_<version>_x64-setup.exe`
- `msi/xclip_<version>_x64_en-US.msi`

### 5. Type-check (optional, fast)

```powershell
npm run check               # Svelte + TS
cargo check --manifest-path src-tauri/Cargo.toml   # Rust
```

## Troubleshooting

**`ffmpeg sidecar not available`** — `src-tauri/binaries/` is missing the `*-x86_64-pc-windows-msvc.exe` files. Re-run step 2.

**`Failed to start ffmpeg`** during export — antivirus may be quarantining the bundled `ffmpeg.exe`. Whitelist the install directory.

**Export shows `last-failed-filter.txt` path on failure** — the focus filter graph is preserved at that path for inspection. Open it and check it parses with `ffmpeg -filter_complex_script <file>` manually.

**Webview missing** on Windows 10 — install the [WebView2 Evergreen Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).
