# Feature Specification: Rust Documentation Availability Baseline

**Feature Branch**: `001-improve-rust-crate`

**Created**: 2026-07-12

**Status**: Draft

**Input**: User description: "Improve Rust crate and module documentation coverage and availability across workspace"

## Current Baseline (from repository audit)

- Missing crate-level docs (`//!`) in 9 crates:
  `libs/gsm-mod-manager`, `libs/gsm-monitor`, `libs/gsm-serde`, `libs/gsm-shared`, `libs/ini-derive`,
  `apps/enshrouded`, `apps/gsm-cli`, `apps/palworld`, `tools/env-parser`.
- Clippy doc-section gaps:
  - `libs/gsm-instance`: 21 missing `# Errors`
  - `libs/gsm-notifications`: 3 missing `# Errors`
  - `libs/gsm-shared`: 3 missing `# Errors`
  - `libs/gsm-mod-manager`: 2 missing `# Errors`
  - `libs/gsm-serde`: 2 missing `# Errors`
  - `libs/ini-derive`: missing `# Panics`
  - `libs/gsm-backup`: missing `# Panics`
- `cargo test --doc --workspace` currently passes.

## User Scenarios & Testing _(mandatory)_

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - Enforce doc availability gates (Priority: P1)

As a maintainer, I need CI-verifiable documentation quality gates so missing API docs and missing rustdoc sections (`# Errors`, `# Panics`) are detected automatically across workspace crates.

**Why this priority**: Without an enforced gate, coverage regresses over time and documentation remains inconsistent.

**Independent Test**: Run documented lint/test commands and verify the workspace fails when required doc conditions are missing, and passes when present.

**Acceptance Scenarios**:

1. **Given** workspace crates, **When** doc linting is run, **Then** missing required rustdoc sections on public APIs are reported.
2. **Given** corrected docs, **When** doc linting is run in CI, **Then** the documentation gate passes.

---

### User Story 2 - Improve crate-level discoverability (Priority: P2)

As a contributor, I need each crate to have a clear crate-level overview so I can understand purpose, key modules, and usage quickly.

**Why this priority**: Crate-level docs reduce onboarding time and make generated docs usable as a reference.

**Independent Test**: Build docs and verify each crate root exposes top-level module documentation.

**Acceptance Scenarios**:

1. **Given** a crate without root docs, **When** docs are generated, **Then** it includes a crate-level description with scope and usage context.

---

### User Story 3 - Standardize public API docs (Priority: P3)

As an integrator, I need consistent docs for public structs/functions (purpose, parameters, return behavior, failures) so I can use APIs without reading source internals.

**Why this priority**: Public API consistency improves reliability of integration and reduces support burden.

**Independent Test**: Sample public APIs across target crates and verify they have normalized doc sections and examples where appropriate.

**Acceptance Scenarios**:

1. **Given** a public API returning `Result`, **When** reviewed in rustdoc, **Then** it includes an `# Errors` section describing failure modes.
2. **Given** a public API that can panic, **When** reviewed in rustdoc, **Then** it includes a `# Panics` section.

---

### Edge Cases

- Crates that expose only binaries (no library API) still require actionable crate/module guidance.
- Generated or macro-heavy code paths where docs are difficult to attach must have explicit documented exceptions.
- Public APIs re-exported from other modules must not lose doc visibility.
- Doctest examples that are platform-dependent must be marked appropriately (`no_run`/`ignore`) while remaining useful.
- APIs intentionally using `expect`/`panic!` for invariant enforcement must document panic conditions, not hide them.

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: The workspace MUST define and document the canonical command(s) for doc validation (lint + doctests).
- **FR-002**: Each crate MUST have crate-level documentation (`//!`) describing purpose and key modules.
- **FR-003**: Public functions returning `Result` MUST include `# Errors` documentation.
- **FR-004**: Public functions that may panic MUST include `# Panics` documentation.
- **FR-005**: Publicly exposed domain types and operations MUST include concise purpose-oriented docs.
- **FR-006**: Documentation checks MUST be runnable locally and in CI with deterministic output.
- **FR-007**: Any intentional documentation exceptions MUST be explicit and justified in code comments or lint configuration.
- **FR-008**: Existing passing doctests MUST remain passing after documentation updates.

### Key Entities _(include if feature involves data)_

- **Documentation Baseline**: The expected minimum documentation surface per crate/module/public API.
- **Doc Gate Command**: The standardized lint/test command set used in local development and CI.
- **Exception Record**: Explicitly tracked and justified cases where strict doc rules are intentionally relaxed.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: 100% of workspace crates include crate-level documentation.
- **SC-002**: 100% of public `Result`-returning APIs pass `missing_errors_doc` checks.
- **SC-003**: 100% of panic-capable public APIs pass `missing_panics_doc` checks or are explicitly exempted.
- **SC-004**: `cargo test --doc --workspace` passes with no regressions.
- **SC-005**: A documented single source of truth exists for running documentation quality checks in CI and locally.

## Delivery Plan

1. **Phase 1 (Gate + Baseline)**: Define canonical documentation commands and enforce them in CI.
2. **Phase 2 (Crate Discoverability)**: Add/normalize crate-level `//!` docs across all workspace crates.
3. **Phase 3 (Public API Completion)**: Add `# Errors`/`# Panics` sections for all required public APIs.
4. **Phase 4 (Stabilization)**: Resolve remaining exceptions and confirm doc checks are green in CI.

## Out of Scope

- Rewriting large API surfaces solely for style.
- Third-party dependency docs.
- Architectural behavior changes unrelated to documentation quality.

## Assumptions

- Rustdoc + Clippy pedantic/nursery remain the primary doc quality enforcement tools.
- This effort targets workspace crates/modules, not external dependency documentation.
- Existing API behavior is unchanged; this is a documentation and policy-enforcement feature.
- CI has sufficient runtime budget to include doc-focused checks.
