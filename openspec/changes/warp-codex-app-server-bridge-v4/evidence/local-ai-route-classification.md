# Local Codex AI Route Classification

This V4 audit artifact keeps local mode fail-closed. When `WARP_LOCAL_CODEX_AI=1`, user-visible generative routes must use local Codex or return a local error. They must not silently call Warp AI credit endpoints.

## Local-Codex Generative Routes

These routes are expected to route through `app/src/ai/local_codex.rs` in local mode:

- `generate_multi_agent_output`
- `generate_ai_input_suggestions`
- `generate_am_query_suggestions`
- `generate_commands_from_natural_language`
- `generate_dialogue_answer`
- `generate_metadata_for_command`
- `generate_code_review_content`

## Allowed Warp Services

These are not replaced by local Codex in V4:

- Voice transcription: `/ai/transcribe`
- Relevant-file lookup and other indexing/sync helpers that are not user-visible text generation
- Account, model-list, usage, sync, artifact, and update services

## Explicit Warp-Credit Routes

These may use Warp credits only when local mode is disabled through `WARP_LOCAL_CODEX_AI=0` or a future explicit UI action:

- Warp server-backed Agent requests
- Warp server-backed command search
- Warp server-backed dialogue answer
- Warp server-backed metadata generation
- Warp server-backed code review generation
- Warp server-backed next input / passive prompt generation

## Audit Rule

New `/ai/` endpoints or GraphQL operations that generate user-visible text, commands, diffs, suggestions, titles, metadata, commit messages, PR text, or Agent output must be added to this file before they are used from local mode.
