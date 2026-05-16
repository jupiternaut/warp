## ADDED Requirements

### Requirement: Codex file patch events must render as display-only diff cards
WarpCodexOss SHALL render structured local Codex app-server file patch updates as native-looking file diff cards without allowing Warp to apply those diffs.

#### Scenario: Structured patch update
- **WHEN** Codex app-server emits `item/fileChange/patchUpdated` with file path and diff metadata
- **THEN** Warp adds an `ApplyFileDiffs` tool call message whose tool call id starts with `local-codex-display-`
- **AND** the converted action does not require a result
- **AND** the action is not queued for Warp-side execution

#### Scenario: Incomplete patch update
- **WHEN** Codex app-server emits `item/fileChange/patchUpdated` without usable file path or diff metadata
- **THEN** Warp renders the event as transcript text
- **AND** no executable file edit action is created

### Requirement: Streaming file output remains transcript text
WarpCodexOss SHALL keep partial app-server file output deltas as transcript text.

#### Scenario: File output delta
- **WHEN** Codex app-server emits `item/fileChange/outputDelta`
- **THEN** Warp renders the delta as file diff transcript text
- **AND** no native file diff card is created from the partial delta
