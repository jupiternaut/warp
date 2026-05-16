## MODIFIED Requirements

### Requirement: Local Agent requests can select exec or app-server
When local Codex mode is enabled, normal Agent requests SHALL use the selected local Codex runner before any Warp server-backed AI request is made.

#### Scenario: Default runner
- **WHEN** `WARP_LOCAL_CODEX_RUNNER` is unset
- **THEN** Warp uses `codex exec --json`

#### Scenario: App-server runner
- **WHEN** `WARP_LOCAL_CODEX_RUNNER=app-server`
- **THEN** Warp uses `codex app-server --listen stdio://` for normal Agent prompts

#### Scenario: Structured short tasks
- **WHEN** passive suggestions or structured JSON helper tasks run
- **THEN** Warp keeps using `codex exec --json`
