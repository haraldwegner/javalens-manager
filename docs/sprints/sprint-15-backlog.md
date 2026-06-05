# Sprint 15 Backlog

> **Status: drafted 2026-06-04** after Sprint 14 closed end-to-end (manager v0.14.0 + v0.14.1, fork v1.8.0). Targets manager **v0.15.0** — three requirements bundled because each individually is small but together they round out a release: a real bug fix the user is hitting, a platform expansion that opens the manager to Windows users, and a project-onboarding UX gap the user surfaced during Sprint 14 live smoke.
>
> Predecessor: [`sprint-14-backlog.md`](sprint-14-backlog.md). Sprint 14 closed 2026-06-04 with manager v0.14.0 + v0.14.1 shipped and fork v1.8.0 shipped (`ffa68c7`, tag `v1.8.0`, published as Latest). The fork remains independent — its release cadence is needs-driven, not stapled to manager sprints. Sprint 15 is **manager-only**.

## Goal

Ship **manager v0.15.0** as Latest on GitHub. Close [`docs/bugs.md`](../bugs.md) **#8** (auto-downloaded fork jar breaks deployed MCP client configs). Expand the platform matrix to **Windows ARM + Windows x64** alongside the existing Linux amd64 / Linux arm64 / macOS Apple Silicon builds. Add a **"Scan folder for projects"** mode to the Add Project form so the natural mental model ("I keep my Java repos under `~/Projects`, point the manager there") works without first writing a `.code-workspace` file.

## Requirements

1. **Bug #8 — Auto-downloaded fork jar breaks deployed MCP clients.** When the release-poller auto-downloads a new fork jar (e.g. `javalens-v1.8.0.jar` replaces `javalens-v1.7.1.jar` on disk), deployed MCP service entries in Cursor / Claude / Antigravity / Claude Desktop still reference the prior versioned filename and fail to spawn ("no such file or directory"). Defeats the headline "polls for fork releases and downloads the latest jar automatically" promise.

   **Decision (deferred to Stage 1 design):** Option A (stable filename — `javalens.jar` plus a versioned symlink for diagnostics) is the smaller, more durable change; Option B (auto-redeploy after every download) layers cleanly on top if needed. Land Option A first; Option B only if the smoke surfaces a case Option A misses.

   GitHub issue: [hw1964/javalens-manager#1](https://github.com/hw1964/javalens-manager/issues/1).

2. **Windows installer generation — Windows ARM + Windows x64.** Add `windows-2022` (x64) + `windows-11-arm` (ARM64) to `release.yml`. Tauri's native Windows bundler produces `.msi` (WiX) and/or `.exe` (NSIS) installers; default both so users can pick. Unsigned at first (SmartScreen warning documented in README's Windows install section, same playbook as the macOS Gatekeeper bypass from Sprint 14). Code-signing certificate is a separate later track.

   Two Windows install paths:
   - **(A)** Direct download — `.msi` / `.exe` from the GitHub Release page, double-click.
   - **(B)** Optional PowerShell one-liner if the cost is small (`iwr | iex`-style installer that pulls the right per-arch artifact). Lower priority than (A); cut candidate.

   Platform matrix after Sprint 15:
   - Linux amd64 ✓ (since v0.13.0)
   - Linux arm64 ✓ (since v0.13.1)
   - macOS Apple Silicon ✓ (since v0.14.0, unsigned)
   - macOS Intel — **explicitly NOT supported.** Apple stopped shipping Intel Macs in 2023; Sprint 14 shipped Apple Silicon only. Intel-Mac users can install Rosetta 2 (`softwareupdate --install-rosetta`) and run the Apple Silicon `.dmg` via translation if needed.
   - **Windows x64 (NEW)** — unsigned, SmartScreen bypass documented.
   - **Windows ARM (NEW)** — unsigned, SmartScreen bypass documented.

3. **Add Project UX — single-project OR scan-parent mode.** Today the Browse button on the Register Project form opens a single-folder picker; whatever folder is picked becomes the project root verbatim ([`ProjectForm.svelte:73-84`](../../src/lib/components/ProjectForm.svelte#L73-L84)). Pointing it at `~/Projects` fails because `~/Projects` isn't itself a Java project. The natural mental model — "scan this directory for all the Java projects under it" — has no entry point unless the user first writes a `.code-workspace` file.

   **Sketch:** add a mode toggle (checkbox or radio) above the path input:
   - **Single project** (default — current behavior). Browse picks one folder, becomes the project root.
   - **Scan folder for projects.** Browse picks a parent directory. Submit runs the existing `WalkDir(max_depth=6)` + `detect_java_project_kind` logic from [`manager_service.rs:900`](../../src-tauri/src/manager_service.rs#L900) (currently only reachable via the `discover_workspace_projects` Tauri command, which is gated behind a `.code-workspace` seed). Results render in the same checkbox-list candidate UX already present at the bottom of `ProjectForm.svelte` (the VSCode-workspace import flow). User unchecks any over-discovered junk, clicks Import.

   The candidate-list checkbox UX is the de-junking mechanism: too many results → uncheck the noise → import only what's wanted. This is the existing flow for `.code-workspace` import; the new mode just changes what seeds the scan.

   **Backend reuse:** factor `discover_workspace_projects` into a smaller `scan_directory_for_java_projects(root: PathBuf)` helper. The existing command becomes a thin wrapper that calls the helper for each `.code-workspace` root. A new Tauri command `scan_folder_for_projects(folder)` calls the helper directly.

## Repos touched

- **`javalens-manager` only.** Fork is unchanged.

## Out of scope (settled)

- macOS Intel builds — explicitly NOT in scope, same as Sprint 14. Apple Silicon only on the macOS side. Intel-Mac users run via Rosetta 2 if needed.
- Windows code-signing certificate — separate later track.
- Fork-side work — the v1.8.x backlog items in [`javalens-mcp/docs/upgrade-checklist.md`](https://github.com/hw1964/javalens-mcp/blob/master/docs/upgrade-checklist.md) (M2E-resolved classpath, `copy_class` / `wrap_class` strangler-fig, HTTP/SSE transport, `pre_edit_impact`, raise `find_references` truncation cap, Buildship target-platform integration, `move_class` javadoc `@link` import filter) ship under whichever future fork release picks them up. **Sprint 15 is manager-only.**
- Sprint 14a refactor-tools-must-apply policy retrofit (fork v1.9.0) — independent of Sprint 15.
- The "Strategic discussion — toward Claude as the best Java dev" items in [`future-sprint-enhancements.md`](future-sprint-enhancements.md) (Gradle / Android, target-form catalogs, smell detection, multi-step orchestration) — bigger arc, separate sprint cycle.
- Bug #4+ post-v0.14.1 smoke findings, if any surface, get filed into [`docs/bugs.md`](../bugs.md) and roll into Sprint 15+0 or 15.1; not pre-scoped here.

## Authorship / attribution rule

No Claude / AI / "Generated by …" attribution anywhere — commit messages, PR bodies, release notes, code comments, docs. Same rule that's held since 2026-04-27.

## Workflow rule

Per-stage atomic commits. Focused tests during dev (`npm run test:unit -- <pattern>` for the Svelte side, `cargo test --bin javalens-manager <pattern>` for the Rust side); full reactor / lint / typecheck only at sprint-exit gate. Never push without explicit ask.

## Order of work (suggested)

1. **Stage 0 — Bug #8 fix (Option A: stable filename + symlink).** Smallest change with the highest unblock value: the user is hitting this every fork release. Rework the release-poller's jar-placement step to write `javalens.jar` (or `javalens-mcp.jar` — TBD by `ReleaseManager` naming convention) as the stable target plus `javalens-v<version>.jar` as a symlink for diagnostics. The Deploy-to-agents writer references the stable name. One-shot re-deploy after this fix rolls out updates existing configs; subsequent auto-downloads stop breaking them.

2. **Stage 1 — Backend helper extraction + Scan-folder Tauri command.** Factor `discover_workspace_projects` ([`manager_service.rs:900`](../../src-tauri/src/manager_service.rs#L900)) into a reusable `scan_directory_for_java_projects(root)` helper. Add a new `scan_folder_for_projects(folder)` Tauri command that calls the helper directly. Unit-test the helper against fixture directories.

3. **Stage 2 — Frontend mode toggle in ProjectForm.svelte.** Add the Single project / Scan folder checkbox or radio above the path input. Wire the Scan path: Browse picks a parent → `scan_folder_for_projects` → render results in the existing candidate-list checkbox UX → Import.

4. **Stage 3 — Windows release matrix.** Add `windows-2022` (x64) + `windows-11-arm` (ARM64) jobs to `release.yml`. Verify Tauri's native bundler produces `.msi` / `.exe` per arch. Smoke-install on a Windows VM (or runner artifact download). Update README install section + add a Windows-specific SmartScreen-bypass note mirroring the macOS Gatekeeper one.

5. **Stage 4 — (optional) Windows PowerShell one-liner installer.** Cut candidate. Land if the previous stages have time budget.

6. **Stage 5 — Release v0.15.0.** Version bumps in `package.json` / `Cargo.toml` / `tauri.conf.json`, release notes at `docs/release-notes/v0.15.0.md`, README marketing pivot (mention Windows + Scan-folder mode), tag + push.

## Critical files

- **Bug #8:** `src-tauri/src/release_manager.rs` (jar-placement + symlink), `src-tauri/src/mcp_deploy.rs` or wherever the Deploy-to-agents writer lives — needs a `grep` pass to confirm the exact file (`src-tauri/src/` has ~10 files; the writer that emits MCP client `args` is the target).
- **Add Project UX backend:** [`src-tauri/src/manager_service.rs:900-940`](../../src-tauri/src/manager_service.rs#L900-L940) (`discover_workspace_projects`), [`src-tauri/src/manager_service.rs:2108-2140`](../../src-tauri/src/manager_service.rs#L2108-L2140) (`detect_java_project_kind`), [`src-tauri/src/commands.rs:126-145`](../../src-tauri/src/commands.rs#L126-L145) (Tauri command wrappers).
- **Add Project UX frontend:** [`src/lib/components/ProjectForm.svelte`](../../src/lib/components/ProjectForm.svelte) (mode toggle, Scan path), `src/lib/api/tauri.ts` (new `scanFolderForProjects` wrapper).
- **Windows release:** `.github/workflows/release.yml` (matrix expansion), `src-tauri/tauri.conf.json` (Windows bundle config — `targets`, `wix` / `nsis` sections), `README.md` (install instructions + SmartScreen note).

## Reusable infrastructure already in place

- **`WalkDir` + `detect_java_project_kind` + `is_ignored_candidate_path`** in `manager_service.rs` — already battle-tested by the `.code-workspace` import flow. The new Scan-folder mode is a thin reuse, not new logic.
- **Candidate-list checkbox UX** in `ProjectForm.svelte` (lines 267-291) — already handles select / unselect / import. The Scan-folder mode reuses it verbatim.
- **GitHub Actions release.yml** with the macOS matrix from Sprint 14 — adding Windows is "two more matrix entries + Tauri bundler config" rather than greenfield CI work.
- **`ReleaseManager`** for the auto-download lifecycle — the stable-filename + symlink change lands inside its existing jar-placement step, not a new subsystem.

## Verification (sprint exit)

- Bug #8: after fix lands, manually trigger a fork release auto-download (or simulate via `release_manager` test), confirm deployed MCP client configs still spawn the server.
- Windows builds: artifacts produced by CI for both ARM + x64; smoke-install on a real Windows machine (or a runner artifact download) and start a workspace.
- Add Project UX: select a parent directory in Scan mode; verify the candidate list populates with detected Maven / Gradle / Eclipse projects under it; uncheck noise; import; confirm imports appear in the dashboard.
- All existing tests stay GREEN — focused per stage during dev, full `npm test` + `cargo test` at exit.

## Cut lines (if a stage hits unexpected pain)

- **Stage 4 (PowerShell one-liner)** is the first cut — Windows users can download the `.msi` / `.exe` directly. Sprint still ships Windows as a platform without it.
- **Stage 2 (frontend toggle)** can split into two: ship the backend command first (Stage 1) + the frontend toggle as a v0.15.1 patch if the UX needs more design iteration than the sprint window allows.
- **Stage 0 (Bug #8)** is the floor — must ship. Without it the v0.15.0 release itself triggers the bug it's trying to fix.

## Build / test commands

```bash
# Manager
cd /home/harald/CursorProjects/javalens-manager
npm install
npm run check               # type / svelte-check
npm run test:unit -- <pattern>   # focused frontend unit tests
cd src-tauri && cargo test <pattern>   # focused Rust tests
cd .. && npx tauri dev      # live smoke
npx tauri build             # local build sanity

# Full sprint-exit verify
npm run check && npm run test:unit && (cd src-tauri && cargo test) && npx tauri build
```

## Definition of Done

- [ ] Bug #8 FIXED in v0.15.0. Verified by simulating an auto-download mid-session and confirming deployed clients still spawn.
- [ ] `release.yml` Windows matrix ships `.msi` + `.exe` for both ARM + x64. Artifacts attached to the v0.15.0 GitHub Release.
- [ ] Add Project form has a Scan folder mode. Selecting `~/Projects` (or any parent dir) shows discovered Java projects in the checkbox list; user can prune + import.
- [ ] `docs/release-notes/v0.15.0.md` written, covering all three areas.
- [ ] README install section updated: Windows install path + SmartScreen note + Scan-folder UX mention.
- [ ] Tag `v0.15.0` pushed; CI publishes the GitHub Release as Latest.
- [ ] No AI-attribution boilerplate anywhere.
- [ ] `MEMORY.md` + `project_sprint_state.md` flipped to "Sprint 15 closed, v0.15.0 shipped".
