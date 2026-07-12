# Tasks: Rust Documentation Availability Baseline

**Input**: `/specs/001-improve-rust-crate/spec.md`

**Prerequisites**: spec.md (complete), plan.md (optional for this first pass)

**Organization**: Tasks are grouped by user story and ordered for implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable (different files, low coupling)
- **[Story]**: US1, US2, US3 from the spec

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the documentation quality gate and shared command guidance.

- [x] T001 [US1] Add a "Documentation Quality" section to `docs/CONTRIBUTING.md` defining canonical commands (`cargo clippy --all-targets --all-features`, `cargo test --doc --workspace`).
- [x] T002 [US1] Add/adjust a CI workflow for doc quality checks in `.github/workflows/` (new workflow or update existing workflow) to run the canonical documentation gate commands.
- [x] T003 [US1] Ensure CI output clearly identifies doc lint failures (`missing_errors_doc`, `missing_panics_doc`) for contributors.

**Checkpoint**: Documentation gate is defined and wired into CI.

---

## Phase 2: User Story 1 - Enforce doc availability gates (Priority: P1)

**Goal**: Make doc quality regressions visible and enforceable.

**Independent Test**: CI and local runs fail on missing required doc sections and pass when fixed.

### Implementation Tasks

- [x] T004 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/instance.rs` public `Result` APIs flagged by clippy.
- [x] T005 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/install.rs` public `Result` API.
- [x] T006 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/launcher.rs` public `Result` API.
- [x] T007 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/startup.rs` public `Result` API.
- [x] T008 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/steamcmd.rs` public `Result` API.
- [x] T009 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/update.rs` public `Result` APIs.
- [x] T010 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/proton/mod.rs` public `Result` APIs.
- [x] T011 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/proton/releases.rs` public `Result` APIs.
- [x] T012 [P] [US1] Add `# Errors` docs to `libs/gsm-instance/src/proton/types.rs` public `Result` API.
- [x] T013 [P] [US1] Add `# Errors` docs to `libs/gsm-shared/src/fetch_public_ip_address.rs` public `Result` APIs.
- [x] T014 [P] [US1] Add `# Errors` docs to `libs/gsm-shared/src/parse_truthy.rs` public `Result` API.
- [x] T015 [P] [US1] Add `# Errors` docs to `libs/gsm-notifications/src/notifications.rs` public `Result` API.
- [x] T016 [P] [US1] Add `# Errors` docs to `libs/gsm-notifications/src/lib.rs` public `Result` APIs.
- [x] T017 [P] [US1] Add `# Errors` docs to `libs/gsm-mod-manager/src/managed_mod.rs` public `Result` APIs.
- [x] T018 [P] [US1] Add `# Errors` docs to `libs/gsm-serde/src/serde_ini.rs` public `Result` APIs.
- [x] T019 [US1] Add `# Panics` docs for `libs/ini-derive/src/lib.rs::ini_serialize`.
- [x] T020 [US1] Add `# Panics` docs for panic-capable public API in `libs/gsm-backup/src/lib.rs`.

**Checkpoint**: `missing_errors_doc` / `missing_panics_doc` warnings are resolved for audited hotspots.

---

## Phase 3: User Story 2 - Improve crate-level discoverability (Priority: P2)

**Goal**: Ensure every crate has a usable crate-level overview.

**Independent Test**: All crate roots expose `//!` docs in generated rustdoc.

### Implementation Tasks

- [ ] T021 [P] [US2] Add crate-level `//!` docs to `libs/gsm-mod-manager/src/lib.rs`.
- [ ] T022 [P] [US2] Add crate-level `//!` docs to `libs/gsm-monitor/src/lib.rs`.
- [ ] T023 [P] [US2] Add crate-level `//!` docs to `libs/gsm-serde/src/lib.rs`.
- [ ] T024 [P] [US2] Add crate-level `//!` docs to `libs/gsm-shared/src/lib.rs`.
- [ ] T025 [P] [US2] Add crate-level `//!` docs to `libs/ini-derive/src/lib.rs`.
- [ ] T026 [P] [US2] Add module-level/cargo entry docs to `apps/enshrouded/src/main.rs`.
- [ ] T027 [P] [US2] Add module-level/cargo entry docs to `apps/gsm-cli/src/main.rs`.
- [ ] T028 [P] [US2] Add module-level/cargo entry docs to `apps/palworld/src/main.rs`.
- [ ] T029 [P] [US2] Add module-level/cargo entry docs to `tools/env-parser/src/main.rs`.

**Checkpoint**: Crate/application entrypoints have discoverable top-level documentation.

---

## Phase 4: User Story 3 - Standardize public API docs (Priority: P3)

**Goal**: Improve consistency and usability of public API docs beyond minimal lint compliance.

**Independent Test**: Sampled high-traffic APIs include purpose + behavior + failure semantics.

### Implementation Tasks

- [ ] T030 [P] [US3] Normalize public type/function docs in `libs/gsm-instance/src/lib.rs` exports to include concise behavior summaries.
- [ ] T031 [P] [US3] Normalize API docs in `libs/gsm-monitor/src/rules.rs` public structs/functions (intent + usage expectations).
- [ ] T032 [P] [US3] Normalize API docs in `libs/gsm-shared/src/lib.rs` public helpers and utilities.
- [ ] T033 [P] [US3] Normalize API docs in `libs/gsm-mod-manager/src/errors.rs` and `libs/gsm-mod-manager/src/managed_mod.rs`.
- [ ] T034 [P] [US3] Normalize API docs in `apps/gsm-cli/src/environment.rs` for public environment contract helpers.

**Checkpoint**: Public API docs are consistently readable and actionable.

---

## Phase 5: Validation & Polish

**Purpose**: Verify documentation quality is stable and repeatable.

- [x] T035 [US1] Run `cargo clippy --all-targets --all-features` and address remaining doc-lint regressions.
- [x] T036 [US1] Run `cargo test --doc --workspace` and resolve any doctest regressions.
- [ ] T037 [US1] Update `specs/001-improve-rust-crate/spec.md` status and success criteria outcomes after implementation.

---

## Dependencies & Execution Order

1. Phase 1 first (T001-T003).
2. US1 gating fixes (T004-T020) before broader API polish.
3. US2 crate-level docs (T021-T029) can run in parallel by crate.
4. US3 consistency pass (T030-T034) after lint blockers are cleared.
5. Validation last (T035-T037).

## Parallel Opportunities

- T004-T018 are mostly independent by file and crate.
- T021-T029 are independent crate-entrypoint docs.
- T030-T034 can be split by crate owner/surface.
