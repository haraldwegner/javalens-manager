# Sprint 15.2 — Lifecycle hotfix bundle (v0.15.2)

> **Status: SUPERSEDED 2026-06-11 — never executed as a standalone release.** The
> rollout was pushed back and the full scope (bugs #10–#13, plus #14 found later the
> same day) was absorbed into **Sprint 16 → v0.16.0** per Harald's decision; see
> [`sprint-16-backlog.md`](sprint-16-backlog.md). The fork-side companion
> (`readOnlyHint`) shipped in fork v1.9.0 (Sprint 14b) instead of a separate v1.8.7.
> This file is retained as the requirements record for the absorbed scope.
>
> **Status (historical): drafted 2026-06-11** from the post-v0.15.1 lifecycle audit (2026-06-09) + live three-client verification (2026-06-10). Patch release on the Sprint 15 line; Sprint 16 (Windows installer + scan-folder) is unaffected and remains next.
>
> **Target version: javalens-manager v0.15.2.** Pure bug-fix patch — no new features, no settings, no UI.
>
> **Coupled fork release:** fork **v1.8.7** (`readOnlyHint` annotations, 14a hotfix line) ships in the same window. The two are independent at the code level but released together as the next pair. Fork scope lives in [`javalens-mcp/docs/sprints/sprint-14a-http-sse-transport.md`](../../../javalens-mcp/docs/sprints/sprint-14a-http-sse-transport.md) § "Post-ship hotfix line".
>
> **Bugs closed:** [#10, #11, #12, #13](../bugs.md).

## Goal

Close the four lifecycle gaps that survived Sprint 15: per-client MCP-config schema (Antigravity), and the port/token + resident-JVM leaks on rename / delete / quit. After v0.15.2, deploy works on all three clients without hand-patching, and no workspace operation leaks ports, tokens, or JVMs.

## Requirements

### 1. Per-client MCP-config writer schema (bug #10) — HIGH

Both writer sites (`manager_service.rs::build_client_mcp_json` ~line 2701 and `write_managed_json_block` ~line 2586) branch on the `client` parameter (currently ignored — `let _ = client;`):

| Client | Entry shape |
|---|---|
| claude, cursor | `{ "type": "http", "url": "http://127.0.0.1:<port>/mcp", "headers": { "Authorization": "Bearer <token>" } }` (v0.15.1 shape, unchanged) |
| antigravity | `{ "serverUrl": "http://127.0.0.1:<port>/mcp", "headers": { "Authorization": "Bearer <token>" } }` — **no `type` field** |

Shape live-verified 2026-06-10 (8-tool smoke over native `serverUrl` HTTP, 0 connection errors). `disabled: true` (Disable writer mode) is supported by all three clients and stays client-agnostic. The post-write validator (`validate_client_config_shape`) accepts both shapes keyed by client.

### 2. Rename migrates port/token state (bug #11) — MEDIUM-HIGH

`config.rs::rename_workspace` migrates the `workspaces[]` `(port, token)` entry to the new name in the same persist transaction. Deployed configs stay valid across rename (same port + token). Re-deploy after rename only changes the server *id*, not the endpoint.

### 3. Delete releases workspace state (bug #12) — MEDIUM

`manager_service.rs::delete_workspace` and `delete_project` (when the last project leaves a workspace) call `config.rs::release_workspace_state` (exists; currently test-only) after stopping the resident.

### 4. Quit stops residents (bug #13) — MEDIUM

`commands.rs::perform_quit_action`: `QuitAction::Quit` gains a best-effort `stop_all_runtimes()` (bounded wait, then exit regardless) so no quit path orphans JVMs. `StopAndQuit` semantics unchanged.

### 5. One-shot orphan prune (migration for bug #11's existing damage)

On `projects.json` load, entries in `workspaces[]` whose name matches no current workspace are dropped (with an info log listing what was pruned). Known live damage: 4 orphans from the rename chain `orb-strategies` → `ORB_strategies` → `strategies_orb` → `strategies-orb` (ports 8802-8804 + one predecessor). Backup the file before the first pruning write (existing `projects.json.bak.<ts>` mechanism).

## Out of scope (settled)

- Fork-side anything (v1.8.7 is its own scope on the 14a line).
- New writer targets beyond the three supported clients.
- Settings/UI changes; Windows (Sprint 16).

## Verification (sprint exit)

- `cargo test` green including new lifecycle tests (rename-migrates, delete-releases, quit-stops, prune-orphans, per-client-shape).
- Live: deploy → all three clients connect **without any hand-patching** (the 2026-06-10 manual patches at `~/.gemini/antigravity/mcp_config.json` / `~/.claude.json` become redundant — verify the writer reproduces them).
- Live: rename a workspace → deployed config keeps working after manager restart; `projects.json` has exactly one entry for it.
- Live: delete a test workspace → its port is reused by the next allocation.
- Live: tray Quit → `pgrep -af javalens.jar` returns 0.
- First v0.15.2 launch prunes the 4 known orphans (log line confirms).

## Definition of Done

- [ ] Bugs #10-#13 flipped to `FIXED in v0.15.2` in [`bugs.md`](../bugs.md).
- [ ] `docs/release-notes/v0.15.2.md` written (per-client schema table + lifecycle fixes + orphan-prune migration note).
- [ ] Tag `v0.15.2` pushed; CI publishes the GitHub Release as Latest.
- [ ] No AI-attribution boilerplate anywhere.
- [ ] Memory `project_sprint_state.md` updated: "v0.15.2 shipped; Antigravity deploys natively; lifecycle leaks closed; next: manager Sprint 16 / fork v1.8.7 (if not yet shipped) + Sprint 14b".
