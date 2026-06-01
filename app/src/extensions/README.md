# VSIX contract layer integration order

`app/src/extensions` is the canonical Rust boundary for future VSIX / VS Code extension support.
It intentionally stays separate from the Warp-native QuickJS plugin host under `app/src/plugin`.

V1 VSIX Lite should integrate in this order:

1. Keep `FeatureFlag::VsCodeExtensions` as the product gate for every entry point.
2. Parse VSIX/package manifest metadata into the contract types without running extension code.
3. Validate compatibility and permissions, surfacing every unsupported feature as a structured diagnostic.
4. Add a trusted package store/registry implementation for local VSIX files, then Open VSX metadata.
5. Only after the contract is enforced, design a sandboxed runtime. Do not expose Codex OAuth tokens to extensions, do not silently consume Warp AI credits, and treat VSIX packages as untrusted.

Node extension execution, Open VSX networking, VSIX extraction, and deep editor/LSP/UI wiring are intentionally out of scope for this scaffold.
