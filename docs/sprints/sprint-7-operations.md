# Sprint 7 Operations Guide

## Purpose

Sprint 7 introduces deploy automation for client MCP config + MCP-first rule artifacts across:

- Cursor
- Claude
- Antigravity
- IntelliJ

Deploy modes:

- `Deploy to Agents` (write)
- `Dry run` (validate without write)
- `Preview` (show generated content intent)
- `Regenerate` (force rewrite managed sections)

## Preconditions

- At least one managed project is configured.
- MCP client paths are configured in Settings (`MCP Config Locations`).
- JavaLens runtime path resolves for managed projects.

## Deploy Workflow

1. Open Dashboard.
2. Trigger one of the deploy actions.
3. In the targets popup, confirm or override clients for this run.
4. Review per-client result summary (`deployed X/Y`) and skipped reasons.
5. For failures, correct path/runtime issues in Settings, then rerun.

## Deploy Target Selection

- Settings defines default deploy participation flags per client:
  - Cursor
  - Claude
  - Antigravity
  - IntelliJ
- Dashboard deploy popup preselects clients from those flags.
- Run-scoped changes in the popup do not mutate saved Settings.
- Unselected or disabled clients are reported as `skipped` with explicit reason.
- Missing target path remains a client-level `skipped` outcome (`not configured`).

## Managed Section Behavior

- The manager writes only manager-owned managed sections.
- Non-manager sections remain preserved.
- Merge behavior follows `mcpMergeMode`.
- Backups are written when `mcpBackupBeforeWrite` is enabled.

## Tray and Quit Behavior

- Window close:
  - `useSystemTray=true` and running services -> hide to tray (no shutdown).
  - otherwise -> prompt for confirmation before app shutdown.
- Tray `Quit`:
  - app surfaces the main window and prompts explicitly.
  - with running services -> confirm stop-and-quit, optional hide-to-tray fallback.
  - with no running services -> lightweight shutdown confirmation.
- Tray menu supports `Show`, `Stop all services`, `Quit`.

## Troubleshooting

- **Path validation failed**: ensure target config parent directory exists.
- **No deployable services**: verify project runtime resolves and project list is non-empty.
- **Unexpected skipped clients**: check Settings deploy flags and run-scoped popup selection.
- **Partial client failures**: inspect per-client `validationErrors` and rerun.
