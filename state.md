# Project State

## Current status

Phases 0 through 8 are complete. The engine supports local Ollama, OpenAI-compatible remote providers, and native Anthropic/Gemini adapters with Secret Service-backed keys. No plugin implementation, Voxtype integration, or release tooling has been created.

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
- Committed the Phase 1 implementation as `1cad1f7` (`feat: add configuration contract and CLI skeleton`).

### 2026-09-11 — Phase 2: Atomic persistence and configuration migration

- Added `ConfigStore`, the engine configuration storage layer.
- Implemented first-use creation of the default configuration at the XDG configuration path.
- Implemented atomic configuration writes using a same-directory temporary file, synchronization, owner-only file permissions on Unix, and atomic replacement.
- Implemented schema detection and sequential migration dispatch. The initial v0-to-v1 migration creates a non-overwriting backup before it writes the migrated configuration.
- Implemented safe handling of malformed, inaccessible, or unsupported configuration: errors are returned and the existing source file is not replaced.
- Implemented persisted `profiles list` and `profiles set-active <id>` commands.
- Ensured an unknown profile selection leaves a valid persisted configuration unchanged.
- Kept Raw mandatory through configuration validation; no profile deletion feature exists at this stage.
- Added storage tests for first-use defaults, persistence, invalid selection preservation, malformed configuration preservation, migration backup creation, and Unix file permissions.
- Committed the Phase 2 implementation as `ef5c686` (`feat: persist configuration and active profiles`).

### 2026-09-11 — Phase 3: Raw processing path and CLI behavioral contract

- Implemented `voxtype-personas process` and `process --profile <id>`.
- The command reads standard input as bytes so Raw can preserve the exact input without altering whitespace, line endings, or other bytes.
- The profile is selected exactly once at the start of an invocation from either `--profile` or the persisted active profile.
- Raw writes only the original input to standard output, produces no diagnostics, and returns success.
- Empty input returns unchanged without a provider path.
- Configuration and unavailable-profile failures preserve the raw input on standard output, use a non-zero status, and write only a non-sensitive diagnostic to standard error. The recoverable provider-failure policy is intentionally deferred to Phase 4.
- Added binary integration tests for byte-preserving Raw output, empty input, explicit profile selection without changing persistent selection, and clean fallback output without dictated text in diagnostics.
- Committed and pushed the Phase 3 implementation as `9f36468` (`feat: add raw processing path`).

### 2026-09-11 — Phase 4: Fallback policy and response validation

- Added a provider-independent processing outcome layer that centralizes processed output and raw-text fallback decisions.
- Defined recoverable provider failures: unavailable provider, timeout, network failure, authentication failure, HTTP status failure, response parsing failure, and invalid response.
- Recoverable provider failures now preserve raw input, return success, and emit only a generic non-sensitive diagnostic on standard error.
- Added per-profile output policy fields. By default, empty output, NUL bytes, obvious preambles, and Markdown framing are rejected.
- Added validation tests for successful output, every recoverable failure category, invalid responses, and profile policies that explicitly allow Markdown or preambles.
- Updated the process integration behavior so an unavailable non-Raw profile returns raw output with success and no dictated text in diagnostics.

### 2026-09-11 — Phase 5: Secret Service abstraction

- Selected the Rust `secret-service` client and its blocking D-Bus API with encrypted-session support as the concrete Linux Secret Service adapter for provider configuration work.
- Added the `SecretStore` interface with read, write, verify, replace, and delete operations.
- Defined opaque provider secret references as `org.voxtype-personas/provider/<provider-id>`.
- Added explicit non-sensitive errors for invalid references, unavailable or locked Secret Service, missing secrets, and failed operations.
- Added an in-memory `SecretStore` test double. It is limited to tests and is not a file fallback or a production key store.
- Added tests covering the complete secret lifecycle and secret-free error text.

### 2026-09-11 — Phase 6: Provider interface and Ollama adapter (in progress)

- Added the provider request contract and adapter interface for model discovery, connectivity testing, and text processing.
- Added the Ollama protocol adapter with default endpoint `http://127.0.0.1:11434`, model discovery through `/api/tags`, and chat processing through `/api/chat`.
- Added a mock HTTP transport test suite covering installed-model discovery, unavailable Ollama, separate system and user messages, output limits, authentication failures, and HTTP failures.
- Added `serde_json` for Ollama JSON request and response handling.
- No Ollama installation or local service is required for the tests. The production HTTP transport and CLI processing integration remain outstanding in this phase.
- Added the production HTTP transport with explicit timeout and unavailable-service mapping.
- Integrated configured Ollama profiles into `process`, keeping system prompts and dictated text in separate messages.
- Implemented `providers test <id>` using a minimal dedicated request rather than a dictated transcription.
- Completed Phase 6 without installing or starting Ollama locally; unavailable Ollama continues to trigger the existing raw-text fallback.

### 2026-09-11 — Phase 7: OpenAI-compatible providers (in progress)

- Added a shared Chat Completions-compatible adapter for OpenAI, Mistral, Groq, OpenRouter, and named OpenAI-compatible endpoints.
- Added fixed endpoint metadata for OpenAI, Mistral, Groq, and OpenRouter.
- Extended the HTTP transport with explicit request headers and implemented Bearer authorization support in the compatible adapter.
- Added model-list parsing and filtering for image, audio, transcription, TTS, embedding, and moderation-only model identifiers.
- Added mock transport tests for authorization headers, separate system/user messages, maximum output tokens, endpoint metadata, model filtering, and HTTP error mapping.
- The adapter does not yet read API keys from Secret Service during normal CLI processing; that integration remains required before remote providers can be configured and used.
- Added the concrete Linux Secret Service D-Bus adapter using the `secret-service` crate and its Tokio/RustCrypto runtime feature.
- Connected remote provider execution to opaque Secret Service references only; API keys are never read from TOML or emitted in diagnostics.
- Added endpoint, timeout, remote-secret-reference, and explicit-model validation for provider configuration.
- Completed Phase 7. The fallback policy handles unavailable keyring access or provider failure by returning the raw transcription.

### 2026-09-11 — Phase 8: Native Anthropic and Gemini adapters

- Added the native Anthropic Messages adapter using the top-level `system` field and a separate `user` message.
- Added the native Gemini GenerateContent adapter using `system_instruction` and separate `contents`.
- Both adapters use the existing Secret Service key path, HTTP error mapping, response validation, and raw-text fallback.
- Added mock transport tests for Anthropic and Gemini request shapes and response parsing.

### 2026-09-11 — Profile catalog and readiness decision

- Replaced the planned ready-to-use built-in Chat, Email, Technical, and Meeting / Notes profiles with one generic Example draft profile.
- Confirmed that Raw is the sole active profile on first use.
- Defined Draft, Ready, and Active profile states. Only Ready profiles may be activated.
- Added the versioned `profiles/` catalog with Markdown plus YAML-front-matter documentation and `profiles/example.md`.
- Confirmed that portable profiles contain public configuration only: no API keys, dictated text, or local Secret Service references.
- Moved portable profile import/export and pull-request contribution support into the v1 profile-catalog scope.
- Confirmed that portable profile imports must be fully validated and rejected atomically when invalid, unsafe, or non-conformant.

### 2026-09-11 — Phase 9: default profile reset (in progress)

- Replaced the previous built-in Chat, Email, Technical, and Meeting / Notes defaults with the generic Example draft profile.
- The generated configuration now contains only Raw and Example; Raw remains active on first use.
- The Example profile remains intentionally incomplete because it has no provider or model assignment.
- Activation gating, explicit Draft/Ready/Active status reporting, and validated import/export remain outstanding Phase 9 work.
- Ran `cargo fmt --check`, `cargo test --workspace`, and `git diff --check`; formatting and diff checks are clean, and all 35 tests pass.

### 2026-09-12 — Specification alignment

- Rewrote `docs/specs.md` in English to comply with the repository documentation convention.
- Removed superseded references to built-in Chat, Email, Technical, and Meeting / Notes profiles and to profile import/export being outside v1.
- Made the v1 contract explicit: Raw is the only active default, Example is a portable Draft, only Ready profiles can be activated, and portable Markdown/YAML profile import/export must be validated atomically.
- Updated the quick-selection journey, CLI contract, data model, privacy/export rules, and acceptance criteria to match the Phase 9 catalog and readiness decision.
- No engine or plugin code was changed.

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
- Ran `cargo fmt --check` after Phase 2; formatting is compliant.
- Ran `cargo test --workspace` after Phase 2; all 16 tests passed.
- Verified the CLI in temporary XDG directories: first-use creation, profile listing, selecting Chat, persisted re-listing, validation, and configuration file mode `0600` all succeeded.
- Ran `git diff --check` after Phase 2; it reported no whitespace errors.
- Ran `cargo fmt --check` after Phase 3; formatting is compliant.
- Ran `cargo test --workspace` after Phase 3; all 20 tests passed.
- Verified `printf 'exact input\\n' | voxtype-personas process` in temporary XDG directories; the output matched byte-for-byte.
- Ran `git diff --check` after Phase 3; it reported no whitespace errors.
- Ran `cargo fmt --check` after Phase 4; formatting is compliant.
- Ran `cargo test --workspace` after Phase 4; all 24 tests passed.
- Verified a non-Raw unavailable profile in temporary XDG directories: raw output was preserved, the process exited successfully, and standard error contained no dictated text.
- Ran `git diff --check` after Phase 4; it reported no whitespace errors.
- Ran `cargo fmt --check` after Phase 5; formatting is compliant.
- Ran `cargo test --workspace` after Phase 5; all 26 tests passed.
- Ran `git diff --check` after Phase 5; it reported no whitespace errors.
- Ran `cargo fmt` and `cargo test --workspace` during Phase 6; all 30 tests passed.
- Ran `cargo fmt --check`, `cargo test --workspace`, and `git diff --check` after completing Phase 6; all 30 tests passed and formatting/diff checks are clean.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check` during Phase 7; all 33 tests passed and formatting/diff checks are clean.
- Ran `cargo fmt --check`, `cargo test --workspace`, and `git diff --check` after completing Phase 7; all 33 tests passed and formatting/diff checks are clean.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check` after Phase 8; all 35 tests passed and formatting/diff checks are clean.

## Next proposed step

Complete the next small Phase 9 step: enforce activation eligibility for draft profiles and expose clear Draft/Ready/Active status through the engine CLI. Do not begin until the user explicitly approves it.

## Commit and push status

- Phase 0 repository baseline committed with message `chore: establish repository baseline`.
- Phase 3 was pushed to `origin/main` as part of commit `9f36468`.
