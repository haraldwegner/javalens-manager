# Sprint 15 Backlog

> **Status: re-scoped 2026-06-07 (second revision).** Originally drafted with three requirements (bug #8 + Windows installer + scan-folder). First re-scope landed bug #9 SECONDARY fix only. **Second re-scope (this version) expands to bug #9 PRIMARY fix end-to-end** alongside fork v1.8.5 HTTP/SSE-default. Sprint 15 now closes the 30 GB JVM leak completely — no "secondary interim". Windows installer + scan-folder remain in [`sprint-16-backlog.md`](sprint-16-backlog.md).
>
> Predecessor: [`sprint-14-backlog.md`](sprint-14-backlog.md). Sprint 14 closed 2026-06-04 with manager v0.14.0 + v0.14.1 shipped and fork v1.8.0 shipped (`ffa68c7`, tag `v1.8.0`, published as Latest).
>
> Targets manager **v0.15.0**. **Depends on fork [Sprint 14a → v1.8.5](../../../javalens-mcp/docs/sprints/sprint-14a-http-sse-transport.md)** (HTTP/SSE as default transport). The two releases ship as a coordinated pair: v1.8.5 first (fork foundation), then v0.15.0 (manager hosting + URL-emitting writer).

## Goal

Ship **manager v0.15.0** as Latest on GitHub, closing two bugs end-to-end (no "deferred to Sprint 17+" — there is no Sprint 17):

1. **Bug #8** ([docs/bugs.md #8](../bugs.md)) — auto-downloaded fork jar breaks deployed MCP client configs because the deployed `args` reference a versioned filename that no longer exists on disk after the auto-download.

2. **Bug #9 PRIMARY fix** ([docs/bugs.md #9](../bugs.md)) — 30 GB JVM leak. Today every MCP client process spawns its own pair of stdio JVMs per deployed workspace (~16 JVMs at ~24 GB observed). The PRIMARY fix is the shared-service architecture: manager hosts ONE resident fork JVM per workspace; deployed MCP-client configs emit URL endpoints pointing at the resident services; N clients × M workspaces → **M JVMs total**, not N × M.

**Acceptance signal:** open 3 Claude windows + 1 Cursor window with `autostartOnBoot` ON → `pgrep -af javalens.jar | wc -l` returns **2** (one per workspace), NOT **16**.

## Requirements

1. **Bug #8 fix — stable jar filename via `current` symlink.** The release-poller's jar-placement step writes the auto-downloaded jar so that `~/.cache/tools/javalens/current` is a symlink atomically pointing at the freshly-extracted versioned dir. Versioned dirs stay on disk for diagnostics + rollback. The deploy writer references `current/javalens.jar` (stable path); subsequent auto-downloads stop breaking deployed configs.

2. **Bug #9 PRIMARY fix — resident JVM hosting + URL-emitting MCP writer.** Four coordinated changes:

   - **Per-workspace port + token allocation.** Each workspace gets a stable `(port, token)` pair on first deploy, persisted in `projects.json`. Port allocator probes a range (default 8800-8999) for free ports; token = 32-byte hex via `SecureRandom`. Pair survives manager restarts; user can override via Settings.

   - **Resident JVM lifecycle.** New `ResidentService` subsystem spawns one fork JVM per workspace with `java -jar ~/.cache/tools/javalens/current/javalens.jar -port <port> -token <token> -data <workspace_path>` (HTTP/SSE is default in fork v1.8.5; no `-transport http` flag needed). Captures the `READY url=... token=...` line from stdout. Health-monitors via cheap TCP-connect; SIGTERM → ≤5s → SIGKILL on stop. Lifecycle hooked to manager startup/shutdown + workspace add/remove + `autostart_on_boot` observer.

   - **URL-emitting MCP writer.** `build_deploy_servers` in `manager_service.rs` emits `{ url: "http://127.0.0.1:<port>", headers: { Authorization: Bearer <token> } }` instead of `{ command, args }`. Per-workspace token from the allocator. Replaces the stdio-spawn model in deployed configs.

   - **Autostart-honoring lifecycle.** When `autostart_on_boot=false`: no resident JVMs spawn AND the writer removes managed entries from `~/.cursor/mcp.json` + `~/.claude.json`. Default mode = `Remove`. Optional `mcp_disabled_writer_mode: "remove" | "disable"` setting exposes Cursor/Claude `disabled: true` mode for users who prefer visible-but-inert entries (cut-candidate; ship Remove-mode-only if schemas surprise).

## Repos touched

- **`javalens-manager`** — this sprint.
- **`javalens-mcp`** — coupled release in fork Sprint 14a → v1.8.5 (HTTP/SSE default). Manager v0.15.0 depends on v1.8.5 binary being available before Stage 10 spawn-test can run end-to-end.

## Out of scope (settled — pushed to Sprint 16)

- **Windows installer generation (Windows ARM + x64)** — moved to [`sprint-16-backlog.md`](sprint-16-backlog.md).
- **Scan-folder mode for Add Project form** — moved to [`sprint-16-backlog.md`](sprint-16-backlog.md).
- **macOS Intel builds** — explicitly NOT in scope. Apple Silicon only on the macOS side.
- **Windows code-signing certificate** — separate later track.
- **TLS / multi-tenant auth / OAuth** — networked service vision (`sprint-future-networked-service.md`). v0.15.0 uses Bearer tokens over localhost HTTP; multi-user / shared-host hardening is a separate sprint.
- **`disabled: true` writer mode for Disable variant** — cut-candidate; default `Remove` mode is sufficient for the user's stated need ("autostart off should mean off").

## Authorship / attribution rule

No Claude / AI / "Generated by …" attribution anywhere — commit messages, PR bodies, release notes, code comments, docs. Same rule that's held since 2026-04-27.

## Workflow rule

Per-stage atomic commits. Focused tests during dev (`npm run test:unit -- <pattern>` for the Svelte side, `cargo test <pattern>` from `src-tauri/` for the Rust side); full sprint-exit verify (`npm run check && npm run test:unit && (cd src-tauri && cargo test) && npx tauri build`) only at sprint-exit gate. Never push without explicit ask.

## Order of work (suggested)

Lives in detail in [`~/.claude/plans/make-a-plan-happy-fern.md`](file:///home/harald/.claude/plans/make-a-plan-happy-fern.md). High-level: bug #8 → port+token allocator → resident JVM hosting → URL writer → joint smoke → release v0.15.0. Fork v1.8.5 must tag before the resident-JVM spawn smoke can run against the real binary.

1. **Stage 8 — Bug #8 stable jar via `current` symlink.** Independent of fork; safe to do first.
2. **Stage 9 — Per-workspace port + token allocation subsystem.** New `port_allocator` + `token_generator` utilities; persisted in `projects.json`.
3. **Stage 10 — Resident JVM hosting (largest stage).** New `resident_service.rs`; spawn / READY-capture / health / lifecycle. Manager startup spawns residents per `autostart_on_boot`; tray "Stop and Quit" stops them.
4. **Stage 11 — URL-emitting writer + autostart-honoring.** `build_deploy_servers` shifts from stdio command to URL endpoint; observer rewires on `javalens://settings-changed`.
5. **Stage 12 — Joint live smoke.** 3 Claudes + 1 Cursor + autostart on → expect 2 JVMs (one per workspace). Toggle autostart off → expect 0.
6. **Stage 13 — Release v0.15.0.** Version bumps, release notes, tag, push.

(Stage numbers continue from the joint plan; Phase A Stages 0-7 are fork-side; Stages 8-13 are manager-side.)

## Critical files

- **Bug #8:** [`src-tauri/src/release_manager.rs`](../../src-tauri/src/release_manager.rs) (jar-placement step around line 605).
- **Port + token allocator (Stage 9):** new utilities in `src-tauri/src/` + per-workspace fields in [`src-tauri/src/config.rs`](../../src-tauri/src/config.rs).
- **Resident JVM hosting (Stage 10):** new `src-tauri/src/resident_service.rs` (or expand existing [`src-tauri/src/runtime_manager.rs`](../../src-tauri/src/runtime_manager.rs)). Lifecycle hooks in [`src-tauri/src/lib.rs`](../../src-tauri/src/lib.rs).
- **URL-emitting writer (Stage 11):** [`src-tauri/src/manager_service.rs`](../../src-tauri/src/manager_service.rs) — `deploy_to_agents` (line 521), `build_deploy_servers` (line 1255), `build_client_mcp_json` (line 2635). Observer pattern at [`src-tauri/src/lib.rs`](../../src-tauri/src/lib.rs) line 265 (`javalens://settings-changed`, Sprint 14 v0.14.1).

## Reusable infrastructure already in place

- **`autostartOnBoot` setting + event observer** — Sprint 14 v0.14.1 shipped settings ↔ tray sync via `javalens://settings-changed`. The writer + lifecycle observer plug into the same event.
- **`ReleaseManager`** — bug #8 fix is a refinement of its existing jar-placement step, not a new subsystem.
- **Deploy-to-agents writer** — already generates MCP client configs; Stage 11 changes WHAT shape it writes (URL vs stdio command), not the surrounding plumbing.
- **Workspace runtime supervision** (Sprint 14 v0.14.0 `RuntimePhase` machine) — Stage 10 extends with resident-JVM PID + port tracking.

## Verification (sprint exit)

- **Bug #8 verified:** simulate fork release auto-download via `release_manager` test fixture; confirm deployed MCP client configs still resolve `current/javalens.jar` after jar swap.
- **Bug #9 PRIMARY verified:** with autostart ON and v0.15.0 + v1.8.5 running, open 3 Claude windows + 1 Cursor window. `pgrep -af javalens.jar | wc -l` → **2** (one per workspace). Toggle autostart OFF + reopen clients → **0**. (Before the fix: 16.)
- **Resident-JVM lifecycle:** quit manager → resident JVMs gracefully stopped (SIGTERM → SIGKILL fallback) before manager process exits. Re-launch manager with autostart=on → residents spawn for all workspaces.
- **Release-poller migration:** simulate v1.8.5 → v1.8.6 release; confirm resident JVMs restart on the new jar (or surface "restart available" indicator per Stage 12's design choice).
- All existing tests stay GREEN — focused per stage during dev, full `npm test` + `cargo test` + `npx tauri build` at exit.

## Cut lines (if a stage hits unexpected pain)

- **`mcp_disabled_writer_mode: "disable"` opt-in (Stage 11 sub-step).** First cut. Default `remove` mode satisfies the user's stated need; `disable` mode is a convenience.
- **Stage 8 (Bug #8)** is a floor — must ship. The v0.15.0 release itself triggers bug #8 without it (own-foot-shoot).
- **Resident JVM health monitoring (Stage 10 sub-step).** Could simplify to "spawn + READY-capture only, no auto-restart-on-death" for v0.15.0. Health monitor adds resilience but isn't strictly necessary for closing the leak. Defer to v0.15.1 if time pressures.
- **Bug #9 PRIMARY fix overall** is the second floor — without it the 24 GB leak continues. Don't cut.

## Build / test commands

```bash
# Manager
cd /home/harald/CursorProjects/javalens-manager
npm install
npm run check               # type / svelte-check
npm run test:unit -- <pattern>   # focused frontend unit tests
cd src-tauri && cargo test <pattern>   # focused Rust tests
cd .. && npx tauri dev      # live smoke (depends on fork v1.8.5 jar being downloaded)
npx tauri build             # local build sanity

# Full sprint-exit verify
npm run check && npm run test:unit && (cd src-tauri && cargo test) && npx tauri build
```

## Definition of Done

- [ ] Bug #8 FIXED in v0.15.0. Verified by simulating an auto-download mid-session and confirming deployed clients still spawn.
- [ ] Bug #9 PRIMARY fix landed: resident JVMs hosted per workspace; deployed MCP configs use URL endpoints. Verified by `pgrep -af javalens.jar` returning workspace count (not client × workspace count).
- [ ] `autostart_on_boot=false` removes managed MCP entries AND stops resident JVMs. Verified by toggling + observing.
- [ ] `docs/release-notes/v0.15.0.md` written, covering both bug fixes + the dependency on fork v1.8.5 + the joint-smoke numbers (16 → 2 JVMs).
- [ ] Tag `v0.15.0` pushed; CI publishes the GitHub Release as Latest.
- [ ] No AI-attribution boilerplate anywhere.
- [ ] `MEMORY.md` + `project_sprint_state.md` updated to "Sprint 15 closed, v0.15.0 shipped; bug #8 + bug #9 (PRIMARY) FIXED end-to-end coupled with fork v1.8.5; Sprint 16 (Windows + Scan-folder) queued next".
