# xclip sidecar binaries

Tauri bundles the executables in this folder into the final installer. **They are not committed to git** — each developer or CI step must populate them before `npm run tauri dev` or `npm run tauri build`.

## Required files

For Windows x86_64 builds, this folder must contain:

```
yt-dlp-x86_64-pc-windows-msvc.exe
ffmpeg-x86_64-pc-windows-msvc.exe
ffprobe-x86_64-pc-windows-msvc.exe
```

The `-x86_64-pc-windows-msvc` suffix is required by Tauri's sidecar resolution. The names without the suffix (`binaries/yt-dlp`, etc.) are what `tauri.conf.json` lists under `bundle.externalBin`; Tauri appends the target triple at build time.

## Where to download

### yt-dlp

Grab the latest `yt-dlp.exe` from the releases page:

- https://github.com/yt-dlp/yt-dlp/releases/latest

Rename `yt-dlp.exe` → `yt-dlp-x86_64-pc-windows-msvc.exe`.

### ffmpeg + ffprobe

Use the BtbN Windows builds (gpl, latest, win64):

- https://github.com/BtbN/FFmpeg-Builds/releases/latest

Download `ffmpeg-master-latest-win64-gpl.zip`, extract, then take only the two binaries from the `bin/` folder:

- `ffmpeg.exe` → rename to `ffmpeg-x86_64-pc-windows-msvc.exe`
- `ffprobe.exe` → rename to `ffprobe-x86_64-pc-windows-msvc.exe`

## Quick PowerShell setup

From `e:\xclip\src-tauri\binaries`:

```powershell
# yt-dlp
Invoke-WebRequest -Uri "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe" `
                  -OutFile "yt-dlp-x86_64-pc-windows-msvc.exe"

# ffmpeg + ffprobe (BtbN gpl)
Invoke-WebRequest -Uri "https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip" `
                  -OutFile "ffmpeg.zip"
Expand-Archive -Path "ffmpeg.zip" -DestinationPath "ffmpeg-tmp"
Move-Item "ffmpeg-tmp\*\bin\ffmpeg.exe"  "ffmpeg-x86_64-pc-windows-msvc.exe"
Move-Item "ffmpeg-tmp\*\bin\ffprobe.exe" "ffprobe-x86_64-pc-windows-msvc.exe"
Remove-Item -Recurse -Force ffmpeg-tmp, ffmpeg.zip
```
