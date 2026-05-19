# Sprint 13 Backlog (draft)

> **Status: draft, written 2026-04-29 after Sprint 12 shipped (fork v1.6.0 + manager v0.12.0).** Sprint 12 left one item unfinished — the new tray-menu screenshot for `public/help/tray-menu.png` and the help.md embed pointing at it. That work carries into Sprint 13 Phase A (still under manager v0.13.0).

> **Design pivot — 2026-04-30.** Fork v1.7.0 shipped on schedule (`fc25acf`, tag `v1.7.0`, 73 tools). Manager v0.13.0 was about to ship the Sprint 12 GTK tray menu with the deferred screenshots, but empirical testing on the user's GNOME setup confirmed the per-menu-item `IconMenuItem` icons are stripped at the AppIndicator D-Bus boundary — the colored circles never reach the user. Two fallbacks were tested and rejected (oversized emoji glyphs; monochrome small bullets — each loses either size or colour). **Decision:** pivot the v0.13.0 deliverable to a Tauri webview popover that replaces the GTK system menu entirely. Full HTML/CSS/Svelte control gives small + coloured + right-aligned + click-without-dismiss + animated transitions in one path. ~5 days work; pushes v0.13.0 ship by ~1 week. Detailed executable plan: [`~/.claude/plans/make-a-plan-happy-fern.md`](../../../.claude/plans/make-a-plan-happy-fern.md). This backlog stays as the historical Sprint 13 narrative; the popover plan is what actually ships under the v0.13.0 tag. Sections below that referenced the GTK menu (Phase A tray screenshots, Phase E cutover) are annotated with their pivot status.

> **Final design — 2026-04-30, end of day.** v0.13.0 ships a **native dynamic menu, NOT the popover**. Empirical testing on the user's GNOME revealed three constraints that together kill the popover approach: (1) libayatana-appindicator hard-requires every indicator to have a menu — a menu-less indicator never registers and is invisible; (2) left-click on the indicator routes directly to the menu — `on_tray_icon_event` doesn't fire on this setup, `show_menu_on_left_click(false)` is silently ignored; (3) `IconMenuItem` images are stripped at the D-Bus boundary. The "menu + popover via menu item" detour was rejected as worse UX than a polished native menu.
>
> **Final v0.13.0 menu shape:**
>
> ```
> Open dashboard            (raises the full manager window)
> ─────
> Workspaces                (disabled header)
>   ●  Javalens_WS          (toggle on click — bullet reflects phase)
>   ○  ORB_JATS_WS
> ─────
> Start all services
> Stop all services
> ─────
> Quit
> ```
>
> Bullets: `●` running, `◐` starting, `○` stopped, `✗` failed — monochrome unicode shapes that render in the menu's own font (1× line height), unlike emoji glyphs which dominate at fixed pixel size.
>
> Plus a real bug fix: `workspace_status_summary` now reads `workspace_name` from the live `config_store.list_projects()` mutex (always fresh after rename) instead of from the runtime-manager's cached status snapshots — closes a stale-name-after-rename loop the user empirically hit. Without this fix, renaming a workspace in the dashboard left the tray menu showing the old name until the workspace was started.
>
> Refresh strategy: 1-second poll with cache-keyed change detection. The menu only rebuilds + swaps when the `(workspace_name, phase)` snapshot actually changed, so steady state is silent (no flicker). Renames / starts / stops / external process deaths propagate within ~1 s.
>
> All popover code/files removed (no `tray.html`, no `TrayPopover.svelte`, no `WebviewWindowBuilder` imports, no popover-only Tauri commands). The architectural lesson — AppIndicator on GNOME constraints make rich tray UI fundamentally impossible without bypassing the system tray entirely — is documented inline in `lib.rs`.

## Goal

Close the **"fully autonomous Java agent"** gap. After Sprint 12, an agent can navigate, search, analyze, refactor, run quick fixes, compile, and run tests through MCP. What's still missing — every gap that costs an agent iterations because it has to hand-edit Java/XML when JDT could do it correctly first time:

1. **Ring 2 — code generation (6 tools).** JDT generates constructors, getters/setters, equals/hashCode, toString, override stubs, and test skeletons correctly the first time (modifiers, generics, throws clauses, varargs, annotations, formatter style). Agents doing this through `Edit` make small mistakes and burn iterations.
2. **Ring 3 — build / dependency management (3 tools).** Agents currently hand-edit `pom.xml` / `build.gradle`, which is error-prone (XML structure, version conflicts, M2E reimport timing).
3. **Ring 4 — formatter / workflow polish (2 tools).** Project-aware formatting and workspace-wide import optimization without an N+1 walk.

Plus the carry-over: **manager tray-menu screenshot + help.md embed** (B.6 from Sprint 12, deferred so v0.12.0 could ship safely without it).

End-of-sprint outcome:

- Per-workspace tool count: **62 → 73** (`+11` across rings 2/3/4).
- Agent's working loop is self-contained inside MCP for the structural-Java work that JDT already knows how to do correctly. Hand-editing `pom.xml` for dep bumps, `Edit`-ing in a constructor, or fanning out `organize_imports` per-file all collapse into one tool call each.
- Fork tagged `v1.7.0`; manager tagged `v0.13.0`.

Predecessor: [`sprint-12-backlog.md`](sprint-12-backlog.md). IDE-grade roadmap reference: [`sprint-11-backlog.md`](sprint-11-backlog.md), "IDE-grade roadmap (Sprint 12+, preview only)" section — Rings 2/3/4 here are the items that section called out.

## Repos touched

- **`javalens-mcp` (fork)** — Phases B/C/D: 11 new tools, target-platform additions for M2E (`org.eclipse.m2e.core`) and Buildship (`org.eclipse.buildship.core`) where dependency tools need them, JDT formatter primitives (already on the target). Cut release `v1.7.0`.
- **`javalens-manager`** — Phase A only: capture the new tray-menu screenshot, update help.md embed, optional re-capture of dashboard/settings if anything visibly changed since v0.12.0. Cut release `v0.13.0`.

## Out of scope (settled)

- HTTP / networked-service direction — still tracked in [`sprint-future-networked-service.md`](sprint-future-networked-service.md).
- Eclipse plugin packaging (Ring 5) and JDK-API migration tools — Sprint 14+.
- Manager-side multi-window / per-workspace settings UI.
- IDE features that aren't structural Java (line-by-line code completion, inline-suggestion preview, debugger).
- Wholesale `pom.xml` / `build.gradle` *creation* — Sprint 13 only edits existing build files. New-project scaffolding is out of scope.

## Authorship / attribution rule

Same as Sprint 11 / 12: zero AI-attribution boilerplate in commits, release notes, or docs produced during execution. See `feedback_no_coauthored_trailer.md`.

## Workflow rule (NEW for Sprint 13) — focused unit tests during, full verify only at the end

Sprint 12 lost multiple hours to 20-minute full reactor builds for single-test debugging. New rule for Sprint 13:

- **During development of any tool**, run only that tool's unit tests via focused execution:
  ```bash
  mvn -pl org.javalens.mcp.tests -am verify -Dtest=ToolNameTest
  ```
  Typical: ~2 min instead of ~20 min.
- **Compile-only loop** when iterating on a tool's source (no test changes):
  ```bash
  mvn -pl org.javalens.mcp -am compile
  ```
  Typical: ~30 s.
- **Manager Rust** uses `cargo test --lib` for unit tests (Sprint 12 baseline; ~10 s).
- **Full reactor `mvn clean verify`** runs **once** at end of sprint as the smoke + regression gate before tagging. Goal: no surprise interaction between rings.
- **No** `mvn clean verify` "just to check" between phases. If a tool's focused test passes, move on; integration regressions are caught by the end-of-sprint full run.

Rationale: Tycho's full reactor builds are I/O-heavy (target-platform resolution, OSGi resolver, every module's package phase). For "did my change break this one test", the focused command path skips those phases entirely. The end-of-sprint full run is the only one that needs to walk the entire reactor.

## Order of work

UI / docs strictly come last. The help-file rewrite and manager v0.13.0 release describe what fork v1.7.0 actually delivers, so they only get written *after* Phases B/C/D are green and we know the real shape of the 11 new tools. Tray screenshots are the one exception — they only need a working v0.12.0 manager dev build and can be captured at any time during the sprint, even on day 1.

1. **Phase A — Tray screenshots (USER STEP)** (~30 min) — captures `public/help/tray-menu.png` + `public/help/tray-icon.png`. Carry-over from Sprint 12. Captured anytime; held aside for the Phase E commit.
2. **Phase B — Ring 2 code generation (fork)** (~5 days, 6 tools)
3. **Phase C — Ring 3 build / dependency management (fork)** (~3 days, 3 tools)
4. **Phase D — Ring 4 formatter / workflow polish (fork)** (~2 days, 2 tools)
5. **Phase E — Cutover (fork v1.7.0 + UI docs + manager v0.13.0)** (~1 day) — single full `mvn clean verify`, fork tag + push, **then** help.md rewrite using the now-real tool inventory, **then** manager release.

Total ~2 weeks of focused work.

UI rule, restated: never write user-facing docs ahead of the code that backs them. Phase B/C/D ship 11 tools; whatever they actually deliver is what Phase E describes. Drafting docs before that risks promising things the tools don't quite do.

## Phase A — Tray screenshots (USER STEP) — **OBSOLETE per 2026-04-30 design pivot**

> The original Phase A captured the v0.12.0 GTK tray menu as a deferred Sprint 12 carry-over. With the GTK menu being removed entirely in v0.13.0 and replaced by a Tauri webview popover, **nothing of the v0.12.0 menu surface survives to be screenshotted**. The deferred screenshot work is replaced by the popover plan's Phase D.1 (capture popover + tray-icon screenshots once the popover is built). The `tray-icon.png` capture itself stays — only the bar/panel icon, unchanged by the pivot — and lives in the popover plan.
>
> Original Phase A text retained below for sprint-history reference; do not execute.

**Why standalone, not bundled with the manager release:** v0.12.0 shipped to GitHub on 2026-04-29 without `public/help/tray-menu.png` (the screenshot is a USER STEP — requires `npx tauri dev` running with two workspaces in different states, can't be automated headlessly). Sprint 13 captures it once and stages the PNGs in `public/help/`. They get bundled into the manager v0.13.0 commit at Phase E.4.

### A.1 Capture tray screenshots

Two screenshots, both captured against `npx tauri dev` with at least 2 workspaces configured (one Running → green icon, one Stopped → gray icon — so the status-icon variations are visible).

1. **`public/help/tray-menu.png`** — tray menu *open*, showing the full menu shape (Show / per-workspace toggles with status icons / Start all / Stop all / Quit). This is the primary screenshot.
2. **`public/help/tray-icon.png`** — tray *icon only* in the system tray bar, showing the JavaLens icon as it appears at rest (small, ~24×24 in the GNOME panel). Helps users locate the tray entry on first install.

Capture flow:

```bash
cd /home/harald/CursorProjects/javalens-manager
npx tauri dev
# in another terminal:
gnome-screenshot --delay=5 -f ~/Desktop/tray-menu.png   # open menu before delay fires
gnome-screenshot --delay=5 -f ~/Desktop/tray-icon.png   # let menu close before this one
```

Crop both with GIMP / `convert -crop` / Preview. Move into `public/help/`.

Re-capture `public/help/dashboard.png` / `settings-top.png` / `settings-bottom.png` **only if** they visibly changed during Sprint 12 / 13. Sprint 12 didn't touch dashboard or settings UI; Sprint 13 won't either (it's all fork-side tool work). Likely skip.

**Do not edit `help.md` yet.** The tray-section embed and the "Tool surface" rewrite both happen at Phase E.3, so they go in one cohesive commit — and the tool-surface rewrite needs the real Ring 2/3/4 inventory from Phase B/C/D, which doesn't exist until then.

## Phase B — Ring 2 code generation (fork)

JDT's source-code generation primitives live in `org.eclipse.jdt.core.dom.rewrite.ASTRewrite` (public API) and `org.eclipse.jdt.internal.corext.codemanipulation.*` (internal but ABI-stable for ~15 years; same pattern Sprint 11 LTK refactorings already use). Each tool wraps one or two operations and surfaces a structured result.

For each tool: extends `AbstractTool`, registered in `JavaLensApplication.registerTools()`, **2 unit tests** in `org.javalens.mcp.tests/src/org/javalens/mcp/tools/codegen/<Tool>Test.java` — happy path + one validation/conflict path. Testing strategy mirrors Sprint 11 LTK refactorings: load the `simple-maven` fixture via `helper.loadProjectCopy(...)`, perform the generation, assert the resulting source matches an expected snapshot.

### B.1 `generate_constructor`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/codegen/GenerateConstructorTool.java`.

**Implementation:** wraps `org.eclipse.jdt.internal.corext.codemanipulation.GenerateConstructorOperation` (or rebuild equivalent via `ASTRewrite` if internal API drift bites). Resolves the target type from `filePath`/`line`/`column`, takes a `fields[]` array of field names, generates a constructor that initializes them with proper `super(...)` chaining if the supertype requires it.

**Input shape:**
```json
{
  "projectKey": "optional",
  "filePath": "...",
  "line": 42,
  "column": 4,
  "fields": ["name", "id"],
  "visibility": "public | protected | private | package",
  "callSuper": "auto | true | false"
}
```

**Result shape:** `{operation, filePath, edits: [...], generatedSource: "...", warnings: []}`.

**Tests (2):**
- `happy_constructorForFields_generatesInitializers` — empty class with two fields → 1 constructor body initializing both.
- `validation_unknownField_returnsInvalidParameter` — field name not on class → `INVALID_PARAMETER`.

### B.2 `generate_getters_setters`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/codegen/GenerateGettersSettersTool.java`.

**Implementation:** wraps `GetterSetterUtil.getGetterStub` / `getSetterStub` per field; supports multi-field selection (this is the gap vs. existing `encapsulate_field`, which is single-field and changes call sites).

**Input shape:**
```json
{
  "projectKey": "optional",
  "filePath": "...",
  "line": 42,
  "column": 4,
  "fields": ["name", "id"],
  "kind": "getters | setters | both",
  "visibility": "public | protected | private | package"
}
```

**Result shape:** `{operation, filePath, edits, generatedSource, methodsAdded: ["getName", "setName", ...]}`.

**Tests (2):**
- `happy_bothForTwoFields_generatesFourMethods` — class with `private String name; private int id` → 4 methods.
- `conflict_existingGetter_isSkipped` — class already has `getName()` → tool skips it (don't double-define), reports in `warnings[]`.

### B.3 `generate_equals_hashcode`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/codegen/GenerateEqualsHashCodeTool.java`.

**Implementation:** wraps `GenerateHashCodeEqualsOperation`. Handles null-safety, generics, hash mixing per JDT's existing template (matches "Source > Generate hashCode() and equals()" menu in Eclipse IDE).

**Input shape:**
```json
{
  "projectKey": "optional",
  "filePath": "...",
  "line": 42,
  "column": 4,
  "fields": ["id", "name"],
  "useInstanceof": true,
  "useObjectsHash": true
}
```

**Tests (2):**
- `happy_equalsHashCodeForFields_generatesPair` — class with two fields → matching `equals` + `hashCode` using both.
- `validation_emptyFields_returnsInvalidParameter` — no fields specified → reject (Eclipse IDE allows it but generates trivial impl; our tool requires explicit fields to avoid surprise).

### B.4 `generate_tostring`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/codegen/GenerateToStringTool.java`.

**Implementation:** wraps `GenerateToStringOperation`. Supports the four standard JDT formatter styles: `STRING_CONCATENATION`, `STRING_BUILDER`, `STRING_FORMAT`, `STRING_BUILDER_CHAINED`.

**Input shape:**
```json
{
  "projectKey": "optional",
  "filePath": "...",
  "line": 42,
  "column": 4,
  "fields": ["id", "name"],
  "style": "STRING_CONCATENATION | STRING_BUILDER | STRING_FORMAT | STRING_BUILDER_CHAINED",
  "skipNulls": false
}
```

**Tests (2):**
- `happy_toStringConcatStyle_generatesMethod` — style=concat → expected output shape.
- `happy_toStringBuilderStyle_generatesMethod` — style=builder → `StringBuilder` chain.

### B.5 `override_methods`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/codegen/OverrideMethodsTool.java`.

**Implementation:** wraps `OverrideMethodsOperation`. Lists abstract methods from supertypes / interfaces; user (agent) selects which to override; tool emits stubs with proper `@Override` annotation and correct generic erasure.

**Input shape:**
```json
{
  "projectKey": "optional",
  "filePath": "...",
  "line": 42,
  "column": 4,
  "methods": ["compareTo(T)", "toString()"],
  "addOverrideAnnotation": true,
  "addUnimplementedThrow": true
}
```

`methods[]` accepts JDT method signatures; if omitted, tool returns the list of overridable methods in `availableMethods[]` so the agent can pick.

**Tests (2):**
- `happy_overrideAbstractMethod_generatesStub` — class extends abstract base → stub with `@Override` + `throw new UnsupportedOperationException("not yet implemented")`.
- `query_listAvailable_returnsCandidates` — call without `methods[]` → `availableMethods[]` populated, no source change.

### B.6 `generate_test_skeleton`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/codegen/GenerateTestSkeletonTool.java`.

**Implementation:** create a JUnit test class adjacent to (or in the standard `src/test/java` mirror of) the target source file. Walk the target's public methods; emit a `@Test` stub method per public method (`<methodName>_<descriptor>` naming), plus `@BeforeEach setUp()`. Detect framework via existing `run_tests` auto-detection logic (junit5 default; junit4 if classpath has only `org.junit.Test`; testng if only TestNG). No actual logic — just stubs with `// TODO: implement` placeholders, so the agent fills them.

**Input shape:**
```json
{
  "projectKey": "optional",
  "filePath": "...",
  "line": 42,
  "column": 4,
  "framework": "junit5 | junit4 | testng | auto",
  "includePrivateMethods": false
}
```

**Tests (2):**
- `happy_testSkeletonForClass_generatesTestPerPublicMethod` — class with 3 public methods → 3 `@Test` methods + `@BeforeEach`.
- `frameworkAutoDetect_picksJUnit5` — fixture has junit-jupiter → resolves to junit5 without explicit `framework` arg.

### B.7 Tool registration

[`org.javalens.mcp/src/org/javalens/mcp/JavaLensApplication.java`](../../CursorProjects/javalens-mcp/org.javalens.mcp/src/org/javalens/mcp/JavaLensApplication.java) — add 6 lines in `registerTools()` near the Sprint 12 verification-tools registration:

```java
toolRegistry.register(new GenerateConstructorTool(() -> jdtService));
toolRegistry.register(new GenerateGettersSettersTool(() -> jdtService));
toolRegistry.register(new GenerateEqualsHashCodeTool(() -> jdtService));
toolRegistry.register(new GenerateToStringTool(() -> jdtService));
toolRegistry.register(new OverrideMethodsTool(() -> jdtService));
toolRegistry.register(new GenerateTestSkeletonTool(() -> jdtService));
```

## Phase C — Ring 3 build / dependency management (fork)

### C.1 Target-platform additions

The dependency tools need M2E for Maven projects and Buildship for Gradle. Add to [`org.javalens.target/org.javalens.target.target`](../../CursorProjects/javalens-mcp/org.javalens.target/org.javalens.target.target) (bump `sequenceNumber` to 5):

- `org.eclipse.m2e.core` (M2E core API: classpath model, project lifecycle).
- `org.eclipse.m2e.maven.runtime` (resolver + indexer; needed for `find_unused_dependencies`).
- `org.eclipse.buildship.core` (Buildship Gradle integration).

Add to [`org.javalens.mcp/META-INF/MANIFEST.MF`](../../CursorProjects/javalens-mcp/org.javalens.mcp/META-INF/MANIFEST.MF) `Require-Bundle:`:

- `org.eclipse.m2e.core;resolution:=optional`
- `org.eclipse.buildship.core;resolution:=optional`

`resolution:=optional` is critical: not every project is Maven or Gradle (e.g., the Tycho-packaged fork modules aren't M2E projects). Tools degrade gracefully when the bundle isn't usable for a given project.

### C.2 `add_dependency`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/build/AddDependencyTool.java`.

**Implementation:**
- Detect build tool by inspecting project root: `pom.xml` → Maven path; `build.gradle` / `build.gradle.kts` → Gradle path.
- Maven path: parse `pom.xml` via `org.eclipse.m2e.core.embedder.IMaven`'s model API, insert `<dependency>` under `<dependencies>` (preserving existing formatting), trigger `IMavenProjectFacade.markerLocator()` reimport.
- Gradle path: append a line to the `dependencies { ... }` block in `build.gradle`. (Buildship's project model is read-only; we edit the file textually then trigger a Gradle refresh via `org.eclipse.buildship.core.workspace.GradleBuild.synchronize`.)
- Result includes the resolved classpath delta (new entries that appeared after re-resolve).

**Input shape:**
```json
{
  "projectKey": "optional",
  "groupId": "org.junit.jupiter",
  "artifactId": "junit-jupiter-api",
  "version": "5.10.0",
  "scope": "compile | test | provided | runtime"
}
```

**Result shape:** `{operation, filePath, classpathAdded: [...], warnings: []}`.

**Tests (2):**
- `happy_addCompileDependency_appearsInPom` — Maven fixture → `pom.xml` updated, classpath delta non-empty.
- `validation_unknownProjectKind_returnsUnsupported` — Tycho fixture (no pom/build.gradle) → `INVALID_PARAMETER` with explanatory message.

### C.3 `update_dependency`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/build/UpdateDependencyTool.java`.

**Implementation:** find existing `<dependency>` (or Gradle equivalent) by `groupId` + `artifactId`, replace `<version>`. Same M2E/Buildship reimport flow as `add_dependency`.

**Input shape:**
```json
{
  "projectKey": "optional",
  "groupId": "org.junit.jupiter",
  "artifactId": "junit-jupiter-api",
  "newVersion": "5.11.0"
}
```

**Tests (2):**
- `happy_updateMavenDependencyVersion_pomReflects` — Maven fixture with existing junit dep → version bumped.
- `validation_unknownDependency_returnsNotFound` — `groupId:artifactId` not in pom → `INVALID_PARAMETER`.

### C.4 `find_unused_dependencies`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/build/FindUnusedDependenciesTool.java`.

**Implementation:** for each declared dependency in `pom.xml` / `build.gradle`, walk the project's `IPackageFragment`s, collect every `import` and qualified type reference, then for each dep determine which classes it provides (via M2E's resolved artifact JAR's package list). A dep with zero referenced classes is unused.

This is a "find" tool — it reports, doesn't modify. Removing an unused dep is `update_dependency` territory (`remove_dependency` if that becomes a thing in Sprint 14).

**Input shape:**
```json
{
  "projectKey": "optional"
}
```

**Result shape:** `{operation, projectKind: "maven|gradle", unusedDependencies: [{groupId, artifactId, version, scope, declaredAt}]}`.

**Tests (2):**
- `happy_unusedDepReported_passingDepNotReported` — fixture with one used + one unused dep → exactly one entry in result.
- `unsupportedProject_returnsEmptyWithWarning` — Tycho project → empty `unusedDependencies`, warning explaining the tool is for Maven/Gradle projects.

### C.5 Tool registration

Add 3 lines in `registerTools()`:
```java
toolRegistry.register(new AddDependencyTool(() -> jdtService));
toolRegistry.register(new UpdateDependencyTool(() -> jdtService));
toolRegistry.register(new FindUnusedDependenciesTool(() -> jdtService));
```

## Phase D — Ring 4 formatter / workflow polish (fork)

These are JDT primitives that already exist on the target platform; no new bundles needed.

### D.1 `format_file` (and `format_workspace` via `scope`)

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/workflow/FormatTool.java`.

**Implementation:** `org.eclipse.jdt.core.formatter.CodeFormatter` with the project's own formatter settings (read from `.settings/org.eclipse.jdt.core.prefs`). Falls back to JDT default if no project-level settings.

**Input shape:**
```json
{
  "projectKey": "optional",
  "scope": { "kind": "file | package | project | workspace", "filePath": "...", "packageName": "..." },
  "dryRun": false
}
```

`scope.kind` controls fan-out: file → one file; package → all `.java` under a package fragment; project → entire project; workspace → every loaded project. `dryRun: true` returns the diff without writing.

**Result shape:** `{operation, filesFormatted, byteDelta, diffs?: [...]}`.

**Tests (2):**
- `happy_formatFile_normalizesWhitespace` — fixture file with spurious whitespace → matches JDT default.
- `dryRun_returnsDiffWithoutWriting` — same file with `dryRun: true` → diff returned, file unchanged on disk.

### D.2 `optimize_imports_workspace`

**File (new):** `org.javalens.mcp/src/org/javalens/mcp/tools/workflow/OptimizeImportsWorkspaceTool.java`.

**Implementation:** existing per-file `organize_imports` (Sprint 11) wrapped in a workspace fan-out. Walks every loaded project, every `ICompilationUnit`, runs the existing organize-imports logic, aggregates the results.

**Input shape:**
```json
{
  "projectKey": "optional",
  "scope": "project | workspace"
}
```

**Result shape:** `{operation, filesProcessed, importsRemoved, importsReorganized}`.

**Tests (2):**
- `happy_workspaceScope_visitsAllProjects` — multi-project fixture → both projects' files processed, total counts match expectations.
- `happy_projectScope_visitsOneProject` — single-project scope → only one project visited.

### D.3 Tool registration

Add 2 lines in `registerTools()`:
```java
toolRegistry.register(new FormatTool(() -> jdtService));
toolRegistry.register(new OptimizeImportsWorkspaceTool(() -> jdtService));
```

## Phase E — Cutover release

> **Status (2026-04-30):** E.1 and E.2 **shipped** (full reactor verify ran clean — 122/122 core + 446/446 mcp tests, 5 `@Disabled`; fork v1.7.0 tagged at commit `fc25acf` and pushed). E.3 and E.4 are **superseded by the popover plan**: the manager's help.md edits + v0.13.0 cutover are sequenced under the popover plan's Phase D.2 / D.3 (Sprint 13 D.4 / D.5 carry-overs). E.3's tray-section embed in particular is fully rewritten there because the popover replaces the GTK menu it originally documented.
>
> Original E.1–E.4 retained below for sprint-history reference.

### E.1 Final full reactor verify (THE ONLY full run of the sprint)

```bash
cd /home/harald/CursorProjects/javalens-mcp
mvn clean verify
```

Expected (assuming Phases B/C/D land green via focused tests):

- `org.javalens.core.tests`: **122 / 122** (unchanged).
- `org.javalens.mcp.tests`: **~446 / 446** (424 from Sprint 12 + 22 new = 11 tools × 2 tests; 4 `@Disabled` carry over from Sprint 12).

If the full run reveals interactions between rings (e.g., `format_file` affecting `OverrideMethodsTool`'s expected snapshot), fix the interaction, re-run focused tests for the affected tool(s), then full verify once more.

### E.2 Tag fork v1.7.0

- Bump `Bundle-Version` qualifier across 4 OSGi bundles + product + 8 reactor pom.xml files: `1.6.0-SNAPSHOT` → `1.7.0-SNAPSHOT`.
- Wipe stale `~/.m2/repository/org/javalens/.../1.6.0-SNAPSHOT/` to avoid the Sprint 12 stale-bundle issue.
- New [`docs/release-notes/v1.7.0.md`](release-notes/v1.7.0.md) — Rings 2/3/4, all 11 tools, target-platform additions.
- [`README.md`](../../CursorProjects/javalens-mcp/README.md) — bump tool count `62 → 73`; add a "Code generation" subsection (Ring 2), expand "Build & dependency management" subsection (Ring 3), expand "Workflow polish" subsection (Ring 4).
- [`docs/upgrade-checklist.md`](../../CursorProjects/javalens-mcp/docs/upgrade-checklist.md) — note the M2E / Buildship target-platform additions.
- `git tag -a v1.7.0 -F docs/release-notes/v1.7.0.md && git push origin master v1.7.0`.
- CI release workflow auto-publishes the GitHub Release.

### E.3 Manager help.md rewrite (UI docs — fork v1.7.0 must be tagged first)

Now that fork v1.7.0 is tagged and the 11 tools are real, write the user-facing docs against what was actually shipped — not against what we drafted at sprint start.

[`src/assets/help.md`](../src/assets/help.md), two coordinated edits in one pass:

**E.3.a — Tray section embed:**

Extend the existing "System tray" section to embed the screenshots staged by Phase A:

```markdown
![Tray icon at rest](/help/tray-icon.png)

The JavaLens icon sits in the system tray once the manager starts. Click
to open the menu:

![Tray menu with per-workspace status icons](/help/tray-menu.png)
```

**E.3.b — "Tool surface" rewrite for fork v1.7.x:**

The existing `### Tool surface (fork v1.5.x)` heading needs:

- Heading bump to `### Tool surface (fork v1.7.x)`.
- Tool count statement: 73 tools per workspace (up from v1.5.x baseline).
- Three new subsections, written from the **shipped** Phase B/C/D tools — describe what an agent can actually do for the user, not API spec. Tone matches the existing Sprint 11 LTK refactoring paragraph above. If any tool got `@Disabled` happy-path tests due to the cut line, soften that subsection's language to "the agent can ask JavaLens to ..." rather than "JavaLens does ..." so we don't promise a smoothness the v1.7.0 release doesn't yet deliver.
  - **Code generation** (Ring 2, ~6 tools) — calibrate wording against what shipped.
  - **Build & dependency management** (Ring 3, ~3 tools) — calibrate against shipped.
  - **Workflow polish** (Ring 4, ~2 tools) — calibrate against shipped.
- Cross-link the fork's `README.md` "Code generation" / "Build & dependency management" / "Workflow polish" subsections that v1.7.0 added.

Verify rendering with `npx tauri dev`'s help-page reload — both new images render, the new subsections sit correctly above the existing "Selected Project Status" subsection, no broken links.

### E.4 Manager v0.13.0 cutover

- Bump `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` from `0.12.0` → `0.13.0`.
- `cargo check --manifest-path src-tauri/Cargo.toml` to refresh `Cargo.lock`.
- New [`docs/release-notes/v0.13.0.md`](release-notes/v0.13.0.md) — covers (a) the Sprint 12 carry-over (tray screenshots + tray section embed) and (b) the help-file refresh for fork v1.7.0 tools (Rings 2/3/4 picked up automatically by the existing release-poller; no manager-side code change for that).
- **One commit** bundling: Phase A screenshots (`public/help/tray-{menu,icon}.png`), Phase E.3 help.md edits, version bumps, release notes.
- `git tag -a v0.13.0 -F docs/release-notes/v0.13.0.md && git push origin main v0.13.0`.
- Publish draft via `gh api -X PATCH /repos/hw1964/javalens-manager/releases/{id}` with `body` from notes file, `draft: false`, `make_latest: "true"`.

## Critical files

| Repo / Path | Phase | Change |
|---|---|---|
| ~~`javalens-manager/public/help/tray-menu.png`~~ | A.1 | **OBSOLETE per design pivot** — replaced by `tray-popover.png` in popover plan D.1 |
| `javalens-manager/public/help/tray-icon.png` | popover plan D.1 | NEW — tray icon at rest, captured by user (unchanged by pivot) |
| `javalens-manager/src/assets/help.md` | popover plan D.2 | Tray section rewritten for popover model + "Tool surface (fork v1.7.x)" rewrite |
| `javalens-manager/{package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json}` | popover plan D.3 | 0.13.0 |
| `javalens-manager/docs/release-notes/v0.13.0.md` | popover plan D.3 | NEW |
| `javalens-mcp/.../tools/codegen/GenerateConstructorTool.java` | B.1 | NEW |
| `javalens-mcp/.../tools/codegen/GenerateGettersSettersTool.java` | B.2 | NEW |
| `javalens-mcp/.../tools/codegen/GenerateEqualsHashCodeTool.java` | B.3 | NEW |
| `javalens-mcp/.../tools/codegen/GenerateToStringTool.java` | B.4 | NEW |
| `javalens-mcp/.../tools/codegen/OverrideMethodsTool.java` | B.5 | NEW |
| `javalens-mcp/.../tools/codegen/GenerateTestSkeletonTool.java` | B.6 | NEW |
| `javalens-mcp/.../tools/build/AddDependencyTool.java` | C.2 | NEW |
| `javalens-mcp/.../tools/build/UpdateDependencyTool.java` | C.3 | NEW |
| `javalens-mcp/.../tools/build/FindUnusedDependenciesTool.java` | C.4 | NEW |
| `javalens-mcp/.../tools/workflow/FormatTool.java` | D.1 | NEW |
| `javalens-mcp/.../tools/workflow/OptimizeImportsWorkspaceTool.java` | D.2 | NEW |
| `javalens-mcp/org.javalens.target/org.javalens.target.target` | C.1 | Add M2E + Buildship bundles, sequenceNumber → 5 |
| `javalens-mcp/org.javalens.mcp/META-INF/MANIFEST.MF` | C.1 | Require-Bundle: M2E + Buildship (resolution:=optional) |
| `javalens-mcp/.../JavaLensApplication.java` | B.7, C.5, D.3 | Register 11 new tools |
| `javalens-mcp/org.javalens.mcp.tests/.../tools/{codegen,build,workflow}/*` | B/C/D | NEW — 22 tests (11 tools × 2 each) |
| `javalens-mcp/{8 pom.xml + 4 MANIFEST.MF + product}` | E.2 | 1.7.0-SNAPSHOT / 1.7.0.qualifier |
| `javalens-mcp/docs/release-notes/v1.7.0.md` | E.2 | NEW |
| `javalens-mcp/README.md` | E.2 | Tool count + ring subsections |
| `javalens-mcp/docs/upgrade-checklist.md` | E.2 | M2E / Buildship target-platform note |

## Reusable infrastructure already in place

- **`AbstractTool.execute(...)` pattern** (lines 90–124 of `AbstractTool.java`) — all 11 new tools extend this; no new abstractions.
- **`IJdtService.allProjects()` / `getProject(key)`** — Phase D workspace-fan-out tools walk projects via these.
- **`ASTRewrite` + `IPackageFragment` walks** — same primitives Sprint 11 LTK refactorings (`extract_method`, `pull_up`, etc.) use; Phase B tools build on them.
- **`TestProjectHelper.loadProjectCopy(...)` / `loadWorkspaceCopy(String...)`** — every Sprint 13 test reuses these.
- **`simple-maven` fixture** — already has classes, fields, methods suitable for Ring 2 codegen tests; extend with one or two synthetic classes for snapshot expectations rather than create new fixture projects.
- **`organize_imports` (Sprint 11)** — `optimize_imports_workspace` is a fan-out wrapper over the existing per-file logic.
- **`run_tests` framework auto-detection (Sprint 12)** — `generate_test_skeleton` reuses the same classpath-walk logic.
- **Tycho release workflow + manager release-poller** — fork v1.7.0 ships the 11 tools; manager auto-picks them up via the same poller path that picked up Sprint 11 / 12 fork releases.

## Verification (sprint exit)

After Phase E:

1. **Tool count** — `health_check` reports **73 tools** per service. All 11 new tools appear, all 62 prior tools still appear.
2. **Ring 2 smoke** — load `simple-maven`, run `generate_constructor` on a simple class, run `generate_equals_hashcode` on the same, run `generate_tostring` on the same, then `compile_workspace` → no errors.
3. **Ring 3 smoke** — load `simple-maven`, `add_dependency` for `commons-lang3:3.14.0`, then `update_dependency` to `3.15.0`, verify pom reflects, run `compile_workspace` → no errors.
4. **Ring 4 smoke** — `format_workspace` over `simple-maven`, `optimize_imports_workspace` over the same, both no-op-clean on second pass.
5. **Manager tray** — open tray menu in v0.13.0 dev build, confirm new screenshot embeds in help, all v0.12.0 tray behaviors still work.
6. **No regression on prior sprints** — full reactor verify (E.1) shows 122/122 core.tests + 446/446 mcp.tests (4 `@Disabled` carrying through from Sprint 12).

After release:

7. Both releases published, manager v0.13.0 marked Latest, fork v1.7.0 visible to the release-poller.

## Cut line if a Phase B/C/D tool hits unexpected pain

Per the v1.5.2 / v1.6.0 precedent (`encapsulate_field` and `run_tests` happy-path `@Disabled` with explanatory pointers): if any individual tool runs into infrastructural blockers (e.g., M2E's headless reimport doesn't fire reliably in Tycho-test runtime), mark the affected happy-path tests `@Disabled` with a pointer to `docs/upgrade-checklist.md` and ship the tool with validation/conflict coverage only. Tool stays registered for production usage. Goal: 11 tools shipped, even if one or two have a v1.7.1 follow-up for the headless-test path.

## Build / test commands

`javalens-mcp` (during Sprint 13 — focused only):

```bash
cd /home/harald/CursorProjects/javalens-mcp

# Per-tool focused unit test (~2 min):
mvn -pl org.javalens.mcp.tests -am verify -Dtest=GenerateConstructorToolTest

# Compile-only loop (~30 s):
mvn -pl org.javalens.mcp -am compile

# End of sprint, ONCE (~20 min):
mvn clean verify
```

`javalens-manager`:

```bash
cd /home/harald/CursorProjects/javalens-manager
npx svelte-check --tsconfig ./tsconfig.json
cargo check --manifest-path src-tauri/Cargo.toml
cargo test  --manifest-path src-tauri/Cargo.toml --lib
npx tauri dev   # for tray-menu.png screenshot capture
```

## Definition of Done

- [x] ~~Phase A: tray screenshots~~ — **OBSOLETE per design pivot.** v0.13.0 ships the webview popover; the GTK menu is removed. Popover screenshots are captured under the popover plan's Phase D.1 instead.
- [x] Phase B: 6 codegen tools shipped, registered, focused tests green (12/12 + 1 @Disabled). **Done.**
- [x] Phase C: 3 dep-management tools shipped, registered, focused tests green (6/6). **Done.**
- [x] Phase D: 2 workflow tools shipped, registered, focused tests green (4/4). **Done.**
- [x] Phase E.1: full reactor `mvn clean verify` green (122 core + 446 mcp + 5 `@Disabled` carrying through). **Done.**
- [x] Phase E.2: Fork v1.7.0 tagged + published (commit `fc25acf`, tag `v1.7.0`). **Done.**
- [ ] ~~Phase E.3 / E.4~~ — **Superseded by popover plan.** Manager v0.13.0 ships the webview popover; help.md edits + cutover sequence are under the popover plan's Phase D.2 / D.3. See [`~/.claude/plans/make-a-plan-happy-fern.md`](../../../.claude/plans/make-a-plan-happy-fern.md).
- [x] Per-workspace tool count is **73** (`health_check` confirms). **Done** (in fork v1.7.0).
- [ ] Zero AI-attribution boilerplate in any commit, release note, or doc produced during the sprint. (carries through popover plan)
- [ ] No regression on Sprint 11 / 12 fixtures (existing 424 mcp.tests + 122 core.tests + 42 manager Rust tests stay green). (popover plan keeps the 42-Rust-tests requirement.)
