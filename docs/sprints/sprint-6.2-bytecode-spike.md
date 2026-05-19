# Sprint 6.2 Bytecode/JAR Spike

## Question

Can `javalens-manager` auto-import bytecode-heavy folders (JAR repositories, decompile targets) safely by default?

## Current Runtime Contract

- The runtime launches JavaLens with:
  - `JAVA_PROJECT_PATH=<projectPath>`
  - `java -jar ... -data <workspaceDir>`
- Project discovery in Sprint 6.2 intentionally targets source-oriented Java projects:
  - Maven/Gradle
  - Eclipse/PDE

## Bytecode/JAR Reality Check

- JAR-only folders are not guaranteed to behave like regular Java workspace roots.
- Decompile/index behavior depends on upstream JavaLens/JDT capabilities and classpath context.
- Large JAR trees can create significant indexing and memory pressure if imported as normal projects.

## Performance Risks

1. Large recursive scans over `lib/`, `target/`, repository caches, or vendor JAR folders.
2. Runtime startup delays due to heavy classpath/index construction.
3. Larger managed workspace state and logs per imported entry.

## Recommendation

- Keep JAR/bytecode auto-import **disabled** for now.
- Restrict workspace auto-discovery to Maven/Gradle and Eclipse/PDE projects.
- Add optional future import mode for JAR roots behind an explicit toggle.

## Future Validation Checklist

Before enabling JAR auto-import:

1. Confirm upstream JavaLens behavior on representative bytecode-only samples.
2. Measure startup/index times on small, medium, and large JAR sets.
3. Define guardrails:
   - max file count
   - max total size
   - excluded directory patterns
4. Add user-facing warning before importing bytecode-heavy paths.
