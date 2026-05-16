# Invalid Local Conversation Quarantine Plan

Status: criteria documented, code pending

V4 records the cases that should be quarantined before old local Codex conversations are treated as normal Agent history. It does not delete or migrate user history.

## Criteria

- Missing root task: restore currently fails with `NoRootTask`.
- Missing initial query: history loading can warn about missing initial query and skip the record.
- Protobuf or serialized task decode failure.
- Root task exists but contains no user query and no usable diff summary.

## Required Runtime Behavior

- Classify invalid records as quarantined before normal restore/history flows.
- Omit quarantined records from ordinary history and restore surfaces.
- Log one startup summary count instead of repeated per-record warnings.
- Preserve the original records until the user explicitly confirms cleanup.

## Non-Goals For This Slice

- No automatic deletion.
- No database rewrite.
- No claim that old invalid records are already hidden by V4 code.
