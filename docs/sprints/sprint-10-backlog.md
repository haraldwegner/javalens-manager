# Sprint 10 Backlog

## Goal

Consolidate javalens MCP services so that multiple Java projects can share one server process. Use **port number as the workspace identity** — projects sharing an `assigned_port` run as one MCP service against one in-memory Eclipse workspace. Different ports = independent services. Drops Antigravity tool registrations from **1,764 (28 projects × 63 tools per service)** today to **63 with one workspace** (or 189 with three, 315 with five). The number of services drops from 28 to 1–N, where N is small (typically 1–3 in practice; 12 paper headroom from the port range cap).

End-of-sprint outcome:
- One javalens process serves N projects on the same port. Tools accept optional `projectKey` to scope queries; absent = search across all loaded projects.
- Adding / removing a project on a running service triggers a **live update** (`add_project` / `remove_project` MCP calls) — no service restart needed. Service start remains a manual user action.
- Workspace import assigns one shared port to all imported projects.
- Default port range shrinks from 11100–11199 to 11100–11111 (12 services).
- Optional: friendly workspace label per port.
- Help docs updated to explain port-as-workspace.

Reference plan: `~/.claude/plans/make-a-plan-happy-fern.md` (Sprint 10 section is superseded by this backlog where they conflict — the original plan had a separate `load_workspace` abstraction; this sprint uses port-grouping instead).

## Problem Statement

- 28 javalens services (one per registered project) × 63 tools per service = 1,764 tool registrations against Antigravity. Each new project adds another service and another 63 tool entries; the path to the ~100-service ceiling is straight-line.
- 63 JVM processes running simultaneously is heavy on RAM and slow on cold start.
- Related projects (e.g., the dozen JATS2 OSGi bundles) belong to one logical workspace but live in separate, unrelated MCP services today.
- The `javalens-mcp` `WorkspaceManager` is single-project by design. Tools assume one current `IJavaProject`.

## Repos touched

- **`javalens-mcp` (fork)**: extend `WorkspaceManager` to multi-project; add `add_project`, `remove_project`, `list_projects` MCP tools; thread optional `projectKey` through every analysis tool. Cut release `v1.3.0`.
- **`javalens-manager`**: port-grouped spawning (one process per unique port serving all projects on that port); live `add_project` / `remove_project` calls on settings/project changes; UI port-selector with existing-ports dropdown; default port range 11100–11111; optional workspace-label storage.

## Phase A — Java side: multi-project `WorkspaceManager` and MCP tools

### A.1 Extend `WorkspaceManager`

File: `org.javalens.core/src/org/javalens/core/workspace/WorkspaceManager.java`.

- Replace single-project state with `Map<String /*projectKey*/, IJavaProject>` (concurrent).
- Track `defaultProjectKey` (= first project loaded) so existing single-project APIs keep working for back-compat.
- Add API:
  ```java
  public void registerProject(String projectKey, IJavaProject project);
  public Optional<IJavaProject> getProject(String projectKey);
  public Collection<String> projectKeys();
  public Collection<IJavaProject> allProjects();
  public boolean unloadProject(String projectKey);
  ```
- `projectKey` derivation rule (helper, lives near `WorkspaceManager` or in a `ProjectKeys` util):
  - sanitized last segment of `projectPath` (e.g. `strategies_orb`).
  - On collision (same last segment): suffix with first 6 chars of a SHA-1 of the absolute path. Stable across sessions.

### A.2 New MCP tools

Files in `org.javalens.mcp/src/org/javalens/mcp/tools/`:

- `AddProjectTool.java` — input `{ projectPath: string }`. Calls `ProjectImporter.configureJavaProject` and `workspaceManager.registerProject`. Returns `{ projectKey, sourceFileCount, packageCount, status: "loaded" | "skipped" | "failed", error? }`. **Does not clear the existing workspace** — append-only.
- `RemoveProjectTool.java` — input `{ projectKey: string }`. Calls `workspaceManager.unloadProject`. Returns `{ projectKey, removed: bool }`. Frees JDT classpath entries; idempotent.
- `ListProjectsTool.java` — no input. Returns `{ projects: [{ projectKey, projectPath, sourceFileCount }] }`.

### A.3 Update existing tools to be multi-project-aware

All tools under `org.javalens.mcp/src/org/javalens/mcp/tools/` (and subfolders `analysis/`, `navigation/`, `refactoring/`, `search/`, `quickfix/`, `project/`):

- Input schema gains optional `projectKey: string`.
- **Absent** → run across all loaded projects, tag each result with its `projectKey`.
- **Present** → scope to that project (`workspaceManager.getProject(key)`).
- `LoadProjectTool` (existing) — keep current behavior (clears the workspace, loads one). Used as the "first call" by the manager. Description updated to mention that subsequent projects are added via `add_project`.
- `health_check` — return the full project list `[{ projectKey, projectPath, fileCount }]` instead of a single-project status.

This is a wide but mechanical change. Drive it via a small `MultiProjectRouter` helper that the tools call; tools stay thin.

### A.4 Tests

`org.javalens.core.tests/src/org/javalens/core/workspace/WorkspaceManagerMultiProjectTest.java`:
- `register_three_projects_keepsAll` — register A, B, C; allProjects() returns 3 distinct entries.
- `unloadProject_removesOneWithoutAffectingSiblings` — register A, B, C; unload B; A and C still present.
- `projectKey_collision_appendsHashSuffix` — two paths with same last segment yield two distinct keys.
- `defaultProjectKey_isFirstLoaded` — register A then B; defaultProjectKey == A's key.

`org.javalens.core.tests/src/org/javalens/core/project/ProjectImporterMultiProjectTest.java`:
- `configureJavaProject_calledTwice_givesDistinctLinkedFolders` — both projects get unique linked-folder names (no collision on `src` linked folder name across projects).

`org.javalens.mcp.tests/src/org/javalens/mcp/tools/AddProjectToolTest.java`:
- `add_validPath_returnsProjectKey` — fixture path → success response with derived key.
- `add_invalidPath_returnsError` — non-existent path → error status.

`org.javalens.mcp.tests/src/org/javalens/mcp/tools/RemoveProjectToolTest.java`:
- round-trip: add → list → remove → list.

`org.javalens.mcp.tests/src/org/javalens/mcp/tools/SearchSymbolsToolMultiProjectTest.java`:
- Load two distinct fixtures; search returns hits from both with `projectKey` provenance.
- With `projectKey=foo`, scopes correctly (no hits from the other project).

Add a second minimal Maven fixture under `org.javalens.core.tests/test-resources/sample-projects/simple-maven-b/` so multi-project tests have distinct paths.

### A.5 Release `v1.3.0`

Tag `v1.3.0` on the fork after Phase A is green. The existing fork-safe release workflow publishes the artifact.

## Phase B — Manager side: port-grouped spawning + live updates

### B.1 Port-grouped spawning

Files in `src-tauri/src/`:

- `runtime_manager.rs`: replace per-project spawn loop with port-grouped loop.
  - Group `Vec<ProjectRecord>` by `assigned_port` → `BTreeMap<u16, Vec<ProjectRecord>>`.
  - One MCP process per unique port. Process command unchanged (still `java -jar javalens.jar --port <port>`).
- `manager_service.rs`: MCP service ID becomes `jl-<port>-<workspace-label>` where `<workspace-label>` is:
  - the user-set workspace label for that port if present (see B.4),
  - else the slug of the first project on that port (e.g. `jl-11100-strategies-orb`).

### B.2 Service start: load all projects on the port

When the user clicks **Start** for a service (a port group):

1. Spawn the javalens process on that port.
2. Wait for `health_check` → operational status (no project loaded yet).
3. For the first project on the port: call `load_project(projects[0].project_path)`. (Resets the workspace; primes it with project 0.)
4. For each subsequent project on the port: call `add_project(projects[i].project_path)`.
5. Service phase = `ready` once all projects are loaded.

If any project fails to load, the service is `degraded` but stays running (other projects work). The failing project gets a per-project error in the UI.

**Service start remains manual.** No auto-start on app launch.

### B.3 Live updates: add / remove on the fly

When the user changes the project list and the affected service is **already running**, the manager updates the live workspace without restarting the process. New behavior per mutation:

- **Add a new project to a running service** (existing port selected): manager calls `add_project(path)` on the running MCP server. UI reflects new project under that service when the call returns.
- **Remove a project from a running service**: manager calls `remove_project(projectKey)` on the running server.
- **Move a project between services** (port change on existing project, both ports running): `remove_project` on the source service + `add_project` on the destination service.
- **Source service stopped**: just update the projects.json record; the next manual Start picks up the new list.
- **Destination service stopped**: same.

The frontend treats these as atomic per-project operations and shows a small spinner on the project card while the MCP call is in flight.

### B.4 Workspace label (optional, but recommended)

New persisted state in `settings.json`:
```json
"workspace_labels": {
  "11100": "strategies-orb",
  "11102": "jats2-bundles"
}
```

(or in projects.json next to projects — TBD; see open question 4.)

UI:
- In the grouped service view, the header shows the label (editable inline). Clicking it lets the user rename.
- Auto-derived if absent: same fallback as B.1's MCP service ID (slug of first project).
- Workspace import suggests a label based on the `.code-workspace` filename minus extension.

### B.5 Default port range

`config.rs`:
- `default_port_range_start` stays `11100`.
- `default_port_range_end` changes from `11199` to `11111`.
- Existing user settings respected via the existing `#[serde(default)]` machinery — only fresh installs see the new default.

### B.6 MCP client deployment

`commands.rs` and the agent-deploy code that writes `mcp.json` for Cursor / Claude / Antigravity / IntelliJ:
- Generate one MCP server entry per **unique port**, not per project.
- Server key = `jl-<port>-<workspace-label>`.
- Drops the per-project entries that today multiply into 63 entries.
- Existing `mcp_merge_mode` (`safeMerge` / `replace`) still applies.

## Phase C — Frontend

### C.1 `ProjectForm.svelte`

Replace the port input with a select:

```
Port
[ Select ▾ ]
  - New port (next free: 11100)             ← default, top option
  - 11101 — strategies-orb                   ← existing service
  - 11102 — jats2 (12 projects)              ← existing service with label + count
  - Custom port…
```

Picking an existing port = "join this service". Picking the new option = own service. Custom = freeform numeric input (advanced).

After workspace import (Discover), the candidate list shows a single port allocated to the whole import at the top:

```
Workspace port: 11102 (will be assigned to all 12 imported projects)
[ Import selected ]
```

### C.2 `ProjectList.svelte`

Two display modes (toggle in the header, default = grouped):

- **Grouped view** (default): one parent card per unique port. Header shows `<workspace-label> · port :11102 · 12 projects · status: running`. Children are project rows (collapsible). Start / Stop buttons act on the whole service. Delete-all on the workspace removes all projects on that port.
- **Flat view** (existing): one card per project. The current behavior, kept for users who prefer per-project visibility.

### C.3 Service status counts

Header `Total / Running / Stopped` chips count **services** (= unique ports), not individual projects. Add a separate `Projects: N` chip below for clarity.

## Phase D — Help / docs

`docs/help.md` (or wherever the in-app Help view loads from):
- New section: **Workspaces and ports** — explain that `assigned_port` is the workspace identity. Multiple projects sharing a port run as one MCP service; different ports = independent services.
- Use cases (mirror UC-1 to UC-4 in the open questions section below).
- Migration note: existing projects keep their port assignments untouched. Use the port dropdown to merge two projects into one workspace by setting the same port.

## Locked decisions (2026-04-25)

User confirmed all recommendations:

| # | Question | Decision |
|---|---|---|
| Q1 | Workspace label & MCP service ID | (b) Optional user-editable label in `settings.json`, falling back to first-project-slug |
| Q2 | Workspace-import port allocation | (a) Always allocate one fresh free port for the whole import |
| Q3 | Project list default view | (a) Grouped view — one parent card per port |
| Q4 | Workspace-label storage location | (a) `settings.json` under `workspace_labels: { [port]: label }` |
| Q5 | Live-update failure semantics | (a) Roll back projects.json change on MCP failure; surface error in UI |
| Q6 | `load_project` MCP tool back-compat | (a) Keep clear-and-load semantics; manager uses `load_project` for first project, `add_project` for subsequent |
| Q7 | Default port range | (a) 11100–11111 (12 ports) |
| Q8 | Draft v0.10.2 release notes now | (a) Yes — `docs/release-notes/v0.10.2.md` |
| Q9 | Sprint 10 version pairing | (a) Fork `v1.3.0` + manager `v0.11.0` shipping together |

## Open questions — confirm before implementation

### Q1. MCP service ID and workspace label storage

Options:
- (a) Auto-only, no user-editable label: service ID = `jl-<port>-<first-project-slug>`. No new persisted field.
- (b) **Optional user-editable label** stored in `settings.json` under a new `workspace_labels` map, falling back to first-project-slug if unset. Service ID and grouped-view header use the label.
- (c) Label stored on each `ProjectRecord` (per-project). Display rule: take the label of the first project on that port. Awkward when projects move between ports.

Recommend **(b)**. Lets users name important workspaces (`jats2-core`, `orb-stack`) without making it mandatory.

### Q2. Workspace-import port allocation

When user imports an N-project `.code-workspace`:
- (a) **Always allocate one fresh free port**. All N projects get that port. User can rename later.
- (b) Default to fresh, but the import dialog has a port selector (existing or new) so the user can merge an import into an existing workspace.
- (c) Always inherit the current "next free" suggestion shown in the form's port selector.

Recommend **(a)** for simplicity. **(b)** is reachable later by registering one project, then using the port-selector approach to add the rest manually.

### Q3. Project list default view

- (a) **Grouped view** (one card per port, projects nested) as default.
- (b) Flat view (current) as default; grouped view available behind a toggle.

Recommend **(a)** — grouped is the more useful mental model under port-as-workspace.

### Q4. Workspace label location

If we go with Q1 (b):
- (a) `settings.json` under `workspace_labels: { [port]: label }`.
- (b) Sibling to `projects.json` — new file `workspaces.json`.
- (c) Field on `ProjectRecord` itself.

Recommend **(a)**. Single file, single mutex, no synchronization issues with projects.json.

### Q5. Live-update failure semantics

When `add_project` or `remove_project` fails on a running service:
- (a) **Roll back the projects.json change** and surface the error in the UI; project state stays consistent across persistence + live state.
- (b) Persist the change anyway and mark the service as "needs restart"; user clicks Restart to resync.
- (c) Persist the change and best-effort live-update; if it fails, log and silently let the next restart fix it.

Recommend **(a)** — projects.json and live MCP state stay in sync. Failure mode is recoverable (try again or fix the bad path).

### Q6. Back-compat for `load_project` semantics

- (a) **Keep `load_project` as "clear and load one"** — used by the manager as the priming call before subsequent `add_project`s. Existing direct callers (CLI users, scripts) still see one-project behavior.
- (b) Make `load_project` an alias for `add_project` — additive only. Breaks any external caller that depends on the clearing semantics.

Recommend **(a)**.

### Q7. Port range default

- (a) **11100–11111** (12 services).
- (b) Different range, e.g. 11100–11119 (20 services) for headroom.

Recommend **(a)** — fewer services is the goal, 12 covers the realistic max workspaces a user would maintain.

## Definition of Done

- [x] All Java tests in Phase A pass; `mvn clean verify` green. **381/381 passing as of 2026-04-26.**
- [x] `v1.3.0` of the fork is published with `add_project`, `remove_project`, `list_projects`, multi-project `WorkspaceManager`, and `projectKey`-aware analysis tools. **Published 2026-04-26 from `hw1964/javalens-mcp` — https://github.com/hw1964/javalens-mcp/releases/tag/v1.3.0.**
- [ ] Manager's port-grouped spawn lands `add_project` / `remove_project` calls correctly for live updates; verified manually with a 5-bundle workspace import.
- [ ] Service count drops from 28 to 1–3 (= number of workspaces the user chose). Antigravity tool registrations drop from 1,848 (28 × 66) to 66 × <workspaces>, i.e. typically 66–198 — well under the 100-tool cap once GitKraken's MCP shim is also removed (decided 2026-04-26).
- [ ] Help docs explain port-as-workspace.
- [ ] CHANGELOG / README mention v0.10.0 capabilities and the upcoming v0.11.0 (manager) targeting fork v1.3.0.

## Sprint 11 (preview — updated 2026-04-26)

Sprint 11 now packages three threads under one fork release (`v1.4.0`):
- **Detection-matrix completion** — Tycho-aware Maven, workspace bundle pool (MANIFEST.MF `Require-Bundle`), Gradle Tooling API.
- **Tool-surface consolidation** — collapse ~13 narrow `find_X` tools into 2 parametric tools so per-service tool count drops 66 → 55, leaving ample budget for IDE-grade Sprint 12+ additions.
- **Cutover** — manager `single_workspace_mode` default flipped to true; manager `v0.11.0` release pairs with fork `v1.4.0`.

Upstream PR posture confirmed: **deferred indefinitely, not abandoned.** See sprint-11-backlog.md "Independence posture" for rationale.

## Team split

- `java-engineer`: Phase A — `WorkspaceManager`, `AddProjectTool`, `RemoveProjectTool`, `ListProjectsTool`, multi-project routing, tests.
- `tauri-engineer`: Phase B.1 / B.2 / B.3 / B.6 — port-grouped spawning, live updates, agent-deploy refactor.
- `frontend-engineer`: Phase C — port selector, grouped view, service-vs-project counts.
- `release-engineer`: Phase A.5 — fork v1.3.0 cut.
- `qa-test-engineer`: end-to-end matrix (single-project, workspace import, mixed mode).
- `docs-engineer`: Phase D — help and README updates.
