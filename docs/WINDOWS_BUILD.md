# Building Luna Telepresence on Windows

Luna Telepresence uses Tauri, Rust, Whisper, and a bundled Llama sidecar. End users do not need these build tools; they are required only on the machine producing the application or installer.

## Required build tools

- Rust stable with the `x86_64-pc-windows-msvc` host
- Visual Studio Build Tools with Desktop development with C++
- LLVM/Clang 18.x
- CMake
- Node.js and pnpm

`LIBCLANG_PATH` should point to `C:\Program Files\LLVM\bin`. LLVM 22 is not compatible with the bindgen version used by `whisper-rs 0.13.2` in this release.

## Build the application without an installer

From the repository root:

```powershell
.\scripts\build-windows.ps1 -Mode App
```

The release executable is written to:

```text
target\release\luna-telepresence.exe
```

## Build an unsigned installer

NSIS is the primary Windows installer format:

```powershell
.\scripts\build-windows.ps1 -Mode Nsis
```

The installer is written beneath:

```text
target\release\bundle\nsis\
```

Use `-Mode Msi` for MSI or `-Mode All` for both formats. These local builds pass `--no-sign`; external distribution requires a Luna-owned code-signing certificate.

## Development mode

The standard command now builds and places the required `llama-helper` sidecar before starting Tauri:

```powershell
cd frontend
pnpm tauri:dev
```

The first native build can take several minutes. Subsequent builds reuse Cargo artifacts.

## Brand assets

The Windows package uses the approved LunaOS desktop assets copied into the repository:

- `frontend/src-tauri/icons/icon.png` — application and tray icon
- `frontend/src-tauri/icons/app_icon.ico` — Windows executable and installer icon

Their source masters are `icons/desktop/lunaos-app-icon-1024.png` and
`icons/desktop/lunaos-desktop.ico` from the LunaOS logo kit. The copied assets
are committed so builds do not depend on an external local folder.
