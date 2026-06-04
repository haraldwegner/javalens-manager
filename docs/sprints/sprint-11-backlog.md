# Sprint 11 Backlog (draft)

> **Status: draft, updated 2026-04-26 after Sprint 10 Phase A landed as fork `v1.3.0`.** This sprint extends the multi-project `WorkspaceManager` shipped in v1.3.0 with detection-matrix completion AND tool-surface consolidation. Targets fork `v1.5.0` (was originally `v1.3.0` in earlier draft — re-versioned to follow Sprint 10's actual release).

## Goal

Four threads, one release:

1. **Detection-matrix completion** — any project type loaded into a workspace gets fully indexed (sources + dependencies) regardless of build system. Workspace bundle pool resolves `Require-Bundle` between sibling PDE bundles loaded together. Gradle resolution moves from heuristic to proper.
2. **Tool-surface consolidation** — collapse ~13 narrow `find_X` tools into 2 parametric tools so the per-service tool count drops from 66 to ~55, freeing budget for the next thread.
3. **Structural refactoring (Ring 1, pulled forward from Sprint 12)** — five JDT LTK-backed refactoring tools: `move_class`, `move_package`, `pull_up`, `push_down`, `encapsulate_field`. Move-class and repackaging are the most error-prone operations in agent-driven Java work; without first-class refactoring the agent has to do find-and-replace across imports and qualified names by hand. With LTK behind these tools, javalens-mcp + javalens-manager become a professional-grade autonomous-Java-development toolset that goes beyond what upstream `pzalutski-pixel/javalens-mcp` aims for.
4. **Cutover** — cut fork release `v1.5.0`, ship manager `v0.11.0` (version bumps + README/Help.md updates for the new tool surface).

End-of-sprint outcome:

- A workspace can hold any mix of regular Maven modules, Maven-Tycho/PDE bundles, pure Eclipse PDE bundles, and Gradle modules, and every project type returns correct source roots + dependencies.
- Cross-bundle navigation, find-references, and refactoring work between PDE bundles in the same workspace — including LTK-backed `pull_up` across OSGi bundle boundaries.
- javalens-mcp registers ~60 tools per service (66 → 55 from Phase D consolidation, then +5 from Phase E refactorings) — leaves ~40 slots in the agent's 100-tool budget for Ring 2+ work without future pressure.
- Fork release `v1.5.0` published; manager-side single-workspace mode becomes the default.

Reference plan: `~/.claude/plans/make-a-plan-happy-fern.md`. Predecessor: `docs/sprints/sprint-10-backlog.md` (multi-project workspace, shipped as v1.3.0).

## Problem Statement

Today's detection matrix (post-Sprint 9):

| Layout | Source roots | Dependencies |
|---|---|---|
| Regular Maven (pom.xml) | `<sourceDirectory>` override + heuristic | `mvn dependency:build-classpath` shell-out |
| Maven-Tycho (pom.xml with `<packaging>eclipse-*</packaging>`) | `.classpath` | **broken**: `mvn dependency:build-classpath` returns wrong/empty results — Tycho deps come from `MANIFEST.MF` + target platform, not pom `<dependencies>` |
| Pure Eclipse PDE (`.classpath` + `MANIFEST.MF`, no pom) | `.classpath` | `.classpath kind="lib"` only — `Require-Bundle` references stay unresolved |
| Gradle (build.gradle / .kts) | heuristic `src/main/java` only | **broken**: just walks `build/classes/java/...`, no real extraction |

Three of the four cases have correctness gaps. Sprint 11 closes them.

## Repos touched

- **`javalens-mcp` (fork)**: Tycho packaging detection, `MANIFEST.MF` parsing helpers, workspace bundle pool, Gradle Tooling API integration, **tool-surface consolidation (Phase E, new)**. Cut release `v1.5.0`.
- **`javalens-manager`**: cut a manager version (`v0.11.0`) targeting fork `v1.5.0` — version bumps and Help.md / README updates for the new tool surface and refactoring tools. No Rust or Svelte code changes required from Phases A–E.

## Phase A — Tycho-aware Maven detection

### A.1 Detect Tycho packaging

File: `org.javalens.core/.../project/ProjectImporter.java`.

- New helper `readPomPackaging(Path pomXml) -> Optional<String>`. Pure DOM, returns the `<packaging>` text (`jar`, `pom`, `eclipse-plugin`, `eclipse-repository`, `eclipse-feature`, `eclipse-test-plugin`, …).
- Extend `BuildSystem` enum or add a sibling field `boolean isTycho` on the result. Tycho packagings: `eclipse-plugin`, `eclipse-test-plugin`, `eclipse-feature`, `eclipse-repository`, `eclipse-update-site`, `eclipse-target-definition`.

### A.2 Skip Maven shell-out for Tycho projects

In `addDependencyEntries`:

- When pom packaging is a Tycho type, **do not** call `getMavenDependencies` — its result is misleading (Tycho injects classpath via target platform; `dependency:build-classpath` misses most of it).
- Rely entirely on `.classpath kind="lib"` (Sprint 9 ✓) plus `Require-Bundle` resolved against the workspace bundle pool (Phase B below).

### A.3 Tests

`ProjectImporterTycho_PackagingTest`:
- `readPomPackaging_returnsEclipsePlugin` — fixture pom with `<packaging>eclipse-plugin</packaging>`.
- `readPomPackaging_returnsJar_byDefault` — pom without `<packaging>`.
- `tycho_skipsMavenShellOut` — fixture Tycho pom + .classpath; `addDependencyEntries` does not invoke `mvn dependency:build-classpath`.

## Phase B — Workspace bundle pool (`Require-Bundle` resolution)

This is the original Sprint 11 core. Helps PDE bundles in any flavor (Tycho or pure) loaded into the same workspace.

### B.1 MANIFEST.MF parsing helpers

In `ProjectImporter.java`:

- `readManifestSymbolicName(Path projectRoot) -> Optional<String>` — read `META-INF/MANIFEST.MF`, parse `Bundle-SymbolicName`, strip `;singleton:=true` and other directives. **Portable.**
- `readManifestRequireBundle(Path projectRoot) -> List<String>` — parse semicolon-delimited `Require-Bundle` header with version/visibility directives, return list of bundle symbolic names. **Portable.**

Use `java.util.jar.Manifest` for proper line-continuation handling.

### B.2 Bundle map in `WorkspaceManager`

Extend Sprint 10's multi-project `WorkspaceManager`:

```java
private final Map<String /*symbolicName*/, String /*projectKey*/> bundleSymbolicNameToProjectKey = new ConcurrentHashMap<>();

public void registerBundle(String symbolicName, String projectKey);
public Optional<IJavaProject> resolveBundle(String symbolicName);
```

`AddProjectTool` (Sprint 10) and `LoadProjectTool` call `readManifestSymbolicName` after registering each project; if present, register the bundle in the map.

### B.3 Wire `Require-Bundle` into dependency resolution

In `ProjectImporter.addDependencyEntries`, after pom + `.classpath` lib resolution:

```java
List<String> requires = readManifestRequireBundle(projectRoot);
for (String requiredBundle : requires) {
    workspaceManager.resolveBundle(requiredBundle).ifPresentOrElse(
        sibling -> entries.add(JavaCore.newProjectEntry(sibling.getPath())),
        () -> log.debug("Require-Bundle {} not found in workspace; skipping", requiredBundle)
    );
}
```

Bundles outside the workspace (e.g., resolved by an external Eclipse target platform) remain unresolved — accepted gap (see Outlook).

### B.4 Tests

`ProjectImporterBundlePoolTest`:
- `readManifestSymbolicName_stripsDirectives` — input `com.foo;singleton:=true` → `com.foo`.
- `readManifestRequireBundle_parsesMultiLineHeader` — Manifest with continuation lines.
- `requireBundle_resolvesWithinWorkspace` — load 2 fixture bundles A → B; A's classpath includes B as a project entry.
- `requireBundle_outsideWorkspace_logsAndContinues` — A requires com.example.unknown; load succeeds with warning.

## Phase C — Gradle Tooling API integration

### C.1 Add Gradle Tooling API dependency

- Add `org.gradle:gradle-tooling-api` (~5 MB) as a runtime dependency in `org.javalens.core/META-INF/MANIFEST.MF` and the fork's target platform.
- Bundle it in the RCP product (`plugins/`).

### C.2 Replace heuristic with Tooling API

In `ProjectImporter`:

- New `getGradleProjectModel(Path projectRoot) -> Optional<GradleProjectInfo>` (DTO with `srcPaths`, `libPaths`, `outputPath`). Uses `GradleConnector.newConnector().forProjectDirectory(...).connect()` and queries the `EclipseProject` model (which Gradle Tooling API exposes — already designed for IDE consumption).
- Replace the existing `getGradleDependencies` heuristic with this. Source roots come from `EclipseProject.getSourceDirectories()`; dependencies from `EclipseProject.getClasspath()` resolving to jar files.

### C.3 Tests

`ProjectImporterGradleToolingTest`:
- Add a minimal Gradle fixture (`org.javalens.core.tests/test-resources/sample-projects/simple-gradle/`).
- `gradle_returnsActualSourceSets` — extracts `src/main/java` and `src/test/java` from the model.
- `gradle_returnsActualDependencies` — finds the resolved jar paths for declared `implementation`/`testImplementation`.
- `gradle_customSrcDir` — fixture with `sourceSets.main.java.srcDirs = ['custom-src']` resolves correctly.

## Phase D — Tool-surface consolidation

### Why now

Per-service tool count is 66 today (5 admin + 61 analysis). Single-workspace mode means a healthy 1-workspace setup registers 66 tools against the agent's 100-tool budget. With GitKraken's MCP shim removed (decided 2026-04-26 — `git`/`gh` via Bash already covers everything an autonomous agent needs), 34 slots remain free. Phase D reclaims ~10 more by collapsing redundant `find_X` tools, leaving ~44 free for Phase E refactoring additions and Sprint 12+ work.

### Consolidation targets

Two parametric tools replace ~13 narrow ones:

**`find_pattern_usages(kind, query, ...)`** — replaces:
- `find_annotation_usages(annotationName)` → `kind=annotation`
- `find_type_instantiations(typeName)` → `kind=instantiation`
- `find_type_arguments(typeName)` → `kind=type_argument`
- `find_casts(typeName)` → `kind=cast`
- `find_instanceof_checks(typeName)` → `kind=instanceof`

**`find_quality_issue(kind, ...)`** — replaces:
- `find_naming_violations()` → `kind=naming`
- `find_possible_bugs()` → `kind=bugs`
- `find_unused_code()` → `kind=unused`
- `find_large_classes()` → `kind=large_classes`
- `find_circular_dependencies()` → `kind=circular_deps`
- `find_reflection_usage()` → `kind=reflection`
- `find_throws_declarations()` → `kind=throws`
- `find_catch_blocks()` → `kind=catches`

**Kept as-is** (different parameter shapes — collapsing them hurts more than helps):
- `find_references(filePath, line, column)` — position-anchored
- `find_implementations(filePath, line, column)` — position-anchored
- `find_method_references(...)` — anchor-based
- `find_field_writes(...)` — anchor-based
- `find_tests(typeName)` — separate semantic

### Net change

13 tools collapse to 2. Tool count drops 66 → 55. Schema size grows (each parametric tool's `kind` enum is ~8 entries with descriptions) but agent context shrinks overall.

### D.1 Define the parametric tools

Files: new `org.javalens.mcp/src/org/javalens/mcp/tools/FindPatternUsagesTool.java`, `FindQualityIssueTool.java`. Each takes `kind` as a required string enum and dispatches to the existing search/analysis methods underneath. **Reuses existing `SearchService` + analysis logic** — only the tool boundary changes.

### D.2 Schema with discoverable `kind` enum

Each tool's input schema lists allowed `kind` values + per-kind descriptions so agents know what's available. Example:

```json
{
  "kind": {
    "type": "string",
    "enum": ["annotation", "instantiation", "type_argument", "cast", "instanceof"],
    "description": "What pattern to find. annotation: usage sites of @X. instantiation: 'new T(...)' calls. type_argument: 'List<T>' style usage. cast: '(T)' casts. instanceof: 'instanceof T' checks."
  }
}
```

Optional kind-specific parameters (e.g. `annotationName` for annotation kind, `query` for type-anchored kinds) are documented in description text.

### D.3 Deprecate-and-delete the 13 narrow tools

Remove the old `Find*Tool.java` files in the same v1.5.0 release. **Breaking change for any external MCP client that wasn't using javalens-manager.** Acceptable — javalens-manager is the only known consumer; the npm/MCP-Registry surface (which the fork doesn't publish to anyway, see v1.2.1's release-workflow strip) means no third-party tooling depends on these names.

### D.4 Update tool registrations

In `JavaLensApplication.registerTools()`: drop the 13 register lines, add the 2 new ones. Tool count goes from 66 to 55.

### D.5 Tests

`FindPatternUsagesToolTest`, `FindQualityIssueToolTest`:
- One test per `kind` value verifying it dispatches to the right underlying search and returns the same shape the old narrow tool did. ~13 small tests.
- One test per tool verifying invalid `kind` returns an `INVALID_PARAMETER` error.

## Phase E — Structural refactoring tools (Ring 1)

### Why this is part of Sprint 11, not deferred

Move-class and repackaging are the most error-prone operations in agent-driven Java work. Without first-class JDT-backed refactoring, the agent has to do find-and-replace across imports, qualified-name strings, and class-file paths by hand — every multi-bundle RCP overhaul, bundle merge, package rename, or hierarchy reshuffle fights the agent. With JDT LTK behind these tools, the agent gets the same atomic, reference-aware refactoring Eclipse/IntelliJ users have had for two decades. Combined with javalens-manager, this turns javalens-mcp into an actually professional-grade tool for autonomous Java development — a level upstream is unlikely to reach because they don't have the use case driving it.

### What's in Ring 1

Five JDT LTK-backed refactoring tools. Each is a thin tool wrapper around `org.eclipse.jdt.core.refactoring.descriptors.*` + `RefactoringCore` apply.

| Tool | JDT descriptor | What it does |
|---|---|---|
| `move_class` | `MoveDescriptor` (`IJavaRefactorings.MOVE`) | Move one or more types to a different package. Updates all `import` lines, qualified references, and the file's location on disk. Honors `.classpath` source folders. |
| `move_package` | `RenameDescriptor` (`IJavaRefactorings.RENAME_PACKAGE`) with new package name | Move/rename a whole package, recursing into all CUs. Updates `package` declarations and all references workspace-wide. |
| `pull_up` | `PullUpDescriptor` (`IJavaRefactorings.PULL_UP`) | Move a method or field from a subtype up to a supertype. Optionally turns the original into an abstract declaration. |
| `push_down` | `PushDownDescriptor` (`IJavaRefactorings.PUSH_DOWN`) | Move a method or field from a supertype into one or more subtypes. Removes the original. |
| `encapsulate_field` | `EncapsulateFieldDescriptor` (`IJavaRefactorings.ENCAPSULATE_FIELD`) | Generate getter/setter for a field, replace all direct accesses with the new methods, optionally tighten the field's visibility. |

After Phase E: 55 (post-Phase D) + 5 = **60 tools per service**. Headroom against the 100 cap stays at ~40.

### E.0 Shared `AbstractRefactoringTool` base class

File: `org.javalens.mcp/src/org/javalens/mcp/tools/AbstractRefactoringTool.java`. Encapsulates the LTK plumbing shared across all five tools so each individual tool stays small.

```java
abstract class AbstractRefactoringTool extends AbstractTool {
    protected ToolResponse runRefactoring(
        IJdtService service,
        RefactoringDescriptor descriptor,
        String operationLabel
    ) {
        // 1. descriptor.createRefactoring(status)
        // 2. checkInitialConditions(monitor)  — bail with INVALID_PARAMETER on ERROR severity
        // 3. checkFinalConditions(monitor)    — bail similarly
        // 4. createChange(monitor) → Change
        // 5. PerformChangeOperation.run(...)
        // 6. Collect modified ICompilationUnits, format paths via service.getPathUtils()
        // 7. Return success { modifiedFiles, summary } or invalidParameter / refactoringFailed
    }
}
```

The tools then become ~80–120 LOC each: parse arguments, build the JDT descriptor, call `runRefactoring`. The same base also handles dirty-buffer detection, conflict diagnostics, and rollback on partial failure.

### E.1 `move_class`

```
Input:
  filePath: string         (source file containing the type)
  line, column: integer    (zero-based, points anywhere inside the type)
  targetPackage: string    (e.g. "com.example.api")
  updateReferences: bool   (default true)

Output:
  modifiedFiles: [{filePath, summary}]
  newFilePath: string (where the file ended up)
```

Internals: `service.getTypeAtPosition(filePath, line, column)` → `IType`, then build `MoveDescriptor` with `setMoveResources(new IResource[]{type.getResource()})` + target package handle. Reuses A.4.2's `ScopedJdtService` so the agent can scope to one project via `projectKey` if a class name collides across projects.

### E.2 `move_package`

```
Input:
  packageName: string                  (e.g. "com.example.old")
  newPackageName: string               (e.g. "com.example.new")
  updateReferences: bool   (default true)
```

Internals: `service.getJavaProject().findPackageFragment(...)` → `IPackageFragment`. `RenameDescriptor.setProject(...)`, `setNewName(newPackageName)`, `setUpdateReferences(true)`.

### E.3 `pull_up`

```
Input:
  filePath, line, column   (positions to a method/field in a subtype)
  targetSuperType?: string (FQ name; default = direct superclass)
  abstractInOriginal: bool (default true for methods, false for fields)
```

Internals: `getElementAtPosition(...)` → `IMember`, build `PullUpDescriptor` with `setSubtype(declaringType)`, `setMembersToMove(new IMember[]{member})`, target supertype lookup via `service.findType(targetSuperType)`. If the supertype is in another project in the workspace — fine, the workspace-scoped `IJavaSearchScope` from A.4.1 already lets JDT see across project boundaries.

### E.4 `push_down`

```
Input:
  filePath, line, column   (positions to a method/field in a supertype)
  targetSubTypes?: [string] (FQ names; default = all direct subtypes)
  removeFromOriginal: bool (default true)
```

Internals: `PushDownDescriptor`. Pre-flight check: enumerate subtypes via JDT `ITypeHierarchy` and warn if any are read-only (in a binary jar, not source).

### E.5 `encapsulate_field`

```
Input:
  filePath, line, column   (positions to a field declaration or reference)
  getterName?: string      (default: "get" + capitalized name; "is" for boolean)
  setterName?: string      (default: "set" + capitalized name)
  newFieldVisibility: string (default "private"; one of public|protected|private|package)
  generateJavadoc: bool    (default false)
```

Internals: `EncapsulateFieldDescriptor.setField(field)`, `setGetterName(...)`, `setSetterName(...)`, `setVisibility(...)`. JDT handles the ref-to-getter rewrite, but the agent still gets a list of modified files for verification.

### E.6 `projectKey` semantics for refactorings

All five tools accept the optional `projectKey` parameter (inherited from `AbstractTool`). When set, the refactoring is constrained to that project's scope — important for `pull_up`/`push_down` where the agent doesn't want to accidentally drag unrelated workspace projects into the change set. When omitted, refactorings run with workspace-wide scope (the natural default for cross-bundle refactorings, e.g. moving a class out of one OSGi bundle into another).

### E.7 Tests

Per refactoring tool, three tests minimum (~5 × 3 = 15 tests):

1. **Happy path** — apply the refactoring against a fixture; assert modified files and verify with a follow-up search/parse that references resolve.
2. **Validation error** — invalid input (e.g. `move_class` with target package that doesn't exist, or `encapsulate_field` against a non-field) returns `INVALID_PARAMETER` with a clear message.
3. **Conflict / safety** — refactoring would break compilation (e.g. `move_class` with target package containing a same-named class) returns `REFACTORING_FAILED` with the LTK status diagnostics, and **no files are modified**.

Plus 1 integration test: `pullUp_acrossOsgiBundles` — fixture with two PDE bundles, pull a method from a subtype in bundle B up to a supertype in bundle A. Verifies workspace-scoped refactoring crosses bundle boundaries (relies on Phase B's bundle pool).

### Out of scope for Phase E (deferred to Sprint 12 or later)

These are also Eclipse/IntelliJ standards but not required for the large RCP refactoring use case this sprint:

- `convert_local_to_field` / `convert_field_to_local`
- `replace_constructor_with_factory` / `inline_constructor`
- `generalize_type_parameter` (replace concrete type with interface in declarations)
- `introduce_parameter` / `remove_parameter` (we have `change_method_signature` already; explicit single-axis ops are easier for agents but not critical)
- `introduce_parameter_object`
- `replace_inheritance_with_delegation` / vice versa
- Bulk JDK-API migration (var, switch expressions, sealed types, records, text blocks)
- Workspace verification helpers (`compile_workspace`, `run_tests`) — Ring 1 of the IDE roadmap, but they sit at the boundary between refactoring and verification; deferring keeps Sprint 11 sized.

### Phase E size warning

Phase E makes Sprint 11 substantially bigger than originally planned (was ~2 weeks for detection-matrix + cutover; now ~3 weeks with 5 LTK refactorings added). Acceptable because the RCP refactoring use case depends on these tools and "do them in Sprint 12" would mean an extra fork release for one feature set. If the sprint runs over, the natural cut line is to ship Phases A/B/C/D as `v1.5.0` first, then a fast follow-up `v1.5.1` with Phase E once the LTK plumbing settles.

## Phase F — Cutover release (fork v1.5.0 + manager v0.11.0)

### F.1 Tag fork v1.5.0

Bump pom + MANIFEST.MF qualifiers as needed; `git tag -a v1.5.0 -F docs/release-notes/v1.5.0.md`; push tag → CI publishes the GitHub release. Release notes cover Phases A/B/C (detection matrix), Phase D (tool consolidation), and Phase E (refactoring tools) under one curated note.

### F.2 Manager release (v0.11.0)

Bump versions in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` to `0.11.0`. Tag and push.

Documentation updates in `src/assets/help.md`:

- The consolidated `find_*` tool surface (Phase D) — drop references to the old narrow tools, add a paragraph on `find_pattern_usages(kind, ...)` and `find_quality_issue(kind, ...)`.
- The new structural refactoring tools (Phase E) — short paragraph noting `move_class`, `move_package`, `pull_up`, `push_down`, `encapsulate_field` exist and what they do.
- **Help screenshot refresh** in `public/help/` — `dashboard.png`, `settings-top.png`, `settings-bottom.png` currently show pre-v0.10.6 UI. Replace with current captures (workspaces card on top of left column, status-lamp colors, Diagnostics workspace counts).

README gets the same one-paragraph summary linking to the v1.5.0 release notes.

No Rust or Svelte code changes are needed for the cutover itself. Sprint 10 already removed the legacy `single_workspace_mode` flag and the per-port concept; workspace mode is the only mode and has been since `v0.10.4`. Manager-side UX is settled for now — the few "bigger ideas" originally captured under a Phase G workstream were either shipped early in `v0.10.5`/`v0.10.6` or deferred to the networked-service track ([`sprint-future-networked-service.md`](sprint-future-networked-service.md)).

## Tests rollup

- Phase A: `ProjectImporterTychoPackagingTest` (3 tests).
- Phase B: `ProjectImporterBundlePoolTest` (4 tests). Add 2 fixture PDE bundles.
- Phase C: `ProjectImporterGradleToolingTest` (3 tests). Add `simple-gradle` fixture.
- Phase D: `FindPatternUsagesToolTest` + `FindQualityIssueToolTest` (~13 dispatch tests + 2 invalid-kind tests).
- Phase E: per-tool happy/validation/conflict trio (~15 tests) + 1 cross-bundle integration test (`pullUp_acrossOsgiBundles`).
- Plus regression suite from Sprint 9 + Sprint 10 must stay green (381 tests today).

## Definition of Done

- [ ] All four detection-matrix cells return correct source roots + deps for their respective fixtures.
- [ ] Cross-bundle find-references works across two PDE bundles loaded into one workspace.
- [ ] Tool count per service drops to ~55 (from 66 in v1.3.0); after Phase E adds 5 refactoring tools, lands at ~60.
- [ ] All five Phase E refactoring tools (`move_class`, `move_package`, `pull_up`, `push_down`, `encapsulate_field`) pass happy/validation/conflict tests; cross-bundle `pull_up` integration test green.
- [ ] Fork `v1.5.0` published with detection-matrix completion + workspace bundle pool + Gradle Tooling API + tool-surface consolidation + structural refactoring.
- [ ] Manager `v0.11.0` tagged with version bumps + Help.md / README updates for the new tool surface (Phase D) and refactoring tools (Phase E).
- [ ] No regression on Sprint 9 / Sprint 10 fixtures.

## Out of scope (deferred)

- **External PDE target-platform expansion.** Bundling `org.eclipse.pde.core` + friends (~30 plugins, ~5–10 MB install size) and using `TargetPlatformService` to expand `Require-Bundle` against external `.target` files or an installed Eclipse install. Bigger effort. Workspace bundle pool covers the inter-workspace case which is the dominant case for the user's projects (the RCP product's bundles all live together). Defer until external deps actually become a blocker.
- **Bazel improvements.** Today's Bazel handler walks for BUILD files + jars in `bazel-{bin,out}`. Not a priority for current user projects.
- **Multi-language indexing (Kotlin, Scala).** JDT does some Kotlin via plugins, but full Kotlin/Scala source indexing is out of scope.

## Outlook — strategy, IDE-grade roadmap, deferred items

### Tool budget — staying under 100 (decided 2026-04-26)

Antigravity caps at ~100 tool registrations across all MCP services. The plan to stay under it without giving up capability:

| Server | Tools | Decision |
|---|---|---|
| javalens (1 workspace, post-Sprint 10 Phase B/C) | 66 today → 55 after Sprint 11 Phase D → 60 after Sprint 11 Phase E | core |
| GitKraken MCP shim | 25 | **drop** — `git`/`gh` via Bash already covers everything an autonomous agent needs; the desktop GUI stays useful for humans (visual rebase, conflict UI), it's only the MCP shim that's redundant |
| Headroom for Ring 2+ IDE work (Sprint 12+) | ~40 slots | future |

Math after Sprint 11: 60 (javalens, post-Phase E) + 0 (GitKraken removed) = 60, leaving 40 slots free. Ring 2 (~8 tools) lands at 68. Ring 4 (debugging, 15+ tools) is the only thing that could blow the budget — and per the analysis below, debugging is the lowest-leverage addition for autonomous safe-upgrade work, so deferring it is fine.

### IDE-grade roadmap (Sprint 12+, preview only)

Captured 2026-04-26 from the strategic discussion on what extends javalens to "full-IDE-grade autonomous Java development" — driven by the RCP overhaul use case (safe upgrades, bundle reshuffles, JDK migration). **Ring 1 was originally planned for Sprint 12 but its core refactoring tools (move/pull/push/encapsulate) were pulled forward into Sprint 11 Phase E.** Remaining Ring 1 + Rings 2-4 below.

**Ring 1 — workspace verification (deferred to Sprint 12; ~2 tools):**
- `compile_workspace` — one tool returning every diagnostic across all loaded projects. The missing feedback loop that lets agents verify "did this refactor break anything?" in one call.
- `run_tests(class | method | package)` — JUnit launching + capture. Closes the refactor → test → fix loop.

After Sprint 11 Phase E + Sprint 12 Ring 1 verification, large-scale RCP upgrades become a "find what to change → refactor → compile → test" loop the agent can run end-to-end.

**Ring 2 — rounds out the IDE-grade autonomous loop** (~one sprint, 6–8 tools):
- `generate_constructor`, `generate_equals_hashcode`, `generate_to_string`, `generate_getters_setters`, `override_methods` — JDT has all the AST APIs.
- `convert_lambda_to_method_reference` (companion to existing `convert_anonymous_to_lambda`).
- `migrate_jdk_api(target=var | switch_expr | sealed_types | records | text_blocks)` — bulk JDK-version migration tool. Critical for the JDK 17 → 21 upgrade use case.
- "Fix all of kind X across project" — bulk quick-fix application.

**Ring 3 — Eclipse plugin packaging** (~one sprint, no new tools):
The same `IJdtService` + `LoadedProject` + tool implementations, repackaged as an Eclipse plugin so the work runs in-IDE (view + command palette + in-IDE AI assist target). The portability constraint we've held since Sprint 9 (ADR 0004 — helpers stay `Path` + DOM, no JDT leakage in helper signatures) makes this cheap. **Not a separate product — same logic, different shell.**

**Ring 4 — debugging surface (deferred)**:
JDT debug uses JPDA/JDI (`org.eclipse.jdt.debug.core`). Full surface: breakpoints (line/method/exception/conditional), attach-to-port, launch, step, evaluate expression, watchpoints, hot-swap, threading. The hard part isn't the JDT API — it's that debug is event-driven and MCP is request/response. Agent sessions need long-polling or a streaming mode. **Deferred** because: (a) agents working from passing tests + good static analysis rarely need to debug, (b) it's mostly a human-in-the-loop feature, (c) it's the largest engineering investment of the four rings, (d) a "debug session" parametric tool with sub-commands would be smarter than 15+ separate top-level tools (similar to Phase E's consolidation philosophy).

### Independence posture — confirmed 2026-04-26

The fork (`hw1964/javalens-mcp`) is the user's own version. Upstream PR back to `pzalutski-pixel/javalens-mcp` is **deferred indefinitely**, not abandoned:

- Sprint 9 (source resolution) and Sprint 10 Phase A (multi-project workspace) work IS general — would benefit any javalens user — and is PR-ready (ADR 0004 portability constraint paid the prep cost up front). Diff stays clean for as long as we want to wait.
- Future work (Rings 1-4, especially Ring 3's Eclipse plugin packaging) is increasingly driven by the RCP use case. PR-able pieces will be a subset; review-cycle tax on the full set isn't justified.
- Pzalutski's roadmap is presumably solved for the single-developer / handful-of-projects case (probably a bash script). The 28-project scale that motivated this fork is a different problem and unlikely to be a priority for him.
- **Networked-service direction.** A future "javalens as a hosted service" (HTTPS + auth + multi-user) is captured separately in [`sprint-future-networked-service.md`](sprint-future-networked-service.md). That direction fundamentally diverges from upstream's local-stdio product identity and is the strongest argument for keeping the fork — the auth/TLS/multi-user surface is too opinionated and too operationally-coupled to PR back. Not scheduled, but marked as a real long-term direction.
- Re-evaluate the PR question only if (a) upstream goes inactive, (b) we want their user base for free testing, or (c) we've stabilized to the point where convergence beats independence. None of those are true today.

The portability constraint stays in force regardless — it's also what makes Ring 3 (Eclipse plugin) cheap.

### Eclipse IDE plugin

(Reframed 2026-04-26 — was previously listed as "separate product, not javalens"; now the plan is to package it as Ring 3 of the javalens roadmap, reusing all of the existing `IJdtService` infrastructure.) See "Ring 3 — Eclipse plugin packaging" above.

### LSP-based standalone server

A future product that keeps the headless model but speaks LSP (Language Server Protocol) instead of MCP. Useful for IDE integrations that prefer LSP over MCP. Reuses the same JDT integration layer and portable helpers from javalens.

This is a separate project. Not on the javalens roadmap.

### Bundling javalens jars inside the manager installer

Currently the manager downloads the runtime separately (per `release_repo`). Bundling them into one AppImage simplifies the install for new users at the cost of binding manager release cadence to fork release cadence. Consider after Sprint 11 ships.

## Team split (preview)

- `java-engineer`: Phases A, B, C — Tycho detection, bundle pool, Gradle Tooling API.
- `java-engineer` (or split): Phase D — `find_pattern_usages`, `find_quality_issue` parametric tools + 13-tool deletion.
- `java-engineer`: Phase E — `AbstractRefactoringTool` base + 5 LTK-backed refactoring tools (`move_class`, `move_package`, `pull_up`, `push_down`, `encapsulate_field`).
- `tauri-engineer`: Phase F — defaults flip, manager release.
- `release-engineer`: fork `v1.5.0` + manager `v0.11.0` cuts.
- `qa-test-engineer`: full detection-matrix verification across the four layout types + refactoring conflict/safety scenarios.
- `docs-engineer`: README + Help.md updates for v0.11.0, including new refactoring tools and consolidated `find_*` surface.
