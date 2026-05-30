# WarpCodexOss VSIX Plugin System Research Report

Date: 2026-05-30

## Executive Summary

The target is feasible, but only if it is staged as a compatibility ladder instead of a one-shot "run every VS Code extension" rewrite.

Recommended product direction:

1. Build `VSIX Lite` first: install `.vsix`, parse `package.json`, support declarative contribution points, expose them through Warp-native UI and settings.
2. Add a dynamic extension registry and plugin manager: local install, enable/disable, uninstall, logs, compatibility labels, and Open VSX download support.
3. Add a Node-based extension host only after the declarative layer works.
4. Treat full VS Code API parity as a long-term compatibility project, not V1.

The current WarpCodexOss codebase already has useful foundations:

- a plugin host process and IPC channel,
- a QuickJS plugin runner,
- LSP infrastructure,
- a local code editor,
- MCP and agent infrastructure,
- a Codex-local AI bridge.

However, the current plugin API is far smaller than VS Code's extension API. Today it is closer to a command-completion plugin system than a full application-extension runtime.

## Definitions

This report interprets `vlsx` as `VSIX`, the common package format for VS Code extensions. If the intended target is a different plugin format, this report should be adjusted.

`VSIX compatibility` has multiple levels:

- Package compatibility: install and inspect `.vsix`.
- Declarative compatibility: consume `package.json` `contributes`.
- Runtime compatibility: execute extension JavaScript/TypeScript.
- API compatibility: provide a `vscode` API shim.
- Workbench parity: implement enough UI and editor surface for complex extensions.

Only the first two are realistic for V1.

## Current WarpCodexOss Evidence

### Existing Plugin Host

Warp already has a native plugin host process and app-to-plugin IPC bootstrap.

Relevant files:

- `app/src/plugin/mod.rs`
- `app/src/plugin/app/mod.rs`
- `app/src/plugin/host/native/mod.rs`
- `app/src/plugin/host/native/runner.rs`
- `app/src/plugin/host/native/js_api/mod.rs`

Important evidence:

- `app/src/plugin/host/native/mod.rs` spawns plugin runners and loads plugins from a local directory.
- The hardcoded plugin path currently assumes `~/.warp/plugins`.
- A plugin directory is valid only if it contains `main.js`.
- `app/src/plugin/host/native/runner.rs` uses `rquickjs` / QuickJS, compiles a plugin module, resolves `activate()`, and calls it with a Warp API object.
- `app/src/plugin/host/native/js_api/mod.rs` says the plugin API currently contains a single `completions` namespace for registering command signatures.

Implication:

The current plugin host is a useful skeleton for isolation and message routing, but it is not a VS Code extension host. VS Code desktop extensions normally expect Node.js, CommonJS/ESM loading behavior, and the `vscode` module API.

### Existing LSP Infrastructure

Warp has an LSP layer, but it is currently static and enum-driven.

Relevant files:

- `crates/lsp/src/config.rs`
- `crates/lsp/src/supported_servers.rs`
- `crates/lsp/src/manager.rs`
- `app/src/code/language_server_extension.rs`
- `app/src/code/global_buffer_model.rs`

Important evidence:

- `LanguageId` is a fixed Rust enum for Rust, Go, Python, TypeScript, JavaScript, C, and C++.
- `LSPServerType` is also a fixed enum: `RustAnalyzer`, `GoPls`, `Pyright`, `TypeScriptLanguageServer`, and `Clangd`.
- File extension to language mapping is hardcoded.
- LSP server command construction is hardcoded per enum variant.

Implication:

VSIX language extensions cannot be supported properly until language definitions and LSP server definitions become data-driven.

### Existing Editor Surface

Warp has a local code editor, file tree, diff viewer, hover/diagnostic rendering, find references, and global buffer model.

Relevant files:

- `app/src/code/mod.rs`
- `app/src/code/local_code_editor.rs`
- `app/src/code/editor/*`
- `app/src/code/file_tree/*`
- `app/src/code/diff_viewer.rs`
- `app/src/code/inline_diff.rs`

Implication:

There is enough editor surface for a controlled extension API, but not enough for arbitrary VS Code workbench APIs such as TreeView, WebviewPanel, SCM, Debug, Notebooks, Custom Editors, and rich terminal APIs.

### Existing Codex and Agent Foundation

WarpCodexOss already routes local generative AI through Codex OAuth.

Relevant files:

- `app/src/ai/local_codex.rs`
- `app/src/ai/agent_sdk/driver/harness/codex.rs`
- `openspec/changes/warp-codex-native-bridge-v3/*`
- `openspec/changes/warp-codex-app-server-bridge-v4/*`
- `openspec/changes/warp-codex-app-server-transport-v5/*`
- `openspec/changes/warp-codex-native-display-cards-v6/*`
- `openspec/changes/warp-codex-display-only-diff-cards-v7/*`
- `openspec/changes/warp-codex-local-completion-v8/*`

Implication:

The plugin system should be designed so local Codex can consume extension-provided capabilities, but V1 should avoid letting arbitrary extensions silently spend Warp credits or leak OAuth tokens.

## VSIX / VS Code Model

Official VS Code docs describe extension execution in extension hosts. Desktop VS Code has a local Node.js extension host and can also have web and remote extension hosts. Node extensions need a `main` entry file; web extensions need a `browser` entry file.

Sources:

- VS Code Extension Host: https://code.visualstudio.com/api/advanced-topics/extension-host
- VS Code Extension Manifest: https://code.visualstudio.com/api/references/extension-manifest
- VS Code Activation Events: https://code.visualstudio.com/api/references/activation-events
- VS Code Contribution Points: https://code.visualstudio.com/api/references/contribution-points
- VS Code Extension Capabilities: https://code.visualstudio.com/api/extension-capabilities/overview
- Open VSX Registry: https://open-vsx.org/

Core concepts that WarpCodexOss must emulate or selectively reject:

- `package.json` manifest.
- `activationEvents`.
- `contributes`.
- `main` / `browser` entry.
- `extensionKind`.
- `vscode` API module.
- extension context storage.
- commands and command palette.
- configuration.
- themes and icons.
- snippets.
- languages and grammars.
- LSP / language client.
- webviews.
- workspace filesystem.
- terminal API.
- secrets/authentication.
- tasks/debug/SCM/notebooks.

## Feasibility Matrix

| Capability | V1 feasibility | Notes |
|---|---:|---|
| Install local `.vsix` | High | `.vsix` is a zip-like artifact; extract and parse manifest. |
| Parse extension `package.json` | High | Need manifest model and validation. |
| Extension list UI | High | Settings page can show installed/enabled/disabled. |
| Open VSX search/download | High | Network + registry client + version selection. |
| Themes | Medium | Warp theme model differs from VS Code themes; can support a subset or convert. |
| File icon themes | Medium | Needs file tree icon mapping integration. |
| Snippets | Medium | Needs editor insertion/completion surface. |
| Language declarations | Medium | Requires data-driven language registry. |
| Grammars/TextMate | Medium-low | Warp uses tree-sitter/arborium path today; TextMate support is separate work. |
| Dynamic LSP definitions | Medium | Refactor enum-based LSP to registry-based configs. |
| Commands | Medium | Need command registry + command palette integration. |
| Configuration | Medium | Need per-extension settings namespace. |
| Node extension host | Medium-low | Requires Node runtime process, module loading, IPC, lifecycle. |
| `vscode` API shim | Low to medium | Can implement a small subset; full API is large. |
| Webviews | Low | Requires embedded webview surface and security model. |
| Terminal API parity | Low | Warp's terminal model is different from VS Code terminal API. |
| Debug adapters | Low | DAP/UI/launch configs are large. |
| SCM providers | Low | Needs source-control UI model. |
| Chat/AI extension APIs | Low | VS Code's AI APIs are fast-moving and complex. |
| Full VS Code extension compatibility | Very low for V1 | Treat as long-term. |

## Recommended Architecture

### Layer 1: Extension Package Store

New crate or module:

- `crates/extension_manifest`
- or `app/src/extensions/manifest.rs`

Responsibilities:

- install local `.vsix`,
- extract to `~/.warp-oss/extensions/<publisher>.<name>-<version>/`,
- parse `package.json`,
- validate schema subset,
- record install metadata,
- compute compatibility status.

Suggested data model:

```rust
struct InstalledExtension {
    id: ExtensionId,
    publisher: String,
    name: String,
    version: String,
    root: PathBuf,
    manifest: ExtensionManifest,
    enabled: bool,
    compatibility: CompatibilityReport,
}
```

### Layer 2: Declarative Contribution Router

New module:

- `app/src/extensions/contributions.rs`

Responsibilities:

- load supported contribution points,
- reject unsupported contribution points clearly,
- expose a compatibility report in UI,
- install contribution artifacts into Warp-native registries.

V1 supported contribution points:

- `contributes.commands`
- `contributes.configuration`
- `contributes.languages`
- `contributes.snippets`
- `contributes.themes`
- `contributes.iconThemes`

Unsupported in V1:

- `views`
- `viewsContainers`
- `menus`
- `keybindings`
- `debuggers`
- `taskDefinitions`
- `grammars` unless a conversion path is built
- `webviews`
- `customEditors`
- `notebooks`
- `authentication`
- AI/chat APIs

### Layer 3: Warp Extension Manager UI

New UI surface:

- `Settings > Extensions`

V1 controls:

- install from `.vsix`,
- install from Open VSX URL,
- enable/disable,
- uninstall,
- show compatibility report,
- show logs.

### Layer 4: Dynamic Language and LSP Registry

Current state is enum-based. The refactor should introduce data-driven registrations.

Suggested shape:

```rust
struct LanguageDefinition {
    id: String,
    aliases: Vec<String>,
    extensions: Vec<String>,
    filenames: Vec<String>,
    first_line: Vec<String>,
}

struct LanguageServerDefinition {
    id: String,
    command: PathBuf,
    args: Vec<String>,
    languages: Vec<String>,
}
```

This would eventually replace or wrap:

- `LanguageId`
- `LSPServerType`
- hardcoded `from_path`
- hardcoded `binary_name`
- hardcoded language-to-server mapping

### Layer 5: Runtime Extension Host

Only after V1 works.

Options:

1. Keep QuickJS for Warp-native plugins and add a separate Node extension host for VSIX runtime extensions.
2. Replace QuickJS with Node for all plugin code.
3. Keep both:
   - Warp-native plugins: QuickJS, low-risk, controlled API.
   - VSIX runtime plugins: Node, gated by compatibility and permissions.

Recommended: keep both.

Node extension host responsibilities:

- load extension `main`,
- provide `require("vscode")` shim,
- manage activation/deactivation,
- expose IPC-backed services to Warp,
- enforce permissions,
- isolate crashes.

### Layer 6: Security and Permission Model

This is mandatory before running arbitrary extension code.

Required controls:

- extension enable/disable,
- explicit workspace trust,
- filesystem access permissions,
- network access permissions,
- terminal execution permissions,
- environment access limits,
- secret storage boundary,
- logs and crash isolation,
- no access to Codex OAuth tokens,
- no silent Warp AI credit usage.

## Product Roadmap

### V1: VSIX Lite

Goal:

Install and consume declarative VSIX packages without executing arbitrary extension code.

Acceptance:

- User can install a local `.vsix`.
- Extension appears in `Settings > Extensions`.
- Manifest is parsed.
- Unsupported contributions are listed.
- Themes/snippets/languages/commands/configuration are recognized.
- No arbitrary Node code is executed.
- No Warp AI credits are used.
- No Codex token is read or copied.

### V2: Open VSX Integration

Goal:

Search and install extensions from Open VSX.

Acceptance:

- Search by extension id.
- Show publisher/name/version/downloads/license.
- Install selected version.
- Update/uninstall.
- Compatibility report before install.

### V3: Dynamic Language/LSP

Goal:

Make languages and LSP definitions plugin-driven.

Acceptance:

- Extension can add a language id and file extensions.
- Extension can register snippets for that language.
- User can map a language to an LSP command.
- Existing Rust/Python/TS LSP still works.

### V4: Node Extension Host Preview

Goal:

Run a small class of Node extensions.

Acceptance:

- Supports `activate(context)` and `deactivate`.
- Provides minimal `vscode.commands`, `vscode.workspace`, `vscode.window`, and `vscode.Uri`.
- Supports `onCommand` and `onLanguage` activation.
- Extensions cannot access Codex OAuth tokens.
- Extension crashes do not crash WarpCodexOss.

### V5: Selected API Expansion

Goal:

Support high-value extension classes, not full VS Code parity.

Priority:

- Language clients.
- Formatter providers.
- Completion providers.
- Code actions.
- Diagnostics.
- Tree views.
- Limited webview.

## High-Risk Areas

### Full VS Code API Surface

The `vscode` API is not just a set of functions. It assumes VS Code's workbench model, editor model, URI model, command registry, event lifecycle, context keys, and extension host behavior.

Risk:

Trying to support all of it early will stall the project.

Mitigation:

Start with declarative contributions and a visible compatibility matrix.

### Webview and UI Extensions

Many popular extensions depend on webviews. Warp's UI framework is not VS Code's DOM workbench.

Risk:

Webview support can become a separate app platform inside Warp.

Mitigation:

Defer webview or support only isolated panels later.

### Node Runtime and Module Loading

VS Code extensions expect Node and bundler-specific output. QuickJS cannot run most real extensions unchanged.

Risk:

QuickJS-only strategy breaks compatibility.

Mitigation:

Use QuickJS for Warp-native plugins; add Node host for VSIX runtime compatibility.

### Language Registry Refactor

The current LSP path is enum-driven and persistence-sensitive.

Risk:

Changing enums directly can break persistence and existing LSP behavior.

Mitigation:

Add a registry layer around existing enum variants first; migrate gradually.

### Security

VSIX packages are arbitrary code once runtime execution is enabled.

Risk:

Filesystem, network, terminal, secrets, and model credentials can be exposed.

Mitigation:

V1 must not run arbitrary code. Runtime execution must be permissioned.

## Recommended Next Change

Create an OpenSpec change:

`warpcodexoss-vsix-lite-extension-manager`

Initial artifacts:

- proposal: VSIX Lite install and manifest parser.
- design: extension store, manifest model, contribution router.
- tasks: parser tests, sample VSIX fixture, settings UI, install/uninstall.
- spec: installed extensions and compatibility report.

## Suggested ChatGPT Research Prompt

Paste this into ChatGPT with the GitHub branch URL:

```text
You are reviewing a local fork of Warp OSS called WarpCodexOss.

Repository branch:
https://github.com/jupiternaut/warp/tree/gengrf/codex-oauth-bridge

Goal:
Research whether WarpCodexOss can support a VSIX / VS Code-style plugin system.

Please produce an engineering research report that covers:

1. Current extension/plugin architecture in the repo.
2. Current editor, LSP, code pane, MCP, and Agent SDK boundaries.
3. What parts can be reused for a VSIX-compatible system.
4. What parts must be refactored.
5. A staged roadmap from VSIX Lite to partial runtime compatibility.
6. A risk matrix for running arbitrary extension code.
7. A concrete V1 implementation plan with file-level changes.

Important constraints:

- Do not propose full VS Code compatibility as V1.
- Keep Codex OAuth safe: never read, copy, or parse ~/.codex/auth.json.
- Local Codex mode must not silently consume Warp AI credits.
- V1 should prefer declarative contribution points over arbitrary code execution.
- Runtime extension support should be permissioned and isolated.

Useful local evidence from prior inspection:

- app/src/plugin/host/native/mod.rs
- app/src/plugin/host/native/runner.rs
- app/src/plugin/host/native/js_api/mod.rs
- app/src/plugin/app/mod.rs
- crates/lsp/src/config.rs
- crates/lsp/src/supported_servers.rs
- crates/lsp/src/manager.rs
- app/src/code/mod.rs
- app/src/code/local_code_editor.rs
- app/src/ai/local_codex.rs

External references:

- https://code.visualstudio.com/api/advanced-topics/extension-host
- https://code.visualstudio.com/api/references/extension-manifest
- https://code.visualstudio.com/api/references/activation-events
- https://code.visualstudio.com/api/references/contribution-points
- https://code.visualstudio.com/api/extension-capabilities/overview
- https://open-vsx.org/
```

## Final Recommendation

Build the plugin system, but define the product as:

`WarpCodexOss Extensions: VSIX Lite + Warp-native APIs`

Do not define it as:

`A full VS Code-compatible extension host`

The former is a realistic, compounding platform. The latter is a multi-quarter workbench compatibility project.
