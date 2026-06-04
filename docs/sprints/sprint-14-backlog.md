# Sprint 14 Backlog

> **Status: ✅ closed 2026-06-04** with manager v0.14.0 shipped. Three manager bugs (#1 / #2 / #3) flipped to FIXED. Originally written 2026-06-03 as cold-start orientation for fresh agents.
>
> Sprint 13 closed 2026-04-29 with fork v1.7.0 (`fc25acf`) + manager v0.13.0 (native dynamic tray menu). Everything 2026-05-11 → 2026-05-19 (fork v1.7.1 `4465e09`, manager v0.13.1 `63d5d39`, install.sh aarch64 wrapper `7f1f7b7`, manager bugs.md tracker `f0bb7dc` → `1699550`) was incremental cleanup, NOT a sprint. Sprint 14 ran 2026-06-03 → 2026-06-04.

## Post-ship findings (v0.14.0 smoke, 2026-06-04) — ✅ RESOLVED in v0.14.1

Live smoke of the v0.14.0 manager UI surfaced three bugs + a feature ask + answered four standing questions. All addressed in the **v0.14.1 hotfix** that shipped 2026-06-04 (`~/.claude/plans/make-a-plan-happy-fern.md`).

**Bugs filed in [`docs/bugs.md`](../bugs.md):**
- **#4** — Autostart on boot: tray and Settings checkbox out of sync after one-side toggle. Root cause: no backend → frontend event for settings-changed; tray-side toggle doesn't notify the Svelte store. Fix: emit `javalens://settings-changed` from the tray arm; frontend `listen()` reloads the dashboard. **v0.14.1 Stage 1.**
- **#5** — Settings page wording "Machine-local runtime paths and **port** controls" still references ports. Ports gone since Sprint 10 v0.10.4 (workspace_name grouping replaced them with stdio transport). Fix: drop "port" from the subtitle. **v0.14.1 Stage 2.**
- **#6** — "Data Root" card header doesn't reflect content (card also holds Use system tray + Autostart on boot, soon + Auto-start workspaces). Fix: rename to **"System Settings"** with vertical checkbox layout. **v0.14.1 Stage 2.**

**Feature filed in [`docs/bugs.md`](../bugs.md):**
- **#7** — Auto-start workspaces on manager boot. Today `autostart_on_boot` launches the manager UI only; workspaces stay stopped until user clicks Start All. New global flag `auto_start_workspaces_on_boot` defaults `false` (opt-in); when both flags true, setup-block runs `start_all_runtimes` after 2 s deferred + on a separate thread. **v0.14.1 Stage 3.**

**Also in v0.14.1:**
- **Help-page screenshot refresh** — after the rename + new checkbox, existing `public/help/settings-{top,bottom}.png` are stale. USER STEP for capture from `npx tauri dev`; agent updates `help.md` text. **v0.14.1 Stage 4.**
- **Manager-side doc-scrub** — forward-only anonymize: remove proprietary product-name references from PUBLIC manager files (docs, sprint backlogs, release notes, code comments). No history rewrite. Fork-side scrub stays Sprint 15. **v0.14.1 Stage 5.**

**Answered standing questions (no work, just answers):**
- *Repos in sync?* — Loosely coupled. Sprints 7–13 paired releases; Sprint 14 dropped the pattern (fork v1.7.2 cut). Each repo can ship at its own cadence now.
- *Where the fork's sprint doc goes?* — Fork uses `docs/bugs.md` + `docs/upgrade-checklist.md` + `docs/release-notes/v<X>.md`. If/when fork hits a sprint-shaped cycle, add `javalens-mcp/docs/sprints/sprint-N-backlog.md` mirroring the manager pattern.
- *Port selection gone?* — Yes since v0.10.4 (Sprint 10). Legacy `ProjectRecord.assigned_port` kept for migration; ignored at runtime.
- *Does autostart-on-boot start MCP processes too?* — No (before v0.14.1). v0.14.1 Feature #7 adds that as a separate opt-in flag.

**Deferred to Sprint 15:**
- Doc-scrub of `javalens-mcp` (fork-side) — same playbook as manager Stage 5, separate repo.
- Gradle / Android read-only support (fork).
- Target-candidate detection + multi-step orchestration framework (fork).
- Fork bugs #6 / #7 (fork — whichever future release picks them up).

> **Detailed executable plan:** [`~/.claude/plans/make-a-plan-happy-fern.md`](../../../../.claude/plans/make-a-plan-happy-fern.md). This backlog is the design / intent narrative; the plan is what an agent executes (Stages 0–8, checkpoints C0–C8). If the plan and this backlog disagree, the plan wins — update both to keep the agreement.

## Goal

Ship **manager v0.14.0** as Latest on GitHub. Close three manager bugs (`bugs.md #1 / #2 / #3`); add two requested features (Reload all, Autostart on boot); open the manager to Mac users (macOS CI matrix, DMG + curl install paths).

**Scope adjustment 2026-06-04:** the originally-paired **fork v1.7.2** was dropped mid-execution (see Requirement 7 below). The fork is independent — its release cadence is needs-driven, not stapled to manager sprints. Fork bugs #6 / #7 stay OPEN in `javalens-mcp/docs/bugs.md` and roll into whichever future fork release ships them.

Predecessor: [`sprint-13-backlog.md`](sprint-13-backlog.md).

## Requirements

1. **Bug #1 full fix — webview blank on aarch64.** The env-var (`WEBKIT_DISABLE_DMABUF_RENDERER=1`) currently lives only in `install.sh`. Bake it into the AppImage's `AppRun` and a `.deb` postinst wrapper so every install path (curl one-liner, AppImage double-click, `dpkg -i`) is covered.

2. **Bug #2 — process-death state.** External `kill` of a managed JVM currently flips the workspace to `Stopped` (gray ○) instead of `Failed` (red ✗). State-machine gap in `runtime_manager.rs`: needs an `expected_stopping: Option<Instant>` flag set in `stop_runtime()`, branched on in `try_join_running_workspace()`. `RuntimePhase::Failed` carries `exit_code: Option<i32>`.

3. **Bug #3 — single-instance.** Double-launch from the dock spawns a second process + second tray icon. ~15-line fix wiring `tauri-plugin-single-instance` as the FIRST plugin in the Builder chain; the duplicate process exits before any expensive setup, the existing window raises via the standard `unminimize → show → set_focus` pattern.

4. **Reload all.** Alongside Start all / Stop all in both the tray menu and the dashboard toolbar, add **Reload all**: stop every workspace, poll until every phase is `Stopped|Failed` (30 s deadline), then start all. One Tauri command, one menu entry, one button, one frontend store action.

5. **Autostart on boot.** Manager auto-launches at session login via `tauri-plugin-autostart`. Surfaced two ways: a checkbox in Machine Runtime Controls next to "Use system tray" AND a checkable tray menu item. One backend setting, two UI entry points, one OS-level autostart (Linux: `~/.config/autostart/*.desktop`; macOS: LaunchAgent; Windows: registry).

6. **macOS CI matrix + Mac install paths.** Add `macos-14` (Apple Silicon) + `macos-13` (Intel) to `release.yml`. Free for public repos. Ships unsigned; Gatekeeper bypass (`xattr -d com.apple.quarantine`) documented in README. Two Mac install paths: (A) `install.sh` curl one-liner gets a Darwin branch (`hdiutil attach` + `cp` to /Applications + `xattr`); (B) DMG download + drag-drop. Apple Developer signing is a separate later track.

7. ~~**Paired fork v1.7.2.**~~ **DROPPED 2026-06-04.** The "Sprint-N = manager + fork" pattern from Sprints 12 / 13 was a historical convenience, not a contract. Bugs #6 / #7 are tracked but not user-blocking; cutting a release just to clear two entries adds CI overhead and regression risk without surfacing anything new to users (the release-poller would hand them the same tool surface). Bugs stay OPEN in `javalens-mcp/docs/bugs.md` and roll into whichever future fork release ships them — likely a v1.7.x patch when more fork-side work accumulates, or v1.8.x with the Gradle / Android backlog.

## Repos touched

- **`javalens-manager`** — Stages 0–6 + 8. Rust (`src-tauri/src/*.rs`), Svelte / TypeScript (`src/lib/**/*.{svelte,ts}`), CI (`.github/workflows/release.yml`), install.sh, README, docs. Cut release **v0.14.0**.
- ~~**`javalens-mcp`** (fork) — Stage 7 only. Java + tests. Cut release **v1.7.2**.~~ — **dropped 2026-06-04**; see Requirement 7.

## Out of scope (settled)

- Apple Developer signing / notarization for macOS builds — separate later track. v0.14.0 ships unsigned with documented `xattr` Gatekeeper bypass.
- Doc-scrub across both repos — Sprint 15 candidate; deferred to keep Sprint 14 atomic.
- Gradle / Android read-only support, target-candidate detection, multi-step orchestration framework — Sprint 15 candidates from `docs/sprints/future-sprint-enhancements.md`.
- Per-workspace settings UI; multi-window manager.
- Configurable Reload-all timeout (default 30 s; configurable becomes a v0.14.x patch if reported).

## Authorship / attribution rule

Same as Sprints 11 / 12 / 13: **zero AI-attribution boilerplate** in commits, release notes, code, or docs produced during execution. See [`feedback_no_coauthored_trailer.md`](../../../../.claude/projects/-home-harald-CursorProjects-javalens-manager/memory/feedback_no_coauthored_trailer.md).

## Workflow rule

Same focused-tests / full-verify-at-sprint-end rule from Sprint 13:

- Manager: `cargo test --lib` per stage (~10 s); `npx svelte-check` for frontend; `cargo check` + `cargo build` before commit.
- Fork: focused `mvn -pl <module> -Dtest=<Class> verify` (~2 min) per bug; full reactor `mvn verify` (~20 min) ONCE at Stage 7's checkpoint as the pre-tag regression gate.
- Per CLAUDE.md collaboration spec: STOP at every checkpoint with the format-checklist summary. Wait for `continue`. Never silently skip / advance past failure.

## Order of work

Stages are atomic; sequence them strictly to avoid lib.rs edit conflicts (Stages 1 / 3 / 4 all touch the Tauri Builder + tray menu).

- **Stage 0 — Sprint 14 admin** (~30 min): memory + this backlog doc. C0.
- **Stage 1 — Bug #3 single-instance** (~45 min). C1.
- **Stage 2 — Bug #2 process-death → Failed** (~2 h). C2.
- **Stage 3 — Reload all** (~2 h). C3.
- **Stage 4 — Autostart on boot** (~3 h). C4.
- **Stage 5 — Bug #1 full fix** (CI post-bundle env-var bake-in) (~2 h). C5.
- **Stage 6 — macOS CI matrix + install.sh Darwin branch + README Mac install** (~1 h). C6.
- ~~Stage 7 — Fork v1.7.2 (bugs #6 + #7) + release~~ — **dropped 2026-06-04**.
- **Stage 8 — Manager v0.14.0 cutover + release** (~1 h). C8.

Total ~1.5 days of focused work (was ~2 days before Stage 7 was dropped).

## Critical files

See the plan's Stage −1 Prerequisites section for the full source-files-to-read list. Highest-touch:

| Repo / Path | Stages | Change |
|---|---|---|
| `javalens-manager/src-tauri/src/lib.rs` | 1, 3, 4 | Tauri Builder plugins + tray menu rebuild + cache-key + on_menu_event arms |
| `javalens-manager/src-tauri/src/runtime_manager.rs` | 2 | `expected_stopping` flag + `RuntimePhase::Failed { exit_code }` |
| `javalens-manager/src-tauri/src/commands.rs` | 3, 4 | New `reload_all_runtimes` + `set_autostart_on_boot` |
| `javalens-manager/src-tauri/src/manager_service.rs` | 3 | `reload_all` impl (stop + poll + start) |
| `javalens-manager/src-tauri/src/config.rs` | 4 | `ManagerSettings::autostart_on_boot` + `UpdateSettingsInput` + update_settings |
| `javalens-manager/src-tauri/Cargo.toml` | 1, 4 | `tauri-plugin-single-instance`, `tauri-plugin-autostart` |
| `javalens-manager/src/lib/components/ProjectList.svelte` | 3 | Reload-all button |
| `javalens-manager/src/lib/components/RuntimeSettings.svelte` | 4 | Autostart-on-boot checkbox |
| `javalens-manager/src/lib/{api/tauri.ts, stores/app.ts}` | 3, 4 | `reloadAllRuntimes` + `setAutostartOnBoot` wrappers + store actions |
| `javalens-manager/.github/workflows/release.yml` | 5, 6 | Post-tauri-action env-var bake-in step + macOS matrix entries |
| `javalens-manager/install.sh` | 5, 6 | Simplify Linux wrapper-removal + Darwin branch |
| `javalens-manager/README.md` | 6, 8 | Mac install section + Gatekeeper note + version timeline |
| `javalens-manager/docs/release-notes/v0.14.0.md` | 8 | NEW |
| `javalens-manager/docs/bugs.md` | 8 | #1 / #2 / #3 → FIXED in v0.14.0 |
| ~~`javalens-mcp/...`~~ | ~~7~~ | **DROPPED** — see Requirement 7. |

## Reusable infrastructure already in place

- **Sprint 13 cache-keyed tray refresh** (`lib.rs:380` snapshot) — Reload all menu item is static; Autostart-on-boot extends the cache-key tuple with `autostart_on_boot: bool` so toggling forces a rebuild.
- **`stop_all_runtimes` / `start_all_runtimes`** (`manager_service.rs:417-440`, `381-405`) — Reload all = stop_all + poll + start_all; no new plumbing.
- **"Use system tray" precedent** (`ManagerSettings::use_system_tray`, RuntimeSettings.svelte:723-726, `ConfigStore::update_settings`) — Autostart-on-boot is a direct copy-paste pattern.
- **`tauri-plugin-dialog`** already in use → other 2.x Tauri plugins are version-compatible.
- **install.sh proven wrapper pattern** (commit `7f1f7b7`) — Stage 5 lifts it into CI; Stage 6's Darwin branch mirrors the Linux shape.
- **release.yml apt-install guard** (`if: startsWith(matrix.platform, 'ubuntu-')`, line 22) — auto-excludes macOS runners; no extra branching needed.
- **Sprint 12 `release-poller`** — manager auto-detects fork v1.7.2 on the next poll cycle after C7.

## Verification (sprint exit)

After Stage 8:

1. Manager v0.14.0 tag published as Latest; CI matrix produces 4 artifacts (Linux amd64 / arm64 + macOS aarch64 / x86_64).
2. Three bugs flipped: manager #1 / #2 / #3 → FIXED in v0.14.0. (Fork bugs #6 / #7 stay OPEN per the 2026-06-04 drop of Stage 7.)
4. Smoke install: `dpkg -i` on a fresh Linux container (no env-var exported) → dashboard renders. `./install.sh` on GB10 → dashboard renders. Mac: `xattr`'d + dragged → app launches and dashboard renders (deferred to user if no Mac available at C8).
5. Double-launch from dock → existing window raises; one tray icon; one process.
6. External `kill -9 <jar>` → tray glyph red ✗ within 5 s.
7. Click Reload all in tray → sequenced Stopped → Starting → Running across all workspaces.
8. Toggle Autostart-on-boot in Settings → log out + back in → manager running. Toggle off via tray → log out + back in → not running.
9. `add_project /home/harald/CursorProjects/javalens-mcp` against the fork itself → success, no duplicate-classpath error.
10. Memory updated to "Sprint 14 closed — Sprint 15 queued"; this backlog's Status flips to `✅ closed`.

## Cut lines (if a stage hits unexpected pain)

- **Stage 5 AppImage AppRun injection fragile** → ship `.deb` postinst-wrapper fix only; defer AppRun injection to v0.14.1. `install.sh` wrapper covers existing AppImage users meanwhile.
- **Stage 6 macOS runners saturated / Apple Silicon-only diverges from Intel** → ship Linux-only as v0.14.0; macOS as v0.14.1. Cut `macos-13` (Intel) first if Intel surfaces unique issues; keep `macos-14` (Apple Silicon).
- **Single-instance plugin doesn't fire on GNOME-appindicator path** (per [`appindicator_constraints.md`](../../../../.claude/projects/-home-harald-CursorProjects-javalens-manager/memory/appindicator_constraints.md)) → fallback PID-lock at startup. Re-open Stage 1.
- **Tray checkable for Autostart-on-boot renders weakly on GNOME** → fallback label-text toggle (`Autostart on boot ✓` / `Autostart on boot`) in a v0.14.x patch.
- **30 s Reload-all deadline too short for slow JVMs** → configurable `ManagerSettings.reload_timeout_seconds` (default 30) in v0.14.x patch if reported.

## Build / test commands

```bash
# Manager (during Stages 0–6 + 8):
cd /home/harald/CursorProjects/javalens-manager
cargo check --manifest-path src-tauri/Cargo.toml
cargo test  --manifest-path src-tauri/Cargo.toml --lib       # ~10 s
npx svelte-check --tsconfig ./tsconfig.json
shellcheck install.sh && bash -n install.sh                  # Stage 6 only

# Fork (Stage 7) — dropped 2026-06-04. The mvn commands here are
# retained for historical reference; they apply to whichever future
# fork release picks up bugs #6 / #7.
```

## Definition of Done

- [ ] Stages 0–6 + 8 closed (Stage 7 dropped 2026-06-04 — see Requirement 7).
- [ ] Manager **v0.14.0** published as Latest; 4 release artifacts (Linux amd64 / arm64 + macOS aarch64 / x86_64).
- [ ] Manager bugs #1 / #2 / #3 → FIXED in v0.14.0.
- [ ] Reload all live in tray + dashboard.
- [ ] Autostart-on-boot live in Settings + tray; OS-level autostart fires at next login.
- [ ] README has Mac install section (curl + DMG + Gatekeeper xattr note).
- [ ] Smoke install: Linux x86_64 + aarch64 (GB10) GREEN.
- [ ] Zero AI-attribution boilerplate in any commit / release note / doc produced during the sprint.
- [ ] No regression on Sprint 13 fixtures (existing Rust tests + svelte-check + fork reactor stay green).
- [ ] Memory updated: `project_sprint_state.md` → "Sprint 14 closed — Sprint 15 queued".
- [ ] This backlog's Status header flipped to `✅ closed` in C8.
