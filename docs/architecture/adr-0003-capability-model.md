# ADR-0003: Effective support = static matrix ∩ runtime probe

- Status: accepted
- Date: 2026-07-12

## Context

Compile-target support is necessary but not sufficient (§6.6): a Linux
build may run under Wayland without ydotool; macOS actions may lack TCC
permissions. The UI must show honest, localized reasons — and never
hardcode per-OS hacks in React (§2.3).

## Decision

- Every `ActionDef` declares a static `PlatformSupport` per OS plus an
  optional required `Capability`.
- Each platform crate probes at startup (and on system changes) into a
  `CapabilitySnapshot` carrying `Available / Degraded / Unavailable` with a
  technical reason and an optional machine-readable fix
  (`InstallTool` / `GrantPermission`).
- The registry enforces the intersection at invoke time and maps outcomes
  onto `Unsupported` (→ `action.unsupportedOnOs`, `{{os}}` =
  `snapshot.os_label()`, e.g. `Linux (Wayland)`), `MissingTool`
  (→ `action.needsTool` + install hint) and `NeedsOsPermission`
  (→ `action.needsPermission` + "Fix it"). Never blame the OS for a
  solvable setup issue (§8.1).
- The frontend renders the same computation from data
  (`effectiveSupport()` in `packages/editor`), so the badge and the runtime
  error can never disagree.

## Consequences

- "Wayland without ydotool ⇒ MissingTool(ydotool)" is a unit test on both
  sides of the IPC boundary.
- Adding a probe never touches React components — only data flows change.
