# AGENTS.md

## Project goal

This repository is being modified to support a VS Code / VSIX compatible extension platform for WarpCodexOss.

Do not treat the work as a temporary MVP. Design public Rust types, IPC protocol boundaries, storage models, and feature flags toward the final target:

- install and manage VSIX packages
- integrate Open VSX
- parse VS Code extension manifests
- support declarative contribution points
- run Node-based VS Code extensions in an isolated extension host
- expose a `vscode` API compatibility shim
- support editor, workspace, command, configuration, language, LSP, debug, task, tree view, webview, theme, icon theme, snippet, and keybinding integration where feasible
- preserve Warp-native plugins as a separate compatibility path

## Architecture rules

- Keep the existing QuickJS Warp-native plugin host separate from the VS Code extension host.
- Do not replace the current plugin system with the VS Code runtime.
- Add a new VS Code extension host path instead of overloading `app/src/plugin/host/native`.
- The final runtime path should be Node-based, because real VS Code desktop extensions expect Node/CommonJS/ESM behavior and the `vscode` module.
- All new functionality must be behind a high-level runtime feature flag, for example `VsCodeExtensions`.
- Use final-shaped interfaces even when a contribution point is not fully implemented yet.
- Unsupported features should produce structured compatibility diagnostics, not silent no-ops.

## Security rules

- Never read, copy, parse, print, or commit `~/.codex/auth.json`.
- Never expose Codex OAuth tokens to extensions, logs, test fixtures, PR comments, or generated reports.
- Extension code must not silently consume Warp AI credits.
- Node extension execution must be permissioned, isolated, observable, and crash-contained.
- Treat VSIX packages as untrusted code.
- Reject path traversal during VSIX extraction.
- Webviews must have a security boundary.
- Secret storage, authentication, filesystem, terminal, network, and AI access require explicit permission modeling.

## Parallel subagent ownership

Five subagents will work in parallel. Respect file ownership to reduce conflicts.

### Subagent A: Package, registry, Open VSX

Owns:

- `crates/extension_manifest/`
- `crates/extension_store/`
- `app/src/extensions/manifest.rs`
- `app/src/extensions/store.rs`
- `app/src/extensions/open_vsx.rs`
- `app/src/extensions/registry.rs`

Does not own:

- Node extension host runtime
- `vscode` API implementation
- editor/LSP internals
- UI rendering

### Subagent B: Runtime host and IPC

Owns:

- `app/src/plugin/host/vscode/`
- `crates/extension_host_protocol/`
- `extensions/vscode-host/` or equivalent Node host package

Does not own:

- manifest parser
- extension manager UI
- editor/LSP implementation

### Subagent C: `vscode` API shim and contribution router

Owns:

- `crates/vscode_api_shim/`
- `app/src/extensions/contributions.rs`
- `app/src/extensions/activation.rs`
- `app/src/extensions/api/`

Does not own:

- Node process spawning
- VSIX extraction
- workbench UI rendering

### Subagent D: Editor, language, LSP, debug, tasks

Owns:

- `crates/lsp/src/registry.rs`
- `crates/lsp/src/dynamic.rs`
- `app/src/code/language_registry.rs`
- `app/src/code/extension_providers/`

Does not own:

- package installation
- Node host runtime
- extension manager UI

### Subagent E: Workbench UI, webviews, permissions, observability

Owns:

- `app/src/extensions/ui/`
- `app/src/extensions/permissions.rs`
- `app/src/extensions/logs.rs`
- `app/src/extensions/webview.rs`
- `app/src/settings/extensions/`

Does not own:

- manifest parsing
- LSP core refactors
- Node IPC protocol internals

## LSP persistence rules

- Do not rename existing `LSPServerType` variants.
- Do not remove existing `LanguageId` variants.
- Add registry layers around existing enums.
- Existing Rust, Go, Python, TypeScript, JavaScript, C, and C++ behavior must continue to work.

## Testing

Use repository guidance from `WARP.md`.

Run targeted tests for the area changed. Before PR:

- `cargo fmt`
- targeted `cargo test` or `cargo nextest`
- `cargo clippy --workspace --all-targets --all-features --tests -- -D warnings` when practical

## Review guidelines

Codex review should flag as P0/P1:

- token leakage
- extension code gaining Codex OAuth access
- silent Warp AI credit usage
- unpermissioned filesystem, network, terminal, or secret access
- path traversal in VSIX extraction
- extension host crashes taking down Warp
- persistence-breaking enum renames
- unbounded webview privileges
- unstructured compatibility failures
