# Project State

## Current status

Phases 0 and 1 are complete. The repository has its documented layout and a tested Rust workspace with a versioned configuration contract and CLI skeleton. No provider adapter, plugin implementation, Voxtype integration, or release tooling has been created.

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

### 2026-09-11 — Phase 1: Configuration contract and Rust workspace

- Created the Cargo workspace and the `engine` package for the `voxtype-personas` binary.
- Added a CLI skeleton for `version`, `process`, `profiles list`, `profiles set-active`, `providers test`, and `config validate`.
- Implemented `version` and `config validate`; commands planned for later phases return an explicit phase-specific unavailable message.
- Defined configuration schema version 1 with typed providers, prompts, profiles, active profile, and execution limits.
- Added all v1 provider kinds to the configuration model: Ollama, OpenAI, Mistral, Groq, OpenRouter, Anthropic, Gemini, and OpenAI-compatible endpoints.
- Defined the built-in Raw, Chat, Email, Technical, and Meeting / Notes profiles. Raw is the default active profile and has no provider, model, or prompt reference.
- Added XDG path discovery for configuration, state, engine installation, temporary downloads, backups, and logs. Managed directories use owner-only permissions on Unix.
- Added tests for default configuration, TOML round-tripping, duplicate TOML profile definitions, unsupported schema versions, invalid active profiles, invalid provider references without secret disclosure, XDG path resolution, and private directory permissions.
- Added `Cargo.lock` for reproducible dependency resolution.

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
- Ran `cargo fmt --check`; formatting is compliant.
- Ran `cargo test --workspace`; all 10 tests passed.
- Ran `voxtype-personas version`, `voxtype-personas config validate --defaults`, and the CLI help output successfully.
- Confirmed `voxtype-personas profiles list` exits with the expected explicit unavailable message until Phase 2.
- Ran `git diff --check` after Phase 1; it reported no whitespace errors.

## Next proposed step

Begin Phase 2 of `ROADMAP.md`: implement atomic configuration persistence, first-run defaults, schema migration scaffolding, and persisted profile selection. Provider networking and UI work remain out of scope.

## Commit and push status

- Phase 0 repository baseline committed with message `chore: establish repository baseline`.
- No push has been performed.
