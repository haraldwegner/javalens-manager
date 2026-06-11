# Sprint 16 Backlog

> **Status: drafted 2026-06-07; scope expanded 2026-06-12.** Originally these two requirements were part of Sprint 15 ([`sprint-15-backlog.md`](sprint-15-backlog.md)), but Sprint 15 re-scoped to bug #8 + bug #9 PRIMARY fix end-to-end (coupled with fork [Sprint 14a](../../../javalens-mcp/docs/sprints/sprint-14a-http-sse-transport.md) → v1.8.5 HTTP/SSE default). Windows installer + scan-folder pushed here. **2026-06-12 expansion: absorbs the superseded [v0.15.2 hotfix scope](sprint-15.2-hotfix-backlog.md) — bugs #10–#14 ship here instead of a standalone patch.**
>
> Targets manager **v0.16.0**. Manager-only sprint; the GOJA rebrand and fork-side architectural work proceed independently. Actionable plan: `~/.claude/plans/make-a-plan-happy-fern.md` (Phase B).

## Goal

Ship **manager v0.16.0** as Latest on GitHub. Three requirement groups:

1. **Expand the platform matrix to Windows** — Windows ARM + Windows x64 alongside the existing Linux amd64 / Linux arm64 / macOS Apple Silicon builds. Adds the missing major desktop OS.
2. **Add "Scan folder for projects" to the Add Project form** — fixes the UX gap surfaced during Sprint 14 live smoke ("I keep my Java repos under `~/Projects`, point the manager there" doesn't work without first writing a `.code-workspace` file). **UX decided 2026-06-11 ("mixture to save space"): no separate mode section — a "Recursive search (autoscan)" checkbox under the Project path field; when checked, Browse autoscans the picked folder, the submit button relabels to "Discover" (for hand-typed paths/rescans), results unfold inline into the existing candidate checkbox-list, Name disables.**
3. **Lifecycle + deploy-correctness bug bundle (absorbed from v0.15.2)** — [bugs #10–#14](../bugs.md): per-client MCP writer schema (Antigravity `serverUrl`), `rename_workspace` migrates the port/token entry, delete paths call `release_workspace_state`, `QuitAction::Quit` stops residents, one-shot `workspaces[]` orphan prune (5 known live orphans), deploy-config staleness on workspace mutations + the silent `.ok()?` drop in `build_deploy_servers`.

## Requirements

1. **Windows installer generation — Windows ARM + Windows x64.** Add `windows-2022` (x64) + `windows-11-arm` (ARM64) to `release.yml`. Tauri's native Windows bundler produces `.msi` (WiX) and/or `.exe` (NSIS) installers; default both so users can pick. Unsigned at first (SmartScreen warning documented in README's Windows install section, same playbook as the macOS Gatekeeper bypass from Sprint 14). Code-signing certificate is a separate later track.

   Two Windows install paths:
   - **(A)** Direct download — `.msi` / `.exe` from the GitHub Release page, double-click.
   - **(B)** Optional PowerShell one-liner if the cost is small (`iwr | iex`-style installer that pulls the right per-arch artifact). Lower priority than (A); cut candidate.

   Platform matrix after Sprint 16:
   - Linux amd64 ✓ (since v0.13.0)
   - Linux arm64 ✓ (since v0.13.1)
   - macOS Apple Silicon ✓ (since v0.14.0, unsigned)
   - macOS Intel — **explicitly NOT supported.** Apple stopped shipping Intel Macs in 2023; Sprint 14 shipped Apple Silicon only. Intel-Mac users can install Rosetta 2 (`softwareupdate --install-rosetta`) and run the Apple Silicon `.dmg` via translation if needed.
   - **Windows x64 (NEW)** — unsigned, SmartScreen bypass documented.
   - **Windows ARM (NEW)** — unsigned, SmartScreen bypass documented.

2. **Add Project UX — single-project OR scan-parent mode.** Today the Browse button on the Register Project form opens a single-folder picker; whatever folder is picked becomes the project root verbatim ([`ProjectForm.svelte:73-84`](../../src/lib/components/ProjectForm.svelte#L73-L84)). Pointing it at `~/Projects` fails because `~/Projects` isn't itself a Java project. The natural mental model — "scan this directory for all the Java projects under it" — has no entry point unless the user first writes a `.code-workspace` file.

   **Sketch:** add a mode toggle (checkbox or radio) above the path input:
   - **Single project** (default — current behavior). Browse picks one folder, becomes the project root.
   - **Scan folder for projects.** Browse picks a parent directory. Submit runs the existing `WalkDir(max_depth=6)` + `detect_java_project_kind` logic from [`manager_service.rs:900`](../../src-tauri/src/manager_service.rs#L900) (currently only reachable via the `discover_workspace_projects` Tauri command, which is gated behind a `.code-workspace` seed). Results render in the same checkbox-list candidate UX already present at the bottom of `ProjectForm.svelte` (the VSCode-workspace import flow). User unchecks any over-discovered junk, clicks Import.

   The candidate-list checkbox UX is the de-junking mechanism: too many results → uncheck the noise → import only what's wanted. This is the existing flow for `.code-workspace` import; the new mode just changes what seeds the scan.

   **Backend reuse:** factor `discover_workspace_projects` into a smaller `scan_directory_for_java_projects(root: PathBuf)` helper. The existing command becomes a thin wrapper that calls the helper for each `.code-workspace` root. A new Tauri command `scan_folder_for_projects(folder)` calls the helper directly.

## Repos touched

- **`javalens-manager` only.**

## Out of scope (settled)

- **macOS Intel builds** — explicitly NOT in scope, same as Sprint 14/15. Apple Silicon only on the macOS side. Intel-Mac users run via Rosetta 2 if needed.
- **Windows code-signing certificate** — separate later track. v0.16.0 ships unsigned with the SmartScreen-bypass docs.
- **GOJA rebrand** — separate sprint that crosses both repos. Doesn't block Sprint 16.
- **Bug #9 primary fix (HTTP/SSE shared-service hosting)** — depends on fork Sprint 14a → v1.8.5. Sprint 17+ work.

## Authorship / attribution rule

No Claude / AI / "Generated by …" attribution anywhere — commit messages, PR bodies, release notes, code comments, docs. Same rule that's held since 2026-04-27.

## Workflow rule

Per-stage atomic commits. Focused tests during dev (`npm run test:unit -- <pattern>` for the Svelte side, `cargo test --bin javalens-manager <pattern>` for the Rust side); full reactor / lint / typecheck only at sprint-exit gate. Never push without explicit ask.

## Order of work (suggested)

1. **Stage 0 — Backend helper extraction + Scan-folder Tauri command.** Factor `discover_workspace_projects` ([`manager_service.rs:900`](../../src-tauri/src/manager_service.rs#L900)) into a reusable `scan_directory_for_java_projects(root)` helper. Add a new `scan_folder_for_projects(folder)` Tauri command that calls the helper directly. Unit-test the helper against fixture directories.

2. **Stage 1 — Frontend mode toggle in ProjectForm.svelte.** Add the Single project / Scan folder checkbox or radio above the path input. Wire the Scan path: Browse picks a parent → `scan_folder_for_projects` → render results in the existing candidate-list checkbox UX → Import.

3. **Stage 2 — Windows release matrix.** Add `windows-2022` (x64) + `windows-11-arm` (ARM64) jobs to `release.yml`. Verify Tauri's native bundler produces `.msi` / `.exe` per arch. Smoke-install on a Windows VM (or runner artifact download). Update README install section + add a Windows-specific SmartScreen-bypass note mirroring the macOS Gatekeeper one.

4. **Stage 3 — (optional) Windows PowerShell one-liner installer.** Cut candidate. Land if the previous stages have time budget.

5. **Stage 4 — Release v0.16.0.** Version bumps in `package.json` / `Cargo.toml` / `tauri.conf.json`, release notes at `docs/release-notes/v0.16.0.md`, README marketing pivot (mention Windows + Scan-folder mode), tag + push.

## Critical files

- **Add Project UX backend:** [`src-tauri/src/manager_service.rs:900-940`](../../src-tauri/src/manager_service.rs#L900-L940) (`discover_workspace_projects`), [`src-tauri/src/manager_service.rs:2108-2140`](../../src-tauri/src/manager_service.rs#L2108-L2140) (`detect_java_project_kind`), [`src-tauri/src/commands.rs:126-145`](../../src-tauri/src/commands.rs#L126-L145) (Tauri command wrappers).
- **Add Project UX frontend:** [`src/lib/components/ProjectForm.svelte`](../../src/lib/components/ProjectForm.svelte) (mode toggle, Scan path), `src/lib/api/tauri.ts` (new `scanFolderForProjects` wrapper).
- **Windows release:** `.github/workflows/release.yml` (matrix expansion), `src-tauri/tauri.conf.json` (Windows bundle config — `targets`, `wix` / `nsis` sections), `README.md` (install instructions + SmartScreen note).

## Reusable infrastructure already in place

- **`WalkDir` + `detect_java_project_kind` + `is_ignored_candidate_path`** in `manager_service.rs` — already battle-tested by the `.code-workspace` import flow. The new Scan-folder mode is a thin reuse, not new logic.
- **Candidate-list checkbox UX** in `ProjectForm.svelte` (lines 267-291) — already handles select / unselect / import. The Scan-folder mode reuses it verbatim.
- **GitHub Actions release.yml** with the macOS matrix from Sprint 14 — adding Windows is "two more matrix entries + Tauri bundler config" rather than greenfield CI work.

## Verification (sprint exit)

- **Windows builds:** artifacts produced by CI for both ARM + x64; smoke-install on a real Windows machine (or a runner artifact download) and start a workspace.
- **Add Project UX:** select a parent directory in Scan mode; verify the candidate list populates with detected Maven / Gradle / Eclipse projects under it; uncheck noise; import; confirm imports appear in the dashboard.
- All existing tests stay GREEN — focused per stage during dev, full `npm test` + `cargo test` at exit.

## Cut lines (if a stage hits unexpected pain)

- **Stage 3 (PowerShell one-liner)** is the first cut — Windows users can download the `.msi` / `.exe` directly. Sprint still ships Windows as a platform without it.
- **Stage 1 (frontend toggle)** can split into two: ship the backend command first (Stage 0) + the frontend toggle as a v0.16.1 patch if the UX needs more design iteration than the sprint window allows.

## Build / test commands

```bash
# Manager
cd /home/harald/CursorProjects/javalens-manager
npm install
npm run check
npm run test:unit -- <pattern>
cd src-tauri && cargo test <pattern>
cd .. && npx tauri dev
npx tauri build

# Full sprint-exit verify
npm run check && npm run test:unit && (cd src-tauri && cargo test) && npx tauri build
```

## Definition of Done

- [ ] `release.yml` Windows matrix ships `.msi` + `.exe` for both ARM + x64. Artifacts attached to the v0.16.0 GitHub Release.
- [ ] Add Project form has a Scan folder mode. Selecting `~/Projects` (or any parent dir) shows discovered Java projects in the checkbox list; user can prune + import.
- [ ] `docs/release-notes/v0.16.0.md` written, covering both areas.
- [ ] README install section updated: Windows install path + SmartScreen note + Scan-folder UX mention.
- [ ] Tag `v0.16.0` pushed; CI publishes the GitHub Release as Latest.
- [ ] No AI-attribution boilerplate anywhere.
- [ ] `MEMORY.md` + `project_sprint_state.md` updated to "Sprint 16 closed, v0.16.0 shipped".
