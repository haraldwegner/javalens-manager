# Sprint 12 Backlog (draft)

> **Status: draft, written 2026-04-28 after Sprint 11 shipped (fork v1.5.0 + v1.5.1; manager v0.11.0).** Sprint 11 closeout work — JDT-UI preference defaults so the 4 disabled refactoring happy-path tests pass, plus the cross-bundle `pullUp_acrossOsgiBundles` integration test — ships as fork **v1.5.2** and is *not* part of Sprint 12.

## Goal

Two threads, one release pair:

1. **Workspace verification tools (Ring 1, fork)** — `compile_workspace` and `run_tests`. The two missing tools that close the agent's *refactor → compile → test → fix* loop. Without them the agent has to either drive Maven/Gradle from the shell (slow, brittle) or walk per-file `get_diagnostics` (incomplete and N+1).
2. **Tray menu lifecycle controls (manager)** — promote "Stop all services" to a "Start / Stop all" pair, and add per-workspace start-stop toggle entries with a colored status indicator. Today the tray can only show the window, stop-all, and quit — there is no way to drive an individual workspace from the tray.

End-of-sprint outcome:

- Per-service tool count: **60 → 62.** Headroom against Antigravity's ≈100-tool cap remains comfortable on a single active workspace.
- The agent can ask "did my refactor compile cleanly?" and "do the tests still pass?" through MCP tools instead of shelling out.
- The manager tray menu shows live workspace state and lets the user start/stop any single workspace without opening the window.
- Fork tagged `v1.6.0`; manager tagged `v0.12.0`.

Predecessor: [`sprint-11-backlog.md`](sprint-11-backlog.md). IDE-grade roadmap reference: same doc, "IDE-grade roadmap (Sprint 12+, preview only)" section.

## Repos touched

- **`javalens-mcp` (fork)** — Phase A: two new tools, target-platform additions for JUnit launching, fixtures for compile-failure / test-success / test-failure paths. Cut release `v1.6.0`.
- **`javalens-manager`** — Phase B: tray rework in `src-tauri/src/lib.rs`, status aggregation helpers in `manager_service` / `runtime_manager`, a Tauri `dashboard-changed` event so the tray refreshes when the dashboard updates. Cut release `v0.12.0`.

## Out of scope (settled)

- HTTP/networked-service direction — still tracked separately in [`sprint-future-networked-service.md`](sprint-future-networked-service.md).
- Eclipse plugin packaging (Ring 3) and JDK-API-migration tools (Ring 2) — Sprint 13+.
- Manager-side tray *settings* (icon variant chooser, tray-only mode, etc.) — keep the existing `use_system_tray` toggle as the only user-facing knob.
- Notifications on workspace state change (toast / OS-native notification) — appealing but out of scope; the tray indicator covers the primary use case.

## Authorship / attribution rule

Same as Sprint 11: zero AI-attribution boilerplate in commits, release notes, or docs produced during execution. See `feedback_no_coauthored_trailer.md`.

## Order of work

Phases A and B are independent — different repos, different stacks. They can run in parallel by separate engineers. Phase C is strictly last.

1. **Phase A — Ring 1 verification tools (fork)** (~5 days)
   1. A.1 `compile_workspace` (~1 day)
   2. A.2 `run_tests` (~3-4 days; JUnit launching is the meaty part)
2. **Phase B — Tray menu lifecycle controls (manager)** (~2 days)
3. **Phase C — Cutover release** (~half day)

Total ~1.5 weeks of focused work.

## Phase A — Ring 1 workspace verification tools

### A.1 `compile_workspace`

**Why:** today, after a multi-file refactor an agent has to call `get_diagnostics` once per modified file to learn whether the refactor compiled cleanly across the workspace. That misses (a) cascading errors in files the agent didn't touch and (b) project-level errors (missing `Require-Bundle`, classpath issues). One JDT incremental-build pass surfaces both in a single call.

**Files (javalens-mcp):**

- `org.javalens.mcp/src/org/javalens/mcp/tools/CompileWorkspaceTool.java` (new) — for each loaded `IJavaProject`, call `project.build(IncrementalProjectBuilder.INCREMENTAL_BUILD, monitor)`, then collect every `IMarker` of severity ERROR / WARNING under each project root, group by file.
- `JavaLensApplication.registerTools()` — register the new tool.

**Input shape:**

```json
{
  "projectKey": "optional — scope to one project; default workspace-wide",
  "minSeverity": "ERROR | WARNING",       // default ERROR
  "includeTaskMarkers": false              // default false (skip TODO/FIXME markers)
}
```

**Result shape:**

```json
{
  "operation": "compile_workspace",
  "projectsCompiled": 3,
  "errorCount": 2,
  "warningCount": 5,
  "diagnostics": [
    {
      "filePath": "src/main/java/com/example/Foo.java",
      "line": 42,
      "column": 8,
      "severity": "ERROR",
      "message": "The method bar() is undefined for the type Foo",
      "sourceProject": "core"
    }
  ]
}
```

**Failure modes:**

- `INVALID_PARAMETER` — `projectKey` doesn't resolve to a loaded project.
- *No `COMPILATION_FAILED` error* — compilation errors are a normal result, returned in `diagnostics`. The tool itself only fails on missing project / aborted build.

**Tests** (3, in `org.javalens.mcp.tests/.../tools/verification/CompileWorkspaceToolTest.java`):

- `happy_cleanProject_returnsZeroErrors` — fixture compiles cleanly; result has `errorCount: 0`.
- `compileError_returnsErrorMarker` — fixture with a deliberate `Type x = "string";` mismatch; result has matching ERROR diagnostic at the expected line.
- `validation_unknownProjectKey_returnsInvalidParameter`.

### A.2 `run_tests`

**Why:** the post-refactor verification half. Agents currently shell out (`mvn test -Dtest=Foo#bar`, `gradle :module:test --tests …`); the round-trip is 30s+ on a cold Maven/Gradle, classpath drift between IDE and CLI is a real failure mode, and stdout parsing for JUnit is fragile. JDT's `JUnitLaunchConfigurationDelegate` uses the same classpath the IDE uses for indexing — single source of truth.

**Files (javalens-mcp):**

- `org.javalens.target/org.javalens.target.target` — add features:
  - `org.eclipse.jdt.junit.feature.group` (JUnit Eclipse integration core)
  - `org.eclipse.jdt.junit.runtime` and `org.eclipse.jdt.junit5.runtime` (the test-runner side launched in the forked JVM)
- `org.javalens.mcp/META-INF/MANIFEST.MF` — `Require-Bundle: org.eclipse.jdt.junit.core, org.eclipse.debug.core, org.eclipse.jdt.launching`.
- `org.javalens.mcp/src/org/javalens/mcp/tools/RunTestsTool.java` (new) — main implementation.
- `org.javalens.mcp/src/org/javalens/mcp/tools/junit/JUnitLaunchHelper.java` (new) — wraps `ILaunchConfiguration` + `JUnitLaunchConfigurationDelegate.launch(...)`, captures stdout/stderr from the forked JVM, parses the JUnit XML report (`junit-platform-events.xml` or the legacy report file written into `<workspace>/.metadata/.plugins/org.eclipse.jdt.junit.core/`).
- `JavaLensApplication.registerTools()` — register.

**Input shape:**

```json
{
  "projectKey": "optional — defaults to all projects in workspace",
  "scope": {
    "kind": "method | class | package",
    "filePath": "for kind=method/class — the source file",
    "line": 42,                            // for kind=method — caret on the @Test method
    "column": 4,                           //   "
    "typeName": "com.example.FooTest",     // alt to filePath/line for kind=class
    "packageName": "com.example.tests"     // for kind=package
  },
  "framework": "junit4 | junit5 | testng | auto",  // default auto-detect from classpath
  "timeoutSeconds": 120,                            // default 120, hard-cap 600
  "vmArgs": []                                       // optional list, e.g. ["-Xmx512m"]
}
```

**Result shape:**

```json
{
  "operation": "run_tests",
  "framework": "junit5",
  "projectsTested": 1,
  "summary": { "total": 14, "passed": 12, "failed": 1, "skipped": 1, "timeMs": 3150 },
  "failures": [
    {
      "testClass": "com.example.FooTest",
      "testMethod": "shouldComputeBar",
      "status": "FAILED",
      "message": "expected: <2> but was: <3>",
      "stackTrace": "  at com.example.FooTest.shouldComputeBar(FooTest.java:18)\n  ...",
      "durationMs": 12
    }
  ],
  "stdoutTail": "last 100 lines of forked-JVM stdout",
  "stderrTail": "last 100 lines of forked-JVM stderr"
}
```

**Failure modes:**

- `INVALID_PARAMETER` — bad `scope` (file/line don't resolve to a `@Test` method, package not found, etc.).
- `LAUNCH_FAILED` — `JUnitLaunchConfigurationDelegate.launch` couldn't start the JVM (missing main class, classpath unresolved, junit-runtime not on the target platform). **No partial results.**
- `LAUNCH_TIMEOUT` — `timeoutSeconds` exceeded; partial results from whatever the JVM had reported by then are still returned, with `summary.timedOut: true`.

**Test scope detection rules:**

- `kind: method` — file + line must land inside a method annotated with the framework's `@Test` (or be the method declaration itself). Reject otherwise.
- `kind: class` — type must contain at least one `@Test` method (otherwise `INVALID_PARAMETER`, since launching a non-test class produces a confusing "no tests found" result).
- `kind: package` — package must exist; collects all test classes recursively under it.

**Auto-framework detection:**

- Walk the project's resolved classpath; if `junit-jupiter-api-*.jar` or `org.junit.jupiter.api` is on it → `junit5`. Else `org.junit.Test` (JUnit 4) → `junit4`. Else `org.testng.annotations.Test` → `testng`. Else `INVALID_PARAMETER` ("no test framework on classpath").

**Tests** (~6, in `org.javalens.mcp.tests/.../tools/verification/RunTestsToolTest.java` + a `simple-junit5-maven` fixture):

- `happy_methodScope_returnsPassed` — fixture method passes; result has `summary.passed: 1`.
- `happy_classScope_returnsMixedResults` — fixture class with 1 pass + 1 fail; both reflected in `summary` and `failures[]`.
- `happy_packageScope_collectsAllTests` — fixture package with 2 test classes (3 + 2 methods) → total 5.
- `validation_methodScopeNotOnTestMethod` — caret on a non-`@Test` method → `INVALID_PARAMETER`.
- `frameworkAutoDetect_pickJUnit5` — fixture with junit-jupiter on classpath → resolved framework is `junit5` (no explicit `framework` arg).
- `timeout_returnsPartialResults` — fixture with `Thread.sleep(60_000)` in a test, `timeoutSeconds: 2` → `summary.timedOut: true`, the slow test absent from `summary.passed`.

**Cross-cutting:**

- The test launch runs in the manager's existing Equinox JVM but in a forked sub-JVM (Eclipse standard: each JUnit launch is a fresh process). Stdout/stderr captured via `IProcess.getStreamsProxy()`. Cap captured output at 1 MB per stream — truncate older lines first.
- Reuse the workspace's `.metadata` for run history. Don't pollute it: clean up `org.eclipse.jdt.junit.core/session-*` files on tool exit unless the result indicates failure (kept for diagnostics).

### A.3 Documentation

- New `docs/release-notes/v1.6.0.md` — covers both tools, the input/result contract, the target-platform additions.
- README updates: bump tool count to 62, add a "Verification" subsection mirroring the v1.5.1 "Structural refactorings" subsection, link to the new release notes.
- `docs/upgrade-checklist.md` — note that `org.eclipse.jdt.junit.runtime` features are required at the target-platform level, not just import-package.

## Phase B — Tray menu lifecycle controls (manager)

### B.1 Status aggregation helper

**Why:** the tray needs a *workspace-level* status (Running / Starting / Stopped / Failed), but `RuntimeStatusRecord.phase` is per-project. A workspace's process is shared across all its projects; aggregation rules are simple but need a single source.

**Files:**

- `src-tauri/src/manager_service.rs` — new method `pub fn workspace_status_summary(&self) -> Vec<WorkspaceStatusSummary>` returning one entry per workspace_name with the aggregated phase. Aggregation:
  - any project `Failed` → `Failed`
  - else any project `Starting` → `Starting`
  - else all projects `Running` → `Running`
  - else (all `Stopped` or empty) → `Stopped`
- `src-tauri/src/runtime_manager.rs` — surface a snapshot getter that returns `(workspace_name, phase)` pairs without holding the snapshot lock across the call.

**New struct:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatusSummary {
    pub workspace_name: String,
    pub phase: RuntimePhase,
    pub project_count: usize,
}
```

**Tests** (Rust unit tests in `manager_service.rs`):

- `workspace_status_summary_running` — 2 projects, both Running → `Running`, project_count 2.
- `workspace_status_summary_failed_dominates` — Running + Failed → `Failed`.
- `workspace_status_summary_starting_dominates_running` — Running + Starting → `Starting`.
- `workspace_status_summary_no_projects_returns_stopped` — empty workspace → `Stopped`.

### B.2 Tray menu rebuild

**Files:**

- `src-tauri/src/lib.rs` — replace the static `Menu::with_items(app, &[&tray_show, &tray_stop_all, &tray_quit])` with a `rebuild_tray_menu(app_handle)` helper called (a) on app startup and (b) every time the dashboard changes.

**New menu shape (top-down):**

```
Show
─────────────────────────
[●] jats                       (per-workspace; click toggles start/stop)
[●] internal-tools
[○] legacy-suite
─────────────────────────
Start all services
Stop all services
─────────────────────────
Quit
```

**Status icon mapping** (PNG icons rendered via `tauri::menu::IconMenuItem` — colored, sharp on HiDPI, dark-theme friendly):

| Phase | Icon | File |
|---|---|---|
| Running | green filled circle | `src-tauri/icons/status-running.png` |
| Starting | yellow filled circle | `src-tauri/icons/status-starting.png` |
| Failed | red filled circle | `src-tauri/icons/status-failed.png` |
| Stopped | gray outline circle | `src-tauri/icons/status-stopped.png` |

**Asset spec:** four 16×16 PNGs (RGBA, transparent background). Same image used on all three platforms — the OS scales for HiDPI from the single asset. `~1 KB each`, total < 5 KB committed.

**Rationale (over Unicode emoji):** colored emoji glyphs render inconsistently across Linux distros without a bundled emoji font (fallback boxes), and desaturate under several common GTK themes. PNG icons via `IconMenuItem` look identical on macOS / Windows / GNOME-with-AppIndicator / KDE / XFCE and stay sharp on high-DPI displays. The cost is four small asset files vs. inline Unicode characters — worth it for predictable rendering.

**Cross-platform footnote:** GNOME ships no system tray by default; users need the AppIndicator extension (or equivalent) for any tray functionality. The existing v0.11.0 tray icon already requires this — no new dependency from Phase B.

**Click semantics:**

- Per-workspace entry: click invokes `start_runtime` if phase is `Stopped` / `Failed`, `stop_runtime` if `Running` / `Starting`. The state-keyed action means the user's mental model is "click toggles" without needing visible "Start" / "Stop" prefixes.
- "Start all services": invokes `manager_service.start_all_runtimes()` (already exists; wired but never called from the tray).
- "Stop all services": existing behaviour, kept.

**Per-workspace menu IDs:** use a deterministic prefix — `tray_workspace_toggle:<workspace_name>` — and parse the suffix in the `on_menu_event` handler.

### B.3 Live menu refresh

**Why:** if the menu is built once at startup, the status circle goes stale the moment any workspace transitions. The dashboard already updates status in real time; the tray needs to follow.

**Approach:**

- In `manager_service` — every `start_runtime` / `stop_runtime` / `start_all_runtimes` / `stop_all_runtimes` call already updates the snapshot. Emit a new Tauri event `javalens://workspace-status-changed` after each successful update.
- In `lib.rs` — on app setup, register a listener for `javalens://workspace-status-changed`. The listener calls `rebuild_tray_menu(app_handle)` with current state. The rebuild is cheap (≤ 10 menu items typically); GTK/macOS rebuild is sub-50ms.
- For external state changes (process died unexpectedly, crash detected by `runtime_manager`'s health-check polling), the polling already produces a `phase` change — extend that path to emit the same event.

**Tests:**

- Rust integration test in `manager_service`: simulate a `start_runtime` call and assert one `workspace-status-changed` event was emitted with the expected payload.
- Manual smoke (in the QA matrix below): start a workspace, confirm the tray glyph flips to 🟡 then 🟢; kill the underlying javalens process from the shell, confirm tray flips to 🔴 within the health-check interval (~5 s).

### B.4 Settings interaction

The existing `use_system_tray: bool` toggle in Settings stays the only knob. When tray is disabled, none of B.2/B.3 fires (no menu to rebuild). When the user re-enables tray at runtime, the existing tray-startup path still works as today; rebuild fires on first dashboard load afterwards.

No new settings fields. No `projects.json` schema change.

## Phase C — Cutover release

### C.1 Tag fork v1.6.0

In `javalens-mcp`:

- Bump pom + MANIFEST.MF qualifiers as needed.
- `docs/release-notes/v1.6.0.md` — `compile_workspace` and `run_tests` together.
- `git tag -a v1.6.0 -F docs/release-notes/v1.6.0.md && git push origin v1.6.0`.
- CI workflow (already fork-safe, see Sprint 11) cuts the GitHub Release.

### C.2 Manager release v0.12.0

In `javalens-manager`:

- Bump `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` to `0.12.0`.
- Update [`src/assets/help.md`](../src/assets/help.md):
  - Add a paragraph on `compile_workspace` and `run_tests` (and that they're picked up automatically once fork v1.6.0 lands via the existing release-repo poller).
  - Add a paragraph + screenshot on the new tray menu (Start all / per-workspace toggles / status circles).
  - Cross-link the README's "System tray on Linux" section so users on vanilla GNOME (Fedora, Debian) know `gnome-shell-extension-appindicator` is required for the tray icon to appear. Pop!_OS / Ubuntu / KDE / XFCE / Cinnamon / MATE users see no change.
- Refresh `public/help/` screenshots if the dashboard or tray changed visibly.
- `docs/release-notes/v0.12.0.md` — covers Phase B (tray rework) and notes the paired fork v1.6.0. **Must include** a "System tray on Linux" subsection mirroring the README's, so users with vanilla GNOME know they need `gnome-shell-extension-appindicator` installed for the new per-workspace tray entries to render.
- `git tag -a v0.12.0 -F docs/release-notes/v0.12.0.md && git push origin v0.12.0`. The release workflow creates a draft; **publish via `gh api -X PATCH /repos/.../releases/{id}` with `draft: false`** (reusing the v0.11.0 publishing pattern from 2026-04-28).

## Critical files

| Repo / Path | Phase | What changes |
|---|---|---|
| `javalens-mcp/org.javalens.mcp/.../tools/CompileWorkspaceTool.java` | A.1 | NEW — diagnostics aggregation |
| `javalens-mcp/org.javalens.mcp/.../tools/RunTestsTool.java` | A.2 | NEW — JUnit launcher wrapper |
| `javalens-mcp/org.javalens.mcp/.../tools/junit/JUnitLaunchHelper.java` | A.2 | NEW — `ILaunchConfiguration` + result parsing |
| `javalens-mcp/org.javalens.target/org.javalens.target.target` | A.2 | Add JUnit features to the target platform |
| `javalens-mcp/org.javalens.mcp/META-INF/MANIFEST.MF` | A.2 | `Require-Bundle` additions |
| `javalens-mcp/org.javalens.mcp/.../JavaLensApplication.java` | A | Register both new tools |
| `javalens-mcp/org.javalens.mcp.tests/.../tools/verification/*` | A | NEW — ~9 tests + 2 fixtures |
| `javalens-manager/src-tauri/src/lib.rs` | B.2, B.3 | Tray rebuild + event listener |
| `javalens-manager/src-tauri/icons/status-{running,starting,failed,stopped}.png` | B.2 | NEW — four 16×16 RGBA status icons |
| `javalens-manager/src-tauri/src/manager_service.rs` | B.1, B.3 | `workspace_status_summary` + event emit |
| `javalens-manager/src-tauri/src/runtime_manager.rs` | B.1 | Snapshot getter for aggregation |
| `javalens-manager/src/assets/help.md` | C.2 | Tray + verification-tools docs |
| `javalens-manager/{package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json}` | C.2 | 0.12.0 |
| `javalens-manager/docs/release-notes/v0.12.0.md` | C.2 | NEW |

## Reusable infrastructure already in place

- `IJavaProject.build(...)` and `IMarker` collection — Eclipse JDT primitives, no new abstractions for `compile_workspace`.
- `commands::start_runtime` / `commands::stop_runtime` (per-project, takes `project_id`) and `commands::start_all_runtimes` / `commands::stop_all_runtimes` — all four already exist and work; Phase B only wires them to tray entries.
- `RuntimePhase` enum (`Stopped` / `Starting` / `Running` / `Failed`) — Sprint 10 v0.10.4. Maps directly to the four glyph states.
- `WorkspaceFileWatcher` (fork v1.5.0) — irrelevant here but worth noting it stays untouched.

## Verification (sprint exit)

End-to-end smoke after Phase C:

1. **Compile-clean workspace** — load a project, run `compile_workspace`, assert `errorCount: 0`.
2. **Compile-error introduced by refactor** — `move_class` something into a package where its imports break, run `compile_workspace`, assert ERROR diagnostic at expected file/line.
3. **Test green path** — `run_tests` with `kind: method` on a green test → `summary.passed: 1`.
4. **Test red path** — same with a deliberately failing test → `failures[0]` populated, `summary.failed: 1`.
5. **Tool count** — `health_check` reports **62 tools** per service. `compile_workspace` and `run_tests` appear; old tools unaffected.
6. **Tray manual smoke** — manager v0.12.0 starts, tray shows three workspace entries with correct status icons. Click a green-icon entry → workspace stops, icon flips to gray. Click gray → workspace starts, icon flips yellow → green within ~5 s.
7. **Tray "Start all" / "Stop all"** — both round-trip correctly; per-workspace icons follow.
8. **External-kill detection** — kill a workspace's javalens PID from the shell; tray icon flips to red within the polling interval (≤ 5 s).
9. **No-tray mode** — disable `use_system_tray` in Settings, restart manager. Window-only operation works as today; no tray crashes.

## Cut line if Phase A.2 slips

`run_tests` is the biggest single piece (JUnit launching + result parsing + framework auto-detect). If it slips:

- Ship `compile_workspace` alone as fork **v1.6.0** (per-service tool count 60 → 61).
- Manager **v0.12.0** still ships with the full Phase B tray work (it's repo-independent).
- Cut a follow-up fork **v1.6.1** for `run_tests` once the launching plumbing settles. Manager picks it up via release-repo polling; help.md gets a small follow-up commit.

## Build / test commands

`javalens-mcp`:

```bash
cd /home/harald/CursorProjects/javalens-mcp
mvn clean verify          # full build + all tests
# Per-phase TDD shortcut:
mvn -pl org.javalens.mcp.tests -am test -Dtest='CompileWorkspaceToolTest,RunTestsToolTest'
```

`javalens-manager`:

```bash
cd /home/harald/CursorProjects/javalens-manager
npx svelte-check --tsconfig ./tsconfig.json
cargo check --manifest-path src-tauri/Cargo.toml
cargo test  --manifest-path src-tauri/Cargo.toml --lib
# Manual tray smoke after package:
npx tauri dev
```

## Definition of Done

- [ ] `compile_workspace` returns workspace-wide diagnostics in one call; tool tests green (3/3).
- [ ] `run_tests` launches JUnit / TestNG via JDT and returns parsed pass/fail/skip with stack traces; tool tests green (6/6).
- [ ] Per-service tool count is **62**.
- [ ] Tray menu shows live per-workspace status glyphs and per-workspace toggle entries; "Start all" parallel to "Stop all" works; menu rebuilds within 50 ms of any workspace state change.
- [ ] `WorkspaceStatusSummary` aggregation tested with the four phase-mix cases (Rust unit tests).
- [ ] Tray smoke matrix items 6–9 above all pass on Linux (Ubuntu 22.04 / GNOME) and at least one of macOS or Windows.
- [ ] Fork `v1.6.0` published; manager `v0.12.0` published as Latest with release notes.
- [ ] No regression on Sprint 11 fixtures (existing 414/414 mcp tests + manager Rust tests stay green).
- [ ] Zero AI-attribution boilerplate in any commit message, release note, or doc produced during the sprint.
