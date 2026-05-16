## ADDED Requirements

### Requirement: Windows local Codex packaging mode
The Windows packaging scripts SHALL support a local Codex packaging mode that produces a `WarpCodexOss` installer while keeping the OSS release channel identity.

#### Scenario: Check-only packaging
- **WHEN** a Windows x64 host runs `script/windows/bundle.ps1 -CHANNEL oss -ARCH x64 -LOCAL_CODEX -CHECK_ONLY`
- **THEN** the script validates required build tools and reports whether Codex CLI readiness can be checked

#### Scenario: Installer output
- **WHEN** a Windows x64 host runs `script/windows/bundle.ps1 -CHANNEL oss -ARCH x64 -LOCAL_CODEX`
- **THEN** the output installer is named `WarpCodexOssSetup.exe`

### Requirement: Windows Codex OAuth stays external
The Windows package SHALL NOT bundle Codex CLI or copy OAuth tokens; the installed app SHALL rely on the Windows user's separately installed and logged-in `codex.exe`.

#### Scenario: Missing Windows Codex login
- **WHEN** the installed app cannot validate `codex login status`
- **THEN** it shows a local Codex login/install error and does not read token files

### Requirement: Windows validation requires a real host
The project SHALL distinguish source/script readiness from completed Windows installation acceptance.

#### Scenario: Running on macOS
- **WHEN** the developer is on macOS
- **THEN** Windows packaging can be prepared but final installer, process-path, UI, and Agent acceptance remain unverified
