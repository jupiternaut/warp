# WarpCodexOss Windows Validation Report

Status: source-ready, host-unverified

V4 does not mark Windows acceptance complete from macOS. A real Windows x64 host must fill this report after installing and opening `WarpCodexOss`.

## Host

- Windows version:
- Architecture:
- Build date:
- Installer path:
- Installed exe path:

## Codex CLI

Run:

```powershell
.\script\windows\check_codex.ps1 -Json
```

Paste the token-safe JSON output here:

```json
{}
```

## Installation

- Start menu entry shows `WarpCodexOss`:
- Desktop shortcut shows `WarpCodexOss`:
- Process path points to installed `warp-oss.exe`:
- No Codex token files were read, copied, or written by installer:

## UI And Credits

- `/MODEL` first local entry:
- Selected model text:
- Fixed prompt: `只回复 warp-windows-local-codex-ok`
- Agent reply:
- Credits UI:
- Logs show `Local Codex AI: enabled`:
- Logs show `codex exec --json` or later app-server runner:

## Fallback Check

- `WARP_LOCAL_CODEX_AI=0` returns to Warp AI:
- UI/logs explicitly indicate Warp AI credits:

## Result

- Passed:
- Blockers:
- Notes:
