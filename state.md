# Project State

## Current status

Phase 0, Repository baseline, is complete. The repository has its documented layout and contributor-facing project files. No application code, plugin implementation, Rust workspace, provider integration, or release tooling has been created.

## Completed work

### 2026-09-11 — Project working agreement and planning baseline

- Created the repository instruction file `AGENTS.md`.
- Recorded that documentation and code comments must be written in English.
- Recorded the incremental workflow: stop after each approved step and wait for the user to request continuation.
- Recorded that commits and pushes require explicit user approval.
- Read `docs/VoxTypePersonas.md` and `docs/specs.md` in full.
- Created `ROADMAP.md` with the detailed v1 delivery sequence.
- Created this state file.
- Updated `AGENTS.md` to make `ROADMAP.md` the mandatory step-by-step development sequence and `state.md` the required continuity record for future sessions.

### 2026-09-11 — Phase 0: Repository baseline

- Inspected the Git repository after the user's initial commit; the working tree was clean before this phase.
- Added the MIT `LICENSE`.
- Added an English `README.md` describing scope, non-goals, privacy principles, the planned layout, and the development plan.
- Added `CONTRIBUTING.md` with English documentation, incremental-workflow, and security conventions.
- Added `.gitignore` rules for build outputs, local environment files, secrets, temporary artifacts, and editor files.
- Created empty tracked directories for the future Rust engine, Omarchy plugin, test fixtures, helper scripts, and CI workflows.
- Set the MIT license copyright holder to Stéphane Depierrepont.
- Did not add application code, dependencies, provider integrations, Omarchy plugin content, or Voxtype configuration changes.

### 2026-09-11 — Deferred prompt catalog decision

- Recorded a post-v1 idea for a versioned prompt catalog stored in this repository.
- The catalog is intended to let users share curated reusable personas while keeping local providers, selected models, configuration, and secrets outside the catalog.
- This does not change the v1 scope: importing/exporting prompts and community sharing remain deferred.
- Committed this recorded decision with message `docs: record shared prompt catalog idea`.

## Decisions currently in force

- Target: Omarchy on Linux only.
- Plugin ID: `io.github.toorop.voxtype-personas`.
- Engine: Rust binary named `voxtype-personas`.
- Supported release architectures: Linux `x86_64` and `aarch64`.
- Secret storage: mandatory Secret Service/keyring; no file fallback.
- Default built-in profile: Raw.
- No dictated text, prompts, secrets, or telemetry are persisted or logged by default.
- The project must not alter the existing Hyprland dictation binding or PipeWire mute relay.

## Validation performed

- Confirmed that the repository instructions are stored in root-level `AGENTS.md`.
- Confirmed the documentation consists of `docs/VoxTypePersonas.md` and `docs/specs.md`.
- Ran `git diff --check` after Phase 0; it reported no whitespace errors.
- No implementation or runtime validation has been performed because the project is intentionally still in planning.

## Next proposed step

Begin Phase 1 of `ROADMAP.md`: create the Rust workspace and define the versioned configuration and CLI contracts, without provider networking or UI work.

## Commit and push status

- Phase 0 repository baseline committed with message `chore: establish repository baseline`.
- No push has been performed.
