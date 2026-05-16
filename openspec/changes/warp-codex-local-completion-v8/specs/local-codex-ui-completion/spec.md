## ADDED Requirements

### Requirement: Local Codex streams must not inherit cloud task ids
WarpCodexOss SHALL use local run ids for local Codex Agent response streams.

#### Scenario: Request contains ambient task id
- **WHEN** local Codex mode handles a request whose metadata contains `ambient_agent_task_id`
- **THEN** the emitted stream init `run_id` starts with `local-codex-`
- **AND** the emitted `run_id` is not the request ambient task id

### Requirement: Local Codex conversations must not sync cloud task status
WarpCodexOss SHALL not report task status for conversations owned by local Codex.

#### Scenario: Local server conversation token
- **WHEN** a conversation server token starts with `local-codex-`
- **THEN** `TaskStatusSyncModel` treats it as local
- **AND** no `UpdateAgentTask` request is made for that conversation

### Requirement: App-server UI acceptance must have event-shape evidence
WarpCodexOss SHALL log lightweight Codex app-server event metadata during local app-server turns.

#### Scenario: App-server event received
- **WHEN** Codex app-server emits a notification with a `method`
- **THEN** Warp logs the method name
- **AND** logs item id and item type when available
- **AND** does not log the full event payload

### Requirement: Local app-server finish must be visible in logs
WarpCodexOss SHALL log the local finish path when a local app-server turn completes, cancels, or errors.

#### Scenario: App-server completed
- **WHEN** local app-server reaches `turn/completed`
- **THEN** Warp logs that the app-server completed and sent a done `StreamFinished`
