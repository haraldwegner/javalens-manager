# Future Sprint Enhancements

## Deploy UX Enhancements

### Reintroduce True Preview Mode

Current state:
- Dashboard deploy actions include `Deploy`, `Dry run`, `Regenerate`, and `Delete`.
- `Preview` button is intentionally removed until a meaningful preview UX exists.

Future enhancement:
- Add back a dedicated `Preview` action and button.
- Render per-client generated MCP/rule output in a compact, readable panel before write.
- Keep preview run-scoped target selection identical to deploy/dry-run/regenerate/delete.

Acceptance goals:
- `Preview` clearly differs from `Dry run` in user-visible output.
- Preview output is compact and does not break dashboard density.
- Per-client preview supports copy/inspect without writing files.

## Platform Release Rollout

### MacOS and Windows Delivery

Future enhancement:
- Add packaging and release workflow for MacOS and Windows builds.
- Document install/update paths, signing requirements, and release notes policy.

Acceptance goals:
- Repeatable CI/CD build pipeline for Linux, MacOS, and Windows artifacts.
- Versioned release bundles and install instructions per platform.

## Services Selection from Probe

### Per-Service Deploy Selection

Current state:
- Settings probe can discover exposed services from JavaLens runtime.
- Deploy currently targets client configs as a full managed set.

Future enhancement:
- Use discovered services as selectable deploy units (single-service granularity).
- Allow selecting or excluding individual services for deploy generation.

Acceptance goals:
- Service discovery output is persisted and used as deploy input options.
- Users can deploy subset of services per client/target run.
- Validation explains missing/incompatible selections clearly.

---

## Strategic discussion — toward "Claude as the best Java dev"

This section is a **discussion document, not a committed plan**. It maps where JavaLens stands today (Sprint 13 closeout: 73 MCP tools per workspace) against the larger ambition of making Claude the most effective Java developer of all time, and asks where the remaining gaps live: in our tool surface, in upstream IDE features we haven't ported, in territory no IDE covers, and ultimately in what tooling alone *can't* fix.

The goal is to surface the choices, not to pre-commit to any of them. Each subsection ends with a difficulty / sequencing note so we can prioritise in Sprint 14+ planning sessions.

### Where we are today (Sprint 13 baseline)

73 tools per workspace, organised in seven families:
- **Workspace administration** (5) — load / list / add / remove project, health.
- **Navigation** (10) — search symbols, go-to-definition, references, type hierarchy, document symbols, hover, position-anchored type/method/field.
- **Search** (5 + 2 parametric) — method references, field writes, test discovery, plus `find_pattern_usages(kind)` and `find_quality_issue(kind)` covering 13 distinct queries.
- **Analysis** (16) — diagnostics, call hierarchy, signature help, javadoc, change-impact, data-flow, control-flow, DI, file/type/method-level summaries.
- **Refactoring** (15) — local (rename, extract method/constant/variable/interface, inline, change-signature, organize-imports, convert-anonymous-to-lambda) plus structural LTK (move-class, move-package, pull-up, push-down, encapsulate-field).
- **Verification** (2) — `compile_workspace`, `run_tests`.
- **Code generation, dep mgmt, workflow polish** (11, Sprint 13) — generate constructor / getters-setters / equals-hashcode / toString / override-methods / test-skeleton; add/update/find-unused dependency (Maven); format; optimize-imports-workspace.
- **Quick fixes / metrics / project & infrastructure** (3 + 2 + 4).

The agent loop **navigate → analyze → refactor → generate → manage deps → format → compile → test** is closed inside MCP. No IDE-feature gap is fatal at this point; the open question is where additional tooling produces the largest marginal improvement in autonomous-Java-agent quality.

### What's still missing from IntelliJ / Eclipse parity

Things IDEs ship that JavaLens doesn't:

| Feature | Sketch of MCP tool | Difficulty |
|---|---|---|
| **Live templates / postfix completion** (`var.if`, `var.nn`, `psvm`) | `expand_postfix(filePath, line, column, template_id)` — JDT has the `org.eclipse.jdt.internal.corext.template.java` machinery; same JDT-UI internal-API shim risk as Sprint 11 LTK refactorings. | Medium. Useful for human-in-the-loop, less so for fully autonomous agent. |
| **Surround-with** (try/catch, if, lambda, synchronized, while loop) | `surround_with(filePath, lineRange, kind)` — the agent can already do this via Edit, but JDT can do it correctly across multi-line selections with proper exception handling. | Easy via `ASTRewrite`. ~1-2 days each. |
| **Convert switch to if/else / pattern-matching switch** | `convert_switch(filePath, line, target_form)` — JDT 21+ supports pattern-matching switch expressions; conversion both directions is mechanical. | Medium. Java-language-version sensitive. |
| **Spring-aware navigation** (find bean definition for `@Autowired Foo`) | `find_di_bean(typeName)` builds on existing `get_di_registrations` (Sprint 11) — return concrete configuration class + bean method. | Medium. Already partial via `get_di_registrations`. |
| **JPA-aware navigation** (entity relationships, query methods) | `find_jpa_relationships(typeName)`, `find_jpa_query_methods(repository)` — Spring Data JPA query method-name parsing. | Medium-hard. Framework-specific. |
| **Smart paste / copy-with-imports** | Out of scope — agent uses `Edit` and `suggest_imports`; no value-add. | N/A |
| **Local history / refactoring undo** | Out of scope — git is the agent's undo. | N/A |
| **Diff & merge** | Out of scope — agent uses Bash diff/git. | N/A |
| **VCS integration** | Deliberately excluded — agent uses Bash git. | N/A |
| **Code coverage display** | `compute_coverage(testTargets)` returns line/branch coverage map after a `run_tests` invocation. JaCoCo integration. | Medium. Useful for "are my tests adequate?" feedback. |
| **Profiler integration** | Out of scope unless we ship perf-aware tools (e.g. `find_hot_methods(profilingRun)` — async-profiler integration). Hard. | Hard. |
| **Hot reload / debug** | Out of scope — agent doesn't run a debugger; runs tests, reads exceptions. | N/A |

The high-leverage additions are **surround-with** (cheap, used often by agents), **convert-switch** (one tool covers 4 modernisation paths), and **JPA-aware navigation** (huge for Spring Data codebases). Postfix completion has limited agent value (agents don't think in keystrokes).

### Gradle support

Already documented as v1.8.x in `docs/upgrade-checklist.md` for the fork. Three Sprint 13 dep tools (`add_dependency`, `update_dependency`, `find_unused_dependencies`) are Maven-only; the Gradle path needs:

1. Target-platform addition: `org.eclipse.buildship.core` and friends.
2. Build-file detection: `build.gradle` / `build.gradle.kts` (Kotlin DSL).
3. Text-level mutation matching the Maven approach (preserves user formatting + comments).
4. `synchronize` call into Buildship's `GradleBuild` to refresh classpath after.

Plus, the Sprint 11 detection-matrix work means we already detect Gradle projects via the Tooling API; the dep-mgmt tools just need to plug into that detection. **Estimate:** ~2-3 days for the three tools end-to-end. Should land Sprint 14 alongside Android (next item — they share the Gradle root).

### Android territory

Android is its own world — Gradle + Android-specific resources + lifecycle constraints. JavaLens currently sees Android projects only as "a Gradle project that happens to import `android.*` packages". The under-served cases:

- **Manifest analysis** — `analyze_manifest(filePath)` returns declared activities/services/receivers, permissions, intent filters, exported flags. Detect `exported=true` without permission as a security signal.
- **Resource resolution** — `find_resource_usages(R.string.foo)` cross-references resource IDs across Java/Kotlin and XML layouts. Big win for refactoring (renaming a string resource updates every usage).
- **Layout XML symbol surface** — `get_layout_symbols(file.xml)` returns the IDs and types declared in a layout. Enables `find_view_by_id` correctness checks.
- **Lifecycle annotations** — Activity/Fragment/Service lifecycle methods: detect override correctness, missing `super.onX()` calls.
- **ViewBinding / DataBinding generation** — Android generates `ActivityMainBinding` from layout XML; `find_binding_for(layout.xml)` gives the agent a path to the generated source so it can reason about it.
- **Compose-specific** — `@Composable` function detection, recomposition impact analysis, state-hoisting suggestions. Compose is structurally different from XML layouts; needs its own tool family.
- **Migration helpers** — `migrate_to_androidx(packagePath)`, `migrate_to_compose(activityClass)`. These are textual + ASTRewrite refactorings on top of pattern recognition.
- **Lint integration** — `run_android_lint(projectKey)` invokes the Android Lint tool, returns parsed issues. Like `compile_workspace` but for Android-specific patterns.

**Sequencing suggestion:** ship the read-only tools first (manifest analysis, resource usages, layout symbols) — they're the highest-leverage and lowest-risk. The migration helpers and Compose-aware tooling are bigger projects (each its own sprint).

**Strategic question:** is Android worth a dedicated sprint, or is it a "if-the-user-writes-Android-the-existing-tools-just-work" situation? Realistic answer: Android-specific tooling is high-leverage *for Android-heavy users only*. If our user base is predominantly server-side Java (which it is for the manager's current projects), Android tooling is medium priority — Sprint 16 or later.

### Tooling beyond what any IDE does

Here's where JavaLens can genuinely lead, not just match. IDEs are designed for human users with a cursor and an attention span — they emphasise per-position, per-keystroke help. Agents have neither limitation. They can do things humans wouldn't think to do.

- **Cross-codebase pattern detection** — find every place that does *(check for null, throw, return default)* and propose `Optional` refactor. IDEs detect *single-occurrence* patterns; an agent should detect *codebase-wide repetitive patterns* and ask "should we abstract this?".
- **Architectural drift detection** — given a declared package-dependency policy (e.g. `domain.*` cannot import `web.*`), find every violation. ArchUnit territory but reachable via JDT.
- **SOLID violation scanning** — see the dedicated section below.
- **Anti-pattern detection at scale** — God class (LOC + member count + change frequency from git), Feature Envy (method calls foreign types more than its own), Primitive Obsession (long parameter lists of primitives), Data Class (no behaviour). Most of this is computable from JDT's AST + a small heuristic library.
- **"Modernise to Java N" sweeps** — find every loop that could be a stream, every anonymous class that could be a lambda (we have `convert_anonymous_to_lambda` but not the codebase-wide *find* + *batch-apply*), every `Optional.get()` that could be `.map().orElse()`, every conditional that could be pattern-matching switch.
- **Project-convention learning** — given a codebase, infer the conventions (naming, package layout, exception handling) and detect new code that violates them. Probably needs a learned model, not pure JDT — but we can ship the *mechanical* part as a tool.
- **Test-from-spec** — given a method signature and a Javadoc spec, generate the JUnit test class with parameterised cases, edge cases (null, empty, boundary). We have `generate_test_skeleton` (Sprint 13) but it stubs methods; the spec-aware version would actually populate them. Needs LLM reasoning, but we can scaffold the *mechanics*.
- **Architecture-level dead-code** — find unused public APIs (top-level classes / methods never called outside their module). Distinct from per-method `find_unused_code`; this is the "should this whole module exist?" question.
- **Cross-module change-impact** — `analyze_change_impact` exists per-symbol; an agent doing a refactor wants "show me the blast radius of this change across the whole workspace, ranked by call-graph distance".

### Target-form catalogs — what does *good* look like?

Fowler's smells tell us *where* code is bad and Fowler's primitives tell us *how* to refactor. Neither answers the harder question: **what should the result look like?** That's the domain of *target-form catalogs* — pre-existing answers, written down by the field over decades, that encode what "good Java code" means in different contexts.

There isn't one such catalog. There's at least nine, with overlap and tension between them. The agent's job is to know them, blend them, and pick the right one for each situation. The tooling layer's job is to surface them and detect where each applies — without prescribing one as universally right.

#### The catalogs

1. **Fowler — *Refactoring* (1999, 2nd ed. 2018).** Implicit target = "clearer code". Smell + primitive + judgment. We've covered this above; it's the foundation everything else builds on.
2. **Kerievsky — *Refactoring to Patterns* (2004).** Target = a named design pattern, but with a strong meta-rule that **patterns must be earned, not imposed**. The book contains *Refactoring TO Patterns* (Replace Conditional Dispatcher with Command, Move Accumulation to Visitor, Replace Type Code with Class, Form Template Method, etc.) AND *Refactoring AWAY from Patterns* (Inline Singleton when the abstraction adds nothing, Replace Pattern with Idiom when the language has caught up). Detection is bidirectional: "where would this pattern help?" *and* "where is this pattern hurting?".
3. **Gamma/Helm/Johnson/Vlissides — *Design Patterns* (1994), the GoF original.** Target = pattern instantiation as a goal in itself. Less refactoring-focused than Kerievsky; more "build it this way from the start". For a refactoring agent, GoF is mainly useful as a vocabulary for naming what Kerievsky's transformations produce.
4. **Martin — *Clean Code / Clean Architecture* + the SOLID principles.** Target = principle compliance. Sometimes principle-target maps to a specific structure (ISP → split fat interface; DIP → introduce abstraction); sometimes principle-target is more diffuse (SRP → "one reason to change", which is judgment-heavy).
5. **Larman — *Applying UML and Patterns* + GRASP.** Adjacent to GoF, target = responsibility assignment. Patterns like Information Expert, Creator, Low Coupling, High Cohesion, Indirection, Polymorphism, Pure Fabrication, Protected Variations. Interesting because the rules are stated as principles ("information expert: assign responsibility to the class that has the information") that the agent can apply locally.
6. **Beck — *Smalltalk Best Practice Patterns* + *Four Rules of Simple Design*.** Target = "simple". The Four Rules: passes the tests, reveals intention, no duplication, fewest elements. **The most useful arbitration rule in our list** — when other catalogs conflict, "which solution has fewer elements?" cuts through. Beck's TDD-derived stance also gives us "any code without a test is owed a test", which an agent can mechanically pursue.
7. **Evans / Vernon — *Domain-Driven Design* + *Implementing DDD*.** Target = strategic boundaries. Bounded Contexts, Aggregates, Domain Services, Repositories, Anti-Corruption Layer. **Module-level, not class-level.** Detection is hard (it requires understanding the *domain*, not just the code), so the tool layer can mostly only detect *violations* of declared bounded contexts (cross-context references that bypass the ACL) and surface candidates for promotion (entities that have grown into aggregate-root behaviour).
8. **Hexagonal / Onion / Clean Architecture (Cockburn / Palermo / Martin).** Target = dependency direction. Inner layers don't depend on outer layers. Detection: find package-import edges that violate a declared layering policy (ArchUnit-style). Tooling here is mechanical and high-leverage when the project explicitly opts in.
9. **Modernisation idioms** — Java's own evolution as a target catalog. Records (Java 16) replace data classes. Sealed types (Java 17) replace tagged unions. Pattern matching for switch (Java 21) replaces visitor in many cases. `var` replaces verbose type annotations. Streams replace explicit loops. `Optional` replaces null sentinels. **Special property:** these targets get *cheaper* over time — the JDT AST already knows about them; the agent just has to recognise where applying them shrinks code without losing clarity.
10. **Project conventions — the codebase as its own catalog.** Target = "what this codebase already does". The most context-sensitive catalog and often the most important — overriding Fowler's "Long Method" rule when the codebase consistently writes ~80-line methods, deferring to the project's Spring-vs.-Guice DI conventions, matching naming patterns. Detection requires inferring the convention from existing code; tooling can do this mechanically (n-gram analysis on identifiers, package-layout fingerprints, common patterns in the existing test suite).
11. **The anti-target catalog.** What *not* to do. Premature Abstraction, Speculative Generality, God Object, Vendor Lock-in by abstraction, Pattern Soup. Kerievsky's "refactoring away from patterns" overlaps; so does Beck's "fewest elements" rule.

#### What unifies them

All eleven of these are **detection catalogs in disguise**. Each one says: *given input code, here's a rule for spotting where my catalog applies, and here's a target shape*. The MCP tool layer treats them uniformly:

```
find_target_candidates(catalog, kind?, projectKey?)
  → list of (location, applicable_target, evidence, confidence)
```

Where `catalog` is one of `{fowler_smell, kerievsky_pattern, kerievsky_anti, gof, solid, grasp, beck_simple, ddd_violation, layer_violation, modernisation, project_convention, anti_pattern}`. The detection rule per `(catalog, kind)` is mechanical (heuristics on JDT AST + bindings + sometimes git history); the *target shape* is partly mechanical (which pattern, which structure) and partly judgment (the specific naming and decomposition).

This is **a single tool family** with a richly enumerated `(catalog, kind)` axis, not a separate tool per principle. Same pattern as Sprint 11's `find_pattern_usages(kind, query)` and `find_quality_issue(kind, ...)` — discoverable via `tools/list`, agent-friendly because the enum is part of the schema.

#### The bidirectional rule

Kerievsky's deepest insight is that refactoring runs both ways. The agent shouldn't only ask "where would Strategy help?" — it should also ask "where is Strategy hurting?". A class that uses Strategy but only ever has one implementation is an Inline Class candidate. A Visitor pattern in a closed-set hierarchy that no longer mutates is a sealed-type candidate. An abstract base class in a CRUD codebase that no agent or human has touched in 18 months is a Speculative Generality candidate.

**Detection of "patterns hurting"** is sometimes easier than "patterns missing", because the evidence is in the existing structure. We can detect: single-implementation interfaces (potentially Lazy Class / Speculative Generality), abstract methods overridden in only one place (Speculative Generality), patterns wrapped around language features that have since landed (e.g. Singleton when the language has DI, Visitor when the language has pattern-matching switch, Iterator-as-class when the language has streams).

#### Catalog conflict and composition

Catalogs disagree. Examples:

- **Fowler "Long Method" + Beck "fewest elements"** — Fowler says extract; Beck says don't add abstractions you don't need. Resolution: extract only if the extracted method has a name that *adds intent*, not just chunks lines.
- **Kerievsky "Strategy" + Speculative Generality** — Strategy with one implementation is over-engineering. Resolution: don't apply Strategy until at least two implementations exist or are imminent.
- **DDD "Bounded Context" + Hexagonal "single dependency direction"** — DDD might want a Repository that imports domain types (downward dependency) while the layering policy forbids it. Resolution: ACL pattern (DDD acknowledges this case), or explicit annotation that this is a translation layer.
- **Modernisation "use records" + project-convention "use Lombok @Data"** — modernise only if the project hasn't standardised on the alternative.

The agent has to **know all the catalogs** and **arbitrate**. The tool layer can supply the inputs to that arbitration (which catalogs flag this location? what's the codebase convention here?) but can't make the call. This is firmly in the judgment-layer territory — the cleanest argument yet for fine-tuning *eventually* on the catalog-arbitration meta-skill, not on the per-catalog detection.

#### What this means for the tool surface

The naive read is "build a tool per catalog × per kind = 50+ tools". The right read is **one parametric tool family** with a typed `(catalog, kind)` enum:

```
find_target_candidates(catalog, kind, projectKey?, threshold?)
get_target_recipe(catalog, kind, target_location)
  → returns the canonical Fowler-primitive sequence to apply,
    plus the catalog-specified target shape
plan_refactoring(target_location, recipe_id)
  → ties to the multi-step orchestration framework (Sprint 16)
```

Each `(catalog, kind)` pair is a small unit of work — an AST-walk + heuristic rule + a stored recipe. The agent discovers them through `tools/list` exactly the way it discovers `find_pattern_usages` kinds today.

#### Sequencing implication

The previous sequencing recommendation (Sprint 15 = Fowler smell detection) generalises to:

- **Sprint 15 — Target-candidate detection (foundation).** `find_target_candidates(catalog, kind)` infrastructure plus the first 15-20 catalog-kinds: all Fowler smells, the highest-leverage Kerievsky patterns (Replace Conditional with Command, Compose Method, Replace Type Code with Class), the SOLID violations from the previous SOLID table, layer violations. ~1-2 weeks because the framework is the work; each kind after that is half a day.
- **Sprint 16 — Multi-step orchestration framework.** Same as before; gates Sprint 17.
- **Sprint 17 — Recipe execution.** `get_target_recipe` + `plan_refactoring` + `apply_refactoring_plan`. Lets the agent go from "I see a Long Method here" to "apply Compose Method recipe" in one move.
- **Sprint 18 — Catalog expansion.** GRASP, DDD violation detection, modernisation idioms, project-convention learning. Each adds N more catalog-kinds to the existing tool family.
- **Sprint 19 — Anti-target detection.** Speculative Generality, Premature Abstraction, single-implementation-Strategy, etc. These complete the bidirectional Kerievsky picture.

Total at the end: one parametric tool family, ~80 catalog-kinds across the eleven catalogs, all discoverable through `tools/list`.

### Refactoring with small steps — Fowler

Martin Fowler's *Refactoring* (1999, 2nd ed. 2018) is the canonical small-step refactoring catalogue. **It has three halves, not one** — and JavaLens currently covers only the first.

#### Half 1 — Refactoring primitives (the *how*) — covered

Each Fowler move is type-preserving, behaviour-preserving, and individually compileable. **JavaLens implements ~15 primitives** (rename, extract method/variable/constant/interface, inline, move-class/method/package, pull-up, push-down, encapsulate-field, change-method-signature, organize-imports, convert-anonymous-to-lambda, etc.).

Strict-Fowler primitives still missing:
- *Replace Magic Number with Symbolic Constant* — partial via `extract_constant` but doesn't *find* magic numbers automatically.
- *Replace Type Code with Subclasses / Strategy* — needs multi-step orchestration (Sprint 16 framework).
- *Hide Method / Hide Delegate* — visibility-tightening; trivial via JDT.
- *Replace Parameter with Method Call* — extract a temp inside the called method.
- *Introduce Null Object* — pattern-targeted; rides on Kerievsky tooling.

#### Half 2 — Smell detection (the *where* and *why*) — **NOT covered**

The primitives tell you *how* to refactor. They do not tell you **where the bad code is** or **why it's bad enough to warrant refactoring**. That's the role of Fowler's *Bad Smells in Code* catalogue, which currently has zero JavaLens tooling.

Fowler's smells, with proposed MCP detection tools:

| Smell | Detection heuristic | Pointed-to refactoring |
|---|---|---|
| **Long Method** | Method body LOC > N (configurable, default 30); cyclomatic complexity > M. AST-walk on method body. | Extract Method × N, Replace Temp with Query. |
| **Large Class / God Class** | Class member count > N (default 25); LOC > M (default 500); fan-in across the codebase. | Extract Class, Extract Subclass, Extract Interface. |
| **Long Parameter List** | Method `parameters().size() > 4`. | Introduce Parameter Object, Preserve Whole Object. |
| **Divergent Change** | Same class touched in commits with disjoint topics — needs git-history correlation. | Extract Class. |
| **Shotgun Surgery** | A given method/symbol's references span > N classes/packages. (we have `analyze_change_impact` — extend to flag this.) | Move Method, Move Field, Inline Class. |
| **Feature Envy** | Method calls foreign-type members more than its own type's members. AST-walk + binding resolution. | Move Method, Extract Method + Move. |
| **Data Clumps** | Same parameter-name tuple recurring across N+ method signatures. Cross-method signature analysis. | Extract Class, Introduce Parameter Object. |
| **Primitive Obsession** | Long parameter lists of `int` / `String` carrying domain semantics. Heuristic + naming analysis. | Replace Type Code with Class, Extract Class. |
| **Switch Statements** | `switch` over a type code where polymorphism would fit. AST-walk for switches on `int` / enum-like usage. | Replace Conditional with Polymorphism, Replace Type Code with Subclasses. |
| **Parallel Inheritance Hierarchies** | Two type hierarchies with structurally parallel subclass relationships. Hierarchy diff. | Move Method/Field to merge hierarchies. |
| **Lazy Class** | Class with very few members, low fan-in, and no public API surface. AST + reference analysis. | Inline Class, Collapse Hierarchy. |
| **Speculative Generality** | Abstract classes/methods with only one implementation; unused parameters. | Inline Class, Remove Parameter. |
| **Temporary Field** | Field used only inside one method; otherwise null/uninitialized. AST + data-flow. | Extract Class. |
| **Message Chains** | `a.b().c().d().e()` patterns of length > N. AST-walk for chained method calls. | Hide Delegate. |
| **Middle Man** | Class whose methods all delegate to a single other class with no added behaviour. AST analysis. | Remove Middle Man. |
| **Inappropriate Intimacy** | Two classes that access each other's internals heavily. Cross-class binding analysis. | Move Method/Field, Change Bidirectional Association to Unidirectional. |
| **Refused Bequest** | Subclass overrides parent methods with `throw new UnsupportedOperationException`. AST scan. | Replace Inheritance with Delegation. |
| **Comments** | Long comment blocks (LOC > N) in method bodies — usually mask poor naming. | Extract Method (with intention-revealing name), Rename. |
| **Duplicated Code** | Token / AST-similarity detection across methods. Has known algorithms (PMD CPD); our move would be JDT-AST-flavoured. | Extract Method, Pull Up Method, Extract Class. |

**Tool sketch:** `find_smell(kind, projectKey, threshold?)` for each smell; returns ranked candidates with location + a heuristic score + a pointer to the refactoring(s) that apply.

This is the **biggest single gap** in our current tool surface relative to "best Java dev". 18-20 detection tools, each individually small (~1-2 days of JDT-AST work each), covering ~95% of Fowler's smell catalogue. **High-leverage** because it transforms the agent's loop from "the user told me what to fix" → "I scanned the codebase and propose this prioritised list of fixes".

#### Half 3 — Target-form recognition (the *to-what*) — partly judgment, partly mechanical

Given a smell + applicable refactoring, the agent still needs to know **what the result should look like**. "Extract Method" only specifies that we extract; it doesn't say what the extracted method should be named, what its signature should be, where in the class it should live.

Some of this is judgment territory (Claude is already good at it). Some is mechanical and JavaLens can help:

- **Naming suggestions from context** — given a code block being extracted, what verbs/nouns appear in its variable names, calling context, javadoc nearby? Mechanical-ish; could be a tool.
- **Find similar refactored shapes elsewhere in the codebase** — "this codebase prefers X-pattern over Y-pattern; here are 5 examples in this project". JDT pattern-search; reuses `find_pattern_usages` infrastructure.
- **Project-convention learning** — see "Tooling beyond what any IDE does" above. Cross-cutting; benefits target-form recognition specifically.
- **The "what should it look like after" question** — this is mostly model judgment. Tools can supply *evidence* (similar examples, naming context, project conventions) but the final taste call is the agent's.

The honest framing: **the affordance layer ends here**. We can hand the agent excellent inputs for target-form decisions (smell location, applicable refactoring, naming context, project precedent). The agent then exercises judgment on what specifically to produce. This is the cleanest seam between tooling and model — and the strongest argument for "ship affordances, fine-tune judgment later".

#### What this means for sequencing

The ~15 primitives we have are necessary but not sufficient. Until we ship smell detection (Half 2), the agent can refactor anything you point at, but it can't find what to refactor. That's a much larger gap than the missing primitives.

Concrete recommendation: **Sprint 18 (SOLID) and a new Sprint targeting "Fowler smell detection"** are likely the two highest-leverage additions to the tool surface, ahead of even Kerievsky orchestration. Smell detection is broader (covers the full Fowler catalogue, not just SOLID's 5 principles) and mechanically simpler (heuristic JDT-AST walks, no orchestration framework needed).

Reordered sequencing suggestion:

1. **Sprint 14 — Gradle + Android read-only.**
2. **Sprint 15 — Fowler smell detection.** ~18 detection tools, each covered by an AST + heuristic walk. Enables the agent to find what to refactor before applying any primitive. *Highest single-sprint leverage gain.*
3. **Sprint 16 — Modernisation sweeps.**
4. **Sprint 17 — Multi-step orchestration framework.**
5. **Sprint 18 — Kerievsky "Refactoring to Patterns".**
6. **Sprint 19 — SOLID detection** (overlap with Sprint 15's work — SOLID violations are a subset of Fowler smells dressed differently).
7. **Sprint 20+ — Cross-codebase, architectural drift, project-convention learning.**

### Worked example: SOLID

Worked deep-dive into one of the catalogs from the section above. The SOLID principles were assembled and named by Robert C. Martin (a.k.a. Uncle Bob) across his *Clean Code* (2008) and *Clean Architecture* (2017) work, building on earlier ideas from Bertrand Meyer (Open/Closed, *Object-Oriented Software Construction*, 1988) and Barbara Liskov (Liskov Substitution, 1987 conference keynote and 1994 paper with Wing). The acronym crystallises five rules that, taken together, push code toward replaceable, low-coupling, high-cohesion designs.

| Principle | Statement | Detection heuristic | Target shape |
|---|---|---|---|
| **S — Single Responsibility Principle** | "A module should have one and only one reason to change" (Martin). One class = one stakeholder concern. | High class LOC + many distinct topic clusters in member-name vocabulary + high git churn correlated with multiple feature topics. Cross-signal; hardest of the five. | Extract Class along the discovered topic boundary; one of the resulting classes keeps the original name. |
| **O — Open/Closed Principle** | "Software entities should be open for extension, but closed for modification" (Meyer 1988; popularised by Martin). New behaviour added by *adding* code, not editing existing code. | Class that's been modified (git diff over last N commits) for every new feature in some feature dimension. Needs git history. | Introduce abstraction (interface or strategy) at the modification axis; new behaviours implement the abstraction without touching existing code. |
| **L — Liskov Substitution Principle** | "Subtypes must be substitutable for their base types" (Liskov 1987). A function that works on the base type must work on any subtype without surprise. | Subclass overrides that strengthen preconditions (add `throws`), weaken postconditions, or partially-implement (`throw new UnsupportedOperationException`). AST + signature comparison + override-body inspection. | Replace inheritance with composition (delegation); or split the hierarchy so each subtype is a true Liskov-compatible subtype of *its* base. |
| **I — Interface Segregation Principle** | "Clients should not be forced to depend on methods they do not use" (Martin). Many small purpose-specific interfaces beat one fat interface. | Interface with N+ methods where each implementation only calls a measurable subset. Call-graph analysis across implementations. | Split the fat interface along the call-pattern clusters; original interface becomes a composition of the smaller ones for backward compatibility. |
| **D — Dependency Inversion Principle** | "Depend on abstractions, not on concretions" (Martin). High-level modules don't import low-level modules; both depend on abstractions. | Concrete-type field declarations / parameters / returns where an abstract supertype exists *and* is the actual usage shape. JDT binding-resolution + interface analysis. | Introduce or use an existing abstraction at the dependency edge; flip the import direction so the inner module owns the abstraction and the outer module implements it. |

LSP, ISP, and DIP are reachable via JDT alone (AST + bindings + call-graph). SRP and OCP need git-history correlation — adding a `git log` shell-out from JavaLens is straightforward.

**Tool sketch:** under the unified target-catalogs framing above, this is `find_target_candidates(catalog="solid", kind="srp" | "ocp" | "lsp" | "isp" | "dip", projectKey?)` — the same parametric tool family that handles Fowler smells, Kerievsky patterns, modernisation idioms, etc.

### Worked example: Kerievsky's pattern catalog

Joshua Kerievsky's *Refactoring to Patterns* (2004) is the bridge between Fowler's small-step catalogue and the GoF design patterns. The book's key contribution isn't a new collection of patterns — it's the recognition that **patterns must be discovered, not imposed**. Each transformation is a sequence of Fowler primitives whose target is a named pattern instantiation, and the catalogue runs in both directions: *toward* patterns when complexity warrants, *away from* patterns when they've outlived their usefulness.

A representative slice:

| Kerievsky transformation | Direction | Detection signal | Target |
|---|---|---|---|
| **Replace Conditional Dispatcher with Command** | Toward | `switch` (or if-else chain) over a type-coded action with N+ cases, each with non-trivial body. | One Command interface, one implementation per case, dispatcher maps type to implementation. |
| **Move Accumulation to Visitor** | Toward | Method that accumulates info across a tree of types via type-dispatched `if/instanceof`. | `Visitor` interface with a visit-method per node type; element classes get `accept(visitor)`. |
| **Replace State-Altering Conditionals with State** | Toward | Class with N+ if-else branches on the same internal state field, each branch mutating that field. | State pattern: each branch becomes a `State` subclass; transitions become state-object swaps. |
| **Compose Method** | Toward | Long method with multiple disjoint sections (often visible as commented section breaks). | A method body that reads as a top-down sequence of intention-revealing helper-method calls. |
| **Replace Type Code with Class** | Toward | `int` or `String` parameter passed around as a domain code; widely-used. | A small class (or in modern Java, a record + sealed type) replacing the primitive. |
| **Inline Singleton** | Away | Singleton class whose lifecycle and uniqueness no longer matter (e.g. now constructed by DI). | Plain class; old `getInstance()` callsites updated to direct construction or DI. |
| **Replace Pattern with Idiom** | Away | Pattern wrapped around a language feature that has since landed (Iterator-as-class when streams exist; Visitor when pattern-matching switch exists; Singleton when DI exists). | Native language feature; pattern boilerplate deleted. |
| **Form Template Method** | Toward | Two near-identical methods that differ only in a few specific steps. | Abstract base method with the common skeleton; subclasses fill in the variant steps. |

Kerievsky's "away" direction is what distinguishes the book from the GoF original (Gamma/Helm/Johnson/Vlissides 1994) — GoF is a pattern *catalog*, Kerievsky is a pattern *application discipline*. The agent benefit: detection rules for "where pattern X *would help*" are well-known; detection rules for "where pattern Y *is hurting*" are less obvious and equally important. **Inline Singleton, Replace Pattern with Idiom, and Inline Class are the highest-leverage "away" transformations** because language evolution keeps creating opportunities for them.

**Tool sketch:** `find_target_candidates(catalog="kerievsky", kind="replace_conditional_with_command" | "move_accumulation_to_visitor" | "compose_method" | "inline_singleton" | …)`. Each `kind` is a small AST-walk + heuristic; the recipe (the multi-step Fowler-primitive sequence to apply) lives behind `get_target_recipe`.

### Multi-step MCP tools — feasibility

The honest answer: **yes, feasible, and probably necessary** for the more interesting refactorings.

What it needs:

1. **A planning tool** — `plan_refactoring(target, kind)` returns a list of `{step, primitive_tool_call, expected_state_after, rollback_to}` records.
2. **An execution tool** — `apply_refactoring_plan(plan_id, options)` walks the plan, calls the primitive tools in sequence, runs `compile_workspace` + `run_tests` between steps if `options.validate_each=true`, rolls back to the last good state on failure.
3. **A diff observability tool** — `inspect_refactoring_state(plan_id)` returns the diff applied so far, which steps have run, which are pending. The agent uses this to reason about whether to continue, modify, or abort.

The challenge is **rollback**. Fowler refactorings are *individually* compileable but the rollback semantics need to be precise: a failed step at index N requires undoing steps N-1, N-2, ... back to the pre-refactor state. JDT's `Change` API has `getUndo()` for LTK refactorings; we can stack those.

**This is real engineering** — probably a full sprint just for the orchestration framework. But the payoff is substantial: it lets the agent attempt complex refactorings (Replace Conditional Dispatcher with Command, Move Accumulation to Visitor, Modernise All Switches) as single agent invocations rather than micromanaging step-by-step.

### The fine-tuning question — is tooling enough?

JavaLens provides **affordances**: what's *possible* to do correctly via JDT. The model — Claude — provides **judgment**: *when* to do which thing, *which* refactoring fits the situation, *what* the resulting code should read like.

These are different problems with different solutions:

**Affordances** are mechanical. They scale linearly with engineering effort. Every JDT primitive we expose, every multi-step orchestration we ship, expands the set of things the agent can attempt without making mistakes. There's a clear ceiling — once we've covered Fowler, Kerievsky, SOLID detection, modernisation sweeps, and Android, we've covered ~95% of what a senior Java dev does mechanically.

**Judgment** is harder. Recognising "this is a Strategy pattern situation", "this method violates SRP because reasons X and Y", "this code reads better with a record than a class" — these are pattern-matching problems with a high taste threshold. An untrained Claude is already very good at them; a Claude trained on a corpus of {good Java codebase, refactoring trace, before/after} would be better. The marginal gain from fine-tuning is real but bounded — Claude's pre-training already covers the literature (Fowler, Kerievsky, Martin, Bloch).

**The 80/20:**
- **80% of "best Java dev" comes from rich, correct tooling.** A model with weak judgment but excellent tools beats a model with strong judgment but only `Edit` + `grep`. We've seen this: Claude with the Sprint 11 LTK refactorings handles Java better than Claude with `Edit` alone.
- **The last 20% is judgment.** Tools tell the agent *what's possible*; only judgment tells it *what's appropriate*. For the highest-quality work — pattern recognition, taste calls, architectural moves — fine-tuning or RLHF on Java-specific refactoring traces would help.

**Concrete answer to the fine-tuning question:**
- Ship rich tooling first. We're at 73 tools per workspace; the marginal value of tools 74–150 is still very high (Android, Gradle, Kerievsky orchestration, SOLID detection, modernisation sweeps).
- Fine-tuning makes sense **after** the affordance set is saturated. Until then, every model release picks up tooling improvements automatically (no fine-tuning required), so we'd be optimising the wrong layer.
- A reasonable horizon: Sprint 13 (now) → Sprints 14–18 expand affordances by ~50 tools across Android / Kerievsky / SOLID / modernisation / Gradle → at that point, fine-tuning a Java-specific Claude on traces of `JavaLens-mediated refactoring sessions` becomes the natural next step.

The right framing: **JavaLens is the affordance layer. Fine-tuning is the judgment layer.** Don't optimise the latter while the former still has obvious gaps.

### Sequencing recommendation (informal)

Not committed plan, but a defensible order based on the above:

1. **Sprint 14 — Gradle + Android read-only.** Gradle dep tools (~3 days), Android manifest/resource/layout read tools (~5 days). Gets us the framework-level coverage we currently lack.
2. **Sprint 15 — Modernisation sweeps.** Find-and-batch-apply for `convert_anonymous_to_lambda`, switch-to-pattern, loop-to-stream, `Optional` introduction. ~1 week.
3. **Sprint 16 — Multi-step orchestration framework.** `plan_refactoring` + `apply_refactoring_plan` + rollback support. ~1 week. Foundation for Sprint 17.
4. **Sprint 17 — Kerievsky "Refactoring to Patterns".** Replace Conditional Dispatcher with Command, Move Accumulation to Visitor, etc. ~1 week riding on Sprint 16's framework.
5. **Sprint 18 — SOLID detection.** Find-violations tools across all five principles, plus the propose-fix wiring back to existing refactorings. ~1 week.
6. **Sprint 19+ — Cross-codebase pattern detection, architectural drift, project-convention learning.** Less mechanical, more design-heavy.

Each sprint adds 5–15 tools. After Sprint 18 we'd be at ~120 tools per workspace. After Sprint 19+ the per-workspace count plateaus and the work shifts to fine-tuning + judgment-layer improvements.

### Open questions for next planning session

- Where does the Sprint 14 boundary actually fall — Gradle-only or Gradle + Android together? (If a single sprint, ~8 days; if split, ~5 + ~5 across two sprints.)
- Is there evidence from agent-usage logs (when we have them) of which tools are most-called? That should reorder the sequencing above.
- For SOLID detection: do we ship the heuristic-based version (mechanical, fast, has false positives) or wait for a learned model? Heuristic-first means false positives, but the agent can triage.
- For "best Java dev of all time": is the goal *autonomous* (agent does it alone) or *assistive* (agent + human pair)? The tool surface is the same either way; the prompt-engineering and UX above differ.
