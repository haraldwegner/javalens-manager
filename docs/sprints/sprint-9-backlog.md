# Sprint 9 Backlog

## Goal

Bootstrap the `<your-org>/javalens-mcp` fork, ship Component 1 (`.classpath` + pom.xml source resolution) so strategies-orb refactoring is unblocked, and put the manager on a configurable release URL so it pulls the fork's first release `v1.2.1`.

End-of-sprint outcome:
- Fork repo exists at `<your-org>/javalens-mcp` with the source-resolution fix on `main`.
- `javalens-manager` reads `release_repo` from settings (default kept as upstream until cutover).
- `v1.2.1` of the fork is published via GitHub Actions and installs cleanly via the manager.
- `mcp__jl-11127-strategies-orb__search_symbols query="SlotManager"` returns ≥ 1 hit.

Reference plan: `~/.claude/plans/make-a-plan-happy-fern.md`.

## Problem Statement

- javalens-mcp's `ProjectImporter` discovers source roots via a hardcoded `SOURCE_MAPPINGS` heuristic. It ignores pom.xml's `<sourceDirectory>` override and Eclipse `.classpath` source entries.
- This silently misses 49 production files in `strategies_orb` and would miss source folders in any future Maven `<sourceDirectory>` override or non-conventional Eclipse layout.
- The manager downloads from upstream `pzalutski-pixel/javalens-mcp` only; we need to point it at our fork before we can ship the fix.

## Repos touched

- **`javalens-mcp` (NEW fork — sibling to `javalens-manager` at `~/CursorProjects/javalens-mcp/`):** Component 1 helpers + refactor in `ProjectImporter.java`, unit tests, GitHub Actions release workflow, ADRs 0001 + 0004.
- **`javalens-manager`:** Configurable release URL (`release_repo` setting, `JAVALENS_RELEASE_REPO` env override).

## Phase 0 — Fork and local setup

### 0.0 Sprint setup

- [ ] This file: `docs/sprints/sprint-9-backlog.md` (✅ created).
- [ ] Fork repo seeded with `docs/architecture.md`, `docs/adr/0001-source-resolution-precedence.md`, `docs/adr/0004-helper-portability-constraint.md`. Each ADR ~½ page: Context / Decision / Consequences.

### 0.1 Fork on GitHub

- [ ] `gh repo fork pzalutski-pixel/javalens-mcp --clone=false --org=<your-github-org>` (or omit `--org` for personal namespace).

### 0.2 Clone fork as sibling and add upstream remote

- [ ] `git clone git@github.com:<your-github-org>/javalens-mcp.git ~/CursorProjects/javalens-mcp`
- [ ] `git remote add upstream https://github.com/pzalutski-pixel/javalens-mcp.git && git fetch upstream`

### 0.2b Cursor multi-repo workspace

- [ ] Create `~/CursorProjects/javalens-dev.code-workspace` with relative paths to `./javalens-manager` and `./javalens-mcp`.
- [ ] Open it in Cursor and confirm both repos appear in the explorer; SCM panel shows both git roots.

### 0.3 Verify upstream baseline build

- [ ] `cd ~/CursorProjects/javalens-mcp && mvn clean package` — succeeds against upstream `main` before any edits.
- [ ] Identify the produced jars: `org.javalens.core/target/org.javalens.core_*.jar` and `org.javalens.mcp/target/org.javalens.mcp_*.jar`.

### 0.4 Work branch

- [ ] `git checkout -b feature/source-resolution-and-workspace`.

### 0.5 Document hotswap layout

- [ ] Inspect `~/.cache/javalens-manager/tools/javalens/javalens-1.2.0/javalens-v1.2.0/plugins/` to confirm the OSGI plugin location used for hotswap during dev iteration.

## Phase 1 — Component 1: `.classpath` + pom.xml source resolution

Acceptance criteria:
- [ ] New helper `readPomSourceDirs(Path)` returns `<sourceDirectory>` and `<testSourceDirectory>` from pom.xml. Pure DOM, no JDT in the signature.
- [ ] New helper `readEclipseClasspath(Path)` returns `srcPaths`, `libPaths`, `outputPath` from `.classpath`. Resolves `..` references against project parent.
- [ ] `ProjectImporter.addSourcePathsFromDirectory` honors discovery precedence: pom.xml override → `.classpath` src → existing `SOURCE_MAPPINGS` heuristic.
- [ ] `ProjectImporter.addDependencyEntries` merges `.classpath` `kind="lib"` entries with Maven-resolved deps.
- [ ] `LoadProjectTool.getDescription()` advertises the new precedence.
- [ ] Unit tests in `ProjectImporterTest`:
  - [ ] `readPomSourceDirs_returnsOverride`
  - [ ] `readPomSourceDirs_absentReturnsEmpty`
  - [ ] `readEclipseClasspath_returnsAllKinds`
  - [ ] `readEclipseClasspath_resolvesParentRefs`
  - [ ] `addSourcePathsFromDirectory_pomOverridesHeuristic`
  - [ ] `addSourcePathsFromDirectory_classpathFallbackBeforeHeuristic`
- [ ] Hotswap validation:
  - [ ] `mcp__jl-11127-strategies-orb__get_project_structure` → `sourceFileCount` ≈ 104.
  - [ ] `mcp__jl-11127-strategies-orb__search_symbols query="SlotManager"` → ≥ 1 hit.
  - [ ] `mcp__jl-11125-execsim-java__get_project_structure` → file count unchanged (no regression on conventional Maven).
  - [ ] One `jl-111xx-com.example.app*` service → file count unchanged or higher (pure-Eclipse path works).

## Phase 2 — Component 4: Configurable release URL in javalens-manager

Acceptance criteria:
- [ ] `ManagerSettings.release_repo: String` with `default_release_repo()` returning `"pzalutski-pixel/javalens-mcp"`.
- [ ] `release_manager.rs` removes `LATEST_RELEASE_URL` const; `fetch_latest_release` accepts settings, composes `https://api.github.com/repos/{repo}/releases/latest`.
- [ ] `JAVALENS_RELEASE_REPO` env var overrides the setting at runtime.
- [ ] `UpdateSettingsInput` extended; `update_settings` writes the field through.
- [ ] Unit test mirroring `release_status_marks_update_when_latest_not_installed` but with non-default `release_repo` confirms URL composition.
- [ ] `cargo test -p javalens-manager release_manager` passes.

## Phase 3 — First fork release

Acceptance criteria:
- [ ] `.github/workflows/release.yml` in fork triggers on `v*` tags, builds via `mvn -B clean package`, stages release tree to match upstream archive shape, publishes `.tar.gz` via `softprops/action-gh-release@v2`.
- [ ] Confirmed: archive layout matches what `find_relative_jar_path` walks (verified via `tar -tzf` against an upstream release).
- [ ] Tag and push `v1.2.1` to fork.
- [ ] `JAVALENS_RELEASE_REPO=<your-org>/javalens-mcp` set; manager update check pulls `v1.2.1` and installs to `~/.cache/javalens-manager/tools/javalens/javalens-1.2.1/`.
- [ ] Phase 1 verification re-run against the cleanly-installed (not hotswapped) version.

## Definition of Done

- [ ] All acceptance criteria above checked.
- [ ] Two commits in `javalens-mcp` (Component 1 + release workflow); one or two commits in `javalens-manager` (Component 4 + tests).
- [ ] No regressions on conventional Maven or pure-Eclipse projects.
- [ ] `mcp__jl-11127-strategies-orb__search_symbols query="SlotManager"` returns ≥ 1 hit when manager is wired to the fork's `v1.2.1`.

## Team Split

- `java-engineer`: Phase 1 helpers, `ProjectImporter` refactor, unit tests in `javalens-mcp`.
- `tauri-engineer`: Phase 2 settings extension, `release_manager.rs` refactor, env-var override, unit test.
- `release-engineer`: Phase 3 GitHub Actions workflow in fork, archive shape verification, first tagged release.
- `qa-test-engineer`: Verification matrix on hotswap and on cleanly-installed `v1.2.1`.
- `docs-engineer`: ADRs 0001 + 0004 in fork, `architecture.md` seed.

## Deferred to Sprint 10/11

- `load_workspace` MCP tool and multi-project `WorkspaceManager` (Sprint 10).
- `projectKey` parameter on every analysis tool (Sprint 10).
- javalens-manager single-workspace spawn mode (Sprint 10).
- MANIFEST.MF parsing helpers and workspace bundle pool (Sprint 11).
- Flipping `single_workspace_mode` default to true and `v1.3.0` cutover release (Sprint 11).

## Manager-side bugs to fix in this sprint or Sprint 10

- **Projects table doesn't refresh when second project is added.** Repro: register one project via the "Register Project" form — appears in Managed Projects table. Register a second project — does not appear until manual refresh / restart. Likely missing reactive update or stale snapshot in `App.svelte` / project list state. File ref: `src/App.svelte`, `src-tauri/src/commands.rs` (project list endpoints), `src-tauri/src/config.rs` (`ProjectRecord`). Acceptance: registering N projects in sequence shows all N in the table without refresh.

## Hotswap-iteration note

When dev-iterating on top of the installed v1.2.0 runtime (replacing `org.javalens.core_*.jar` and `org.javalens.mcp_*.jar` in the cached install), keep the **filename qualifier the same** as the originally installed jar. Reason: `~/.cache/javalens-manager/tools/javalens/javalens-1.2.0/javalens-v1.2.0/configuration/config.ini` hardcodes `osgi.bundles=...,reference:file:org.javalens.core_<qualifier>.jar@4,...` — Equinox can't find the bundle if the filename qualifier changes, and the launcher fails with exit code 13. If the qualifier needs to change, also clear `configuration/org.eclipse.osgi/<digit>/` and `.manager/` so OSGi rebuilds its bundle state cache. This issue dissolves once the fork ships its own release: `v1.2.1`'s install is a self-consistent snapshot with config.ini referencing the new qualifier.
