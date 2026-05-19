# Future Sprint — javalens as a networked service (HTTPS, auth, multi-user)

> **Status: future outlook, no committed sprint number.** This document captures a strategic direction surfaced during the 2026-04-26 Sprint 10 architecture discussion. It is **not** scheduled. It is recorded so that when the driver materializes — likely a multi-developer team or shared analysis service — the work has a starting point and the decision rationale is preserved.

## Context

Today javalens is a **local, single-user developer tool**. It runs on your machine, talks to your MCP clients (cursor / claude / antigravity / IntelliJ) via stdio, indexes your local Java projects. javalens-manager configures and orchestrates these processes for one user on one machine.

During the Sprint 10 design pass we considered four transport architectures for the multi-project workspace feature:

| Option | Transport | Scope | Decision |
|---|---|---|---|
| Stdio + multi-path startup | stdio | Local single-user | Sprint 10 baseline |
| Stdio + file-watch (`workspace.json`) | stdio | Local single-user, transparent live updates | **Chosen for Sprint 10** |
| HTTP localhost servlet | HTTP, no auth, localhost-only | Local single-user, shared process per workspace | Considered, deferred — file-watch is sufficient for local |
| HTTPS + auth + multi-user | HTTPS+auth | **Networked, multi-developer team** | This document |

The first three serve the same use case (single developer on a single machine). The fourth is a fundamentally different product shape and is the subject of this document.

## When this becomes worth building

Real signals — not hypotheticals:

- **A team wants to share a workspace.** Two or more developers analyze the same Java codebase from their own machines and would benefit from a shared, pre-indexed JDT workspace running on a beefy server.
- **CI / agent farms.** Automated agents (refactor bots, code-review assistants, migration runners) run on infrastructure separate from developer machines and need consistent JDT analysis service.
- **Cold-start cost is unacceptable.** Big workspaces (think: full enterprise monolith) take minutes to index. Pre-indexing on a server and connecting from many clients amortizes that cost.
- **Compliance / centralization.** A team policy mandates that source code analysis runs in a controlled environment, not on individual developer laptops.

If none of those exist, **don't build this.** The local stdio model fits the realistic single-developer use case (1-3 workspaces, big-but-bounded JATS-style projects, ~weekly add/remove events) and adds no friction.

## What it actually requires

A networked javalens service is **a different product** from the local tool. The honest checklist:

### Transport
- HTTPS with TLS certificates (Let's Encrypt or org CA).
- MCP over HTTP+SSE or Streamable HTTP transport (which the MCP Java SDK supports — but javalens's current hand-rolled `McpProtocolHandler` would need either an SDK migration or a parallel HTTP layer).
- Reverse proxy front (nginx / Caddy) for TLS termination, rate limiting, IP allowlisting.

### Authentication
- Token-based auth (JWT, API keys, or OAuth 2.1 — MCP spec defines the OAuth flow for clients).
- Per-user identity tied to MCP sessions.
- Token issuance / rotation / revocation.

### Authorization
- Per-workspace access control: which users can see which workspaces?
- Per-tool authorization: should refactoring tools be agent-callable on a shared workspace? Probably not without explicit per-session opt-in (writes are dangerous).
- Audit log: who called what tool with what arguments at what time. Critical for compliance use cases.

### Multi-user session management
- Session isolation: two users analyzing the same workspace should each have their own JDT search scope (or at least their own request lifecycle).
- Session quotas: prevent one user from monopolizing the JDT indexer with a 10-minute deep analysis.
- Concurrent-edit semantics: if two users have refactor tools open, what happens when one moves a class while another is searching for references in it?

### Operations
- Process supervisor (systemd / supervisord) — javalens must auto-restart on crash without losing user sessions if possible.
- Log aggregation, metrics export (Prometheus), alerting.
- Backup / restore for the JDT data dir (re-indexing big workspaces from scratch is expensive).
- Health endpoints separate from MCP endpoints (load balancers care about HTTP 200, not JSON-RPC).
- Graceful shutdown: drain in-flight requests, persist any pending state, close cleanly.

### User management UI
- A web UI (or extension to javalens-manager) for: creating users, issuing tokens, granting workspace access, viewing audit logs, managing workspaces.
- This is its own product surface and probably its own sprint.

### Realistic scope estimate
A first usable version: **2-3 sprints minimum** (~6-8 weeks of focused work). A production-ready version with all the operational polish: **larger and ongoing**.

## Why this reinforces "stay on the fork"

This is the primary new argument for not PR'ing back to upstream `pzalutski-pixel/javalens-mcp`:

1. **Upstream's design is local stdio.** The README architecture diagram explicitly shows `JSON-RPC over stdio` with a single MCP client. Adding HTTPS + auth + multi-user fundamentally diverges from upstream's product identity. It's not a small "feature to PR back" — it's a different product.
2. **The work is large and opinionated.** Auth/TLS/user-management decisions are highly opinionated and depend on the deployer's environment (cloud provider, IdP, compliance regime). Upstream cannot serve all those choices well; a fork can pick one path that fits one user's needs.
3. **Operational ownership.** If javalens runs as a hosted service for your team, you own the operations: SLAs, security patches, ops runbook, oncall. Upstream cannot take this on.
4. **Independence already paid for.** Sprint 9-11 work has been built under the portability constraint (ADR 0004). The networked-service work would extend that fork unambiguously, not contaminate the local-tool design upstream cares about.

The pre-existing reasons to defer the upstream PR (Sprint 11 backlog "Independence posture" section) all still apply. This is one more, and it's the strongest.

## Adjacent work that should land first

If/when this sprint becomes real, these prerequisites should be in place:

- **Sprint 10 + 11 stable**: multi-project workspaces (file-watched), Tycho/Bundle pool detection, tool consolidation, structural refactorings — all proven on the local product before extending to a networked one.
- **HTTP transport in javalens-mcp** (could be Sprint 12 or 13 standalone — local-only HTTPS-disabled HTTP for testing): the foundation that this sprint extends with auth/TLS.
- **MCP Java SDK migration** (likely): replacing the hand-rolled `McpProtocolHandler` with the official SDK gets HTTP+SSE / Streamable HTTP transport for free, and tracks future MCP spec evolution. Worth doing before adding auth / session management on top of a hand-rolled implementation.
- **Production-grade test coverage** of the existing tools — exposing them to a multi-user network surface raises the bar for tool-side correctness (input validation, error handling, resource cleanup).

## Manager-side UX hooks that gain real value in this direction

Two manager-side ideas were originally captured under Sprint 11 Phase G (Tier 4) but make far more sense once a networked, multi-user javalens deployment exists. For a single-developer local tool they are convenience features at best; for a shared service they become operationally meaningful. Captured here so they aren't lost.

### Per-workspace data-dir size

What it is: surface the disk footprint of each workspace's JDT data dir (`<data_root>/workspaces/<name>/`) in the manager UI — workspace card header, Diagnostics card, or both. Backend: a small Tauri (or networked-service API) call computing recursive size; UI reads it on demand or caches with a refresh button.

Why it matters more in the networked direction:

- **Capacity planning.** For one developer with 1-3 workspaces, a `du -sh` from a terminal answers the question. For a hosted service with many workspaces and many users, per-workspace size is a first-class operational metric — feeds into capacity dashboards and cleanup-policy decisions.
- **Quota enforcement.** A per-user or per-team workspace quota is a natural feature on a shared deployment. Visibility is the precursor to enforcement.
- **Cleanup nudges.** "This 12-GB workspace hasn't been touched in 90 days" only fires once you have visibility plus access logs.

*Spec questions when scheduled:* refresh cadence (on demand, periodic, push from server); disk-only vs disk + RAM (RAM is per-PID, OS-specific, harder to make portable); whether to surface per-user totals as well; whether the metric flows through the same API as audit-log endpoints.

### Workspace presets / templates (export / import / share)

What it is: serialize a workspace definition (project paths, settings, optional metadata) as a portable JSON file, importable on another machine.

Why it matters more in the networked direction:

- **The "share with teammate" use case is the entire point of multi-user.** On a hosted deployment, a workspace preset becomes a *template*: "everyone joining the JATS workspace pulls these 12 OSGi bundles, with these analysis settings, against this target platform." That's a server-side resource, not a local file.
- **Reproducibility.** CI / agent farms need declarative workspace specs, not click-through-the-UI configuration.
- **Compliance.** A signed workspace template ("this is the approved JATS workspace shape") is something an org-policy environment will demand.

The local single-developer use case (`cp ~/.cache/javalens-manager/workspaces/X` between two of your own machines) is a thin one — not worth shipping presets just for it.

*Spec questions when scheduled:* preset payload (path resolution, env-var substitution, project-vs-workspace-level settings); on a networked service, whether presets are server-stored objects with their own access control; round-tripping with `.code-workspace` files; signing / provenance metadata.

## Open questions to resolve when scheduled

- **Single-tenant or multi-tenant?** One javalens deployment per team, or one per user? Affects the auth model.
- **Self-hosted only, or also a managed offering?** The latter is a different business shape.
- **Which IdP integration?** OIDC, SAML, GitHub OAuth, custom — depends on team environment.
- **What's the minimum viable feature set to ship a first usable network release?** Probably: HTTPS + token auth + per-workspace ACL + audit log. Skip OAuth, skip web UI v1.
- **Compatibility with javalens-manager?** Does the local manager UI also become a networked-service admin client, or is admin a separate web UI? Latter is cleaner.
- **Pricing / licensing model** if this becomes a paid product (separate from the open-source local tool).

## Out of scope for this document

- Detailed implementation plan (deferred until scheduled).
- Choice of HTTP framework, auth library, etc. (depends on requirements at scheduling time).
- Comparison to existing networked code-analysis services (SonarQube, JetBrains Code With Me, etc.) — useful context when scheduled.

## Cross-references

- Sprint 10 backlog (`docs/sprints/sprint-10-backlog.md`): port-as-workspace decision and the local file-watch model that this networked service would supersede if/when it ships.
- Sprint 11 backlog (`docs/sprints/sprint-11-backlog.md`): "Independence posture" section, which this document strengthens.
- ADR 0004 (`docs/adr/0004-helper-portability-constraint.md` in `javalens-mcp` fork): the portability constraint that survives any future transport change.
