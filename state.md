# Project State

## Current status

Phases 0 through 9 are complete and have been pushed to `origin/main`. Phase 10 is in progress: the current Omarchy plugin contract has been reviewed, but no plugin source has been created. The engine supports local Ollama, OpenAI-compatible remote providers, and native Anthropic/Gemini adapters with Secret Service-backed keys. No Voxtype integration or release tooling has been created.

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

### 2026-09-12 — Phase 9: profile activation eligibility and CLI status (in progress)

- Added the engine-level `Draft`, `Ready`, and `Active` profile states.
- Defined Raw as always Ready (and Active when selected); the shipped Example profile is reported as Draft because it lacks a configured provider and model.
- Added readiness checks for a referenced non-Raw prompt and an explicitly selected model offered by its configured provider.
- Updated `profiles list` to print the profile ID, display name, and state, with `*` marking the Active profile.
- Prevented `profiles set-active <id>` from activating a Draft profile. The command returns a clear non-sensitive error and leaves the existing configuration byte-for-byte unchanged.
- Added unit and integration coverage for Draft, Ready, and Active states, CLI list output, and rejected Draft activation.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all 35 tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9: unready active-profile fallback (in progress)

- Kept an existing configuration loadable when its persisted active profile is now Draft, so it can be recovered safely rather than becoming an unusable configuration error.
- Updated `process` to resolve a selected Draft profile to byte-preserving Raw output with a zero exit status and a fixed non-sensitive diagnostic.
- Added an integration test that simulates a persisted Draft active profile and proves raw output preservation, successful exit status, and the absence of dictated text from diagnostics.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all 35 unit tests and 7 integration tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9: portable profile parsing and safety validation (in progress)

- Added a typed portable-profile catalog module for Markdown files with YAML front matter.
- Defined the v1 portable Draft metadata contract: schema version, identifier, display name, provider metadata, compatibility notes, limits, output policy, and Markdown-body system prompt.
- Added strict front-matter parsing, schema/version and typed-field checks, identifier/limit/endpoint validation, and non-sensitive rejection of prohibited secret-reference and authorization-like content.
- Added tests for valid parsing, malformed or missing front matter, unsupported schemas, invalid metadata, secret-like content, and the shipped `profiles/example.md` fixture.
- Added the `serde_yaml` dependency and recorded it in `Cargo.lock`.
- This sub-step deliberately does not import, export, or persist a portable profile yet.
- Ran `cargo fmt --check`, `cargo test --workspace`, and `git diff --check`; all 40 unit tests and 7 integration tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9: portable catalog set validation and fixtures (in progress)

- Added validation for a set of parsed portable profiles, rejecting duplicate profile IDs before any future import operation can mutate local configuration.
- Added reusable catalog fixtures for duplicate IDs, a prohibited secret reference, and missing YAML front matter.
- Added tests proving duplicate IDs and unsafe or malformed fixtures are rejected with diagnostics that do not disclose the fixture's private reference.
- This sub-step still performs no import, export, or configuration write.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all 42 unit tests and 7 integration tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9: read-only portable-profile CLI validation (in progress)

- Added `voxtype-personas profiles validate <file>...` as a read-only CLI surface for one or more portable catalog files.
- The command reuses the strict parser and set-level duplicate-ID checks, reports only safe diagnostics, and does not discover, create, read, or write the local XDG configuration.
- Added integration coverage for successful validation of the shipped Example profile and rejection of duplicate IDs, proving that neither result creates `config.toml`.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all 42 unit tests and 9 integration tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9: atomic Draft-profile import (in progress)

- Added `voxtype-personas profiles import <file>...`.
- The command reads and validates every requested portable profile, including set-level duplicate-ID checks, before discovering or writing local configuration.
- A successful import atomically adds each profile's name, system prompt, limits, and output policy as a local Draft; it deliberately leaves provider and model unconfigured, so imported content cannot be activated before local provider setup.
- Reserved `raw` as an invalid portable profile ID, preventing any catalog file from replacing the mandatory Raw profile.
- Existing local profile or prompt ID collisions fail without replacing the prior configuration.
- Added storage and CLI integration tests for successful Draft import, unsafe-file rejection without configuration creation, preserved active Raw selection, and failed-import preservation.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all 44 unit tests and 11 integration tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9: safe portable-profile export (in progress)

- Added `voxtype-personas profiles export <id> <file>`.
- The command serializes an exportable local non-Raw profile as a portable Draft with YAML front matter and a Markdown-body system prompt.
- Export includes only public provider kind, endpoint, and selected-model metadata; it never serializes Secret Service references or key material.
- Export refuses the mandatory Raw profile and uses create-new file semantics, so it never overwrites an existing destination.
- Added catalog and CLI integration coverage for safe serialization, parser round-trip, output-file creation, and overwrite refusal.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all 45 unit tests and 12 integration tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9: portable-profile CLI documentation (in progress)

- Updated the root README, catalog README, and functional specification with `profiles validate`, `profiles import`, and `profiles export`.
- Documented the read-only behavior of validation, all-files-before-write atomic Draft import behavior, safe export contents, protected Raw profile, and export refusal to overwrite a destination file.
- Corrected the README implementation status to Phase 9 in progress and removed an obsolete statement that Secret Service integration was still pending.
- Ran `git diff --check`; documentation changes are whitespace-clean.

### 2026-09-12 — Phase 9: remote Secret Service reference eligibility (in progress)

- Made every remote provider kind require an opaque, syntactically valid VoxTypePersonas Secret Service reference. This now includes Anthropic and Gemini as well as OpenAI-compatible providers.
- Updated profile readiness so a remote profile with an absent or invalid secret reference remains Draft even when it has a prompt, provider, and selected model.
- Kept Ollama as the only provider kind that does not require a Secret Service reference.
- Added coverage for every remote provider kind and for the Draft-to-Ready transition when a valid opaque reference is supplied. These checks validate the reference only; verification that the referenced key currently exists in Secret Service remains the next small step.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all 47 unit tests and 12 integration tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9: remote key verification on activation (in progress)

- Added Secret Service item verification to `profiles set-active` before a Ready remote profile can become Active.
- Kept local Ollama and Raw activation independent from Secret Service.
- Added an injectable secret-store path for deterministic storage tests while production activation uses the real Secret Service adapter.
- A missing, locked, unavailable, or otherwise unverifiable provider key rejects activation without writing configuration; diagnostics do not reveal the secret reference or key material.
- Added success and failure tests proving verification behavior and byte-for-byte configuration preservation on failed activation.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all 49 unit tests and 12 integration tests passed and formatting/diff checks are clean.

### 2026-09-12 — Phase 9 completion review (awaiting validation)

- Reviewed every Phase 9 roadmap item and its acceptance criteria against the implementation, catalog documentation, and functional specification.
- Confirmed the shipped configuration contains Raw and Example only; Example is a non-activatable Draft, and Raw remains the active default and fallback.
- Confirmed portable Markdown/YAML parsing, safe validation, duplicate-ID checks, atomic Draft import, safe create-new export, and Secret Service verification on remote activation are covered by tests.
- Rewrote `docs/VoxTypePersonas.md` in English and removed its superseded profile/import-export decisions, so project documentation now follows the repository English-only convention.
- Fixed two Clippy `collapsible_if` warnings discovered during the review.
- Ran `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`, and `git diff --check`; all checks passed, with 49 unit tests and 12 integration tests.
- The Phase 9 review was committed as `5c21a16` (`chore: complete phase 9 review`) and the complete Phase 9 history was pushed to `origin/main`.

### 2026-09-12 — Phase 10: Omarchy plugin contract review

- Reviewed the current stable Omarchy development and publishing guides before creating plugin files.
- Confirmed the fixed contract for this project: `kinds` must contain only `bar-widget`; `entryPoints.barWidget` must reference `BarWidget.qml`; and `Panel.qml` must be loaded internally by the widget rather than declared as a second plugin kind.
- Confirmed that plugins share the long-running shell process, run unsandboxed with user permissions, and must never start a second Quickshell process.
- Recorded the required lifecycle surface for the entry point: forward `opened`, `open()`, `close()`, `toggle()`, and `closeForPopoutSwitch()` to the loaded panel; inject the bar, anchor button, and host widget into that panel.
- Recorded the validation commands for the next implementation step: `omarchy plugin validate <plugin-dir>` and `qmllint -I "$OMARCHY_PATH/shell" <plugin-dir>/BarWidget.qml <plugin-dir>/Panel.qml`.
- No plugin manifest, QML source, system configuration, or Omarchy installation was changed in this review step.

### 2026-09-12 — Phase 10: bar-widget manifest (in progress)

- Added the repository-root `manifest.json` for plugin ID `io.github.toorop.voxtype-personas`.
- Declared only `kinds: ["bar-widget"]`, with `entryPoints.barWidget` set to `BarWidget.qml`; no standalone panel kind was declared.
- Added non-invasive bar-widget metadata: a single instance in the right bar section, a user-facing name, MIT license, author, and concise description.
- Validated JSON syntax with `jq empty manifest.json`.
- Ran `omarchy plugin validate .`; it correctly reached the manifest entry-point check and reported that `BarWidget.qml` does not exist yet. This expected failure will be resolved by the next approved sub-step; no system configuration was changed.

### 2026-09-12 — Phase 10: BarWidget entry point (in progress)

- Added `BarWidget.qml` as the manifest entry point, using the current Omarchy `BarWidget` and `WidgetButton` contract.
- Implemented the required forwarding surface for the nested panel: `opened`, `popoutSwitchClosing`, `open()`, `close()`, `toggle()`, `closeForPopoutSwitch()`, and bar/anchor/host injection when the panel loads.
- Added a static `Raw` label and a left-click toggle. Dynamic engine state and profile selection are intentionally deferred to later approved sub-steps.
- Kept `Panel.qml` internal through a `Loader`; it is not declared as a second plugin kind.
- Ran `omarchy plugin validate .` successfully after adding the entry point. `qmllint` is installed as `/usr/lib/qt6/bin/qmllint` through the existing `qt6-declarative` package, but is not on `PATH`.
- Ran `/usr/lib/qt6/bin/qmllint -I /usr/share/omarchy/shell BarWidget.qml`. The installed linter did not resolve Omarchy's nonstandard `qs.Ui` module layout and therefore emitted unresolved-type warnings; this is an environment/tooling limitation to revisit before final plugin validation, not a reason to install a duplicate package.
- Ran `git diff --check`; no whitespace errors were found.

### 2026-09-12 — Phase 10: widget-owned anchored panel (in progress)

- Added `Panel.qml` as the sole panel implementation loaded internally by `BarWidget.qml`.
- Implemented the documented Omarchy panel lifecycle, anchor/host ownership, panel switching, and Escape-to-close behavior through `PanelKeyCatcher`.
- Added static rows that represent the current default state: Raw as Ready and Active, and Example as a Draft that needs configuration. The rows do not invoke the engine yet.
- Ran `omarchy plugin validate .` successfully and `git diff --check` with no whitespace errors.
- Ran the installed `qmllint` binary against both QML files. It returned success with no syntax error, but emitted unresolved-import/type warnings because it cannot resolve Omarchy's `qs.Ui` and `qs.Commons` module layout from this packaged shell path. This environment limitation remains recorded for final plugin validation.

### 2026-09-12 — Phase 10: read-only engine state in the widget (in progress)

- Added a direct Quickshell `Process` invocation of `voxtype-personas profiles list` to `BarWidget.qml`; no shell wrapper is used.
- Added parsing of the active-profile row and dynamic bar-button label/tooltip updates from the engine's tab-separated output.
- Added a non-sensitive `Unavailable` state when the engine command fails, configuration cannot be resolved, or no active row is returned. The widget does not display command output or dictated text.
- Kept this integration read-only. It does not select a profile, create configuration deliberately, or alter Voxtype or Omarchy system configuration.
- Ran `omarchy plugin validate .` successfully and `git diff --check` with no whitespace errors. The installed `qmllint` returned success but still emitted the previously recorded unresolved `qs.*` module warnings from the packaged-shell layout.

### 2026-09-12 — Phase 10: dynamic selector and persisted selection (in progress)

- Extended `BarWidget.qml` to parse every row from `voxtype-personas profiles list` and pass profile entries plus non-sensitive error state to its loaded panel.
- Replaced the panel's static rows with a dynamic selector. Draft entries remain visible with a configuration hint but are disabled; Ready and Active entries are selectable.
- Added a direct Quickshell `Process` call to `voxtype-personas profiles set-active <id>` for selection. On success, the panel closes and refreshes the widget state; on failure, it remains open and shows a generic non-sensitive error.
- No shell wrapper, provider request, secret value, dictated text, Voxtype change, or Omarchy system configuration is involved.
- Ran `omarchy plugin validate .` successfully and `git diff --check` with no whitespace errors. The installed `qmllint` returned success but continued to report the recorded unresolved `qs.*` module warnings; the local linter's delegate-scope warnings were reduced by qualifying model data.

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

Complete the next small Phase 10 step: add an explicit refresh action and a clear empty/error state to the panel, then review the Phase 10 foundation against its acceptance criteria before running it in an installed plugin directory. Do not begin until the user explicitly approves it.

## Commit and push status

- Phase 0 repository baseline committed with message `chore: establish repository baseline`.
- Phase 3 was pushed to `origin/main` as part of commit `9f36468`.
