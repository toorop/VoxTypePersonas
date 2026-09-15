# Project State

## Current status

Phases 0 through 11 are complete. The Omarchy bar widget, anchored selector, verified engine installation, rollback protection, and selected theme-tinted persona icon have been visually reviewed in a live Omarchy session. The user has since removed the user-owned Omarchy development-plugin directory; reinstall the plugin from this repository before the next live UI test. Phase 12 — extended settings panel — is next. The engine supports local Ollama, OpenAI-compatible remote providers, and native Anthropic/Gemini adapters with Secret Service-backed keys. No Voxtype integration or release tooling has been created.

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

### 2026-09-12 — Phase 10: selector refresh and empty/error states (in progress)

- Added an explicit Refresh action to the panel. It re-runs the widget's read-only profile-list command and clears the previous action error.
- Added distinct panel states for unavailable configuration/engine, an empty profile list, and a failed profile activation. All messages are fixed and non-sensitive.
- Ran `omarchy plugin validate .` successfully and `git diff --check` with no whitespace errors. The installed `qmllint` returned success with the recorded unresolved `qs.*` module warnings from the packaged-shell layout.

### 2026-09-12 — Phase 10 foundation review (awaiting visual test)

- Rechecked JSON syntax, `omarchy plugin validate .`, and `git diff --check`; all passed.
- Confirmed the manifest declares only `bar-widget`; both QML files use the fixed plugin module ID; Panel remains loaded only by BarWidget; Escape, Refresh, unavailable, empty-list, and activation-failure states are present.
- Confirmed the widget uses direct command arrays for `profiles list` and `profiles set-active`, without a shell wrapper.
- Confirmed that the target user-owned development-plugin directory `~/.config/omarchy/plugins/io.github.toorop.voxtype-personas` did not exist before installation, so the test copy could not overwrite a known local plugin folder.
- With explicit user authorization, installed the three plugin source files (`manifest.json`, `BarWidget.qml`, and `Panel.qml`) as a temporary development copy in that directory. No Voxtype configuration, release binary, or existing plugin was changed.
- Requested a plugin rescan and enablement. Enablement reported that the plugin was unknown; subsequent checks confirmed that no `omarchy-shell`, `quickshell`, or `Hyprland` process is available to this command session. `omarchy plugin list --json` and `omarchy plugin validate` consequently report that the Omarchy shell is not running.
- The visual test is blocked only until commands can be run from an active Omarchy graphical session. The development copy remains ready for that test.

### 2026-09-12 — Phase 10: first visual test

- The user rescanned and enabled the temporary development copy from an active Omarchy session.
- Visual inspection confirmed that the bar widget and its anchored panel load successfully. The panel’s title, refresh affordance, explanatory copy, divider, and empty-profile state render as intended with the active Omarchy theme.
- The bar label remains `Loading…` and the panel has no profiles because the release engine binary is not installed on the user PATH yet. The widget currently invokes the release command name directly; a repository-local `cargo` build is deliberately not an implicit runtime dependency.
- Treat the persistent loading state as a widget robustness defect: when the command cannot be started, the visible state should settle on the existing non-sensitive `Unavailable` state rather than retaining the initial loading label. Address this in the next approved Phase 10 sub-step.

### 2026-09-12 — Phase 10: unavailable-engine fallback (awaiting visual confirmation)

- Changed the initial bar state from `Loading…` to the existing non-sensitive `Unavailable` fallback. This covers a Quickshell process that cannot start because the release engine command is not present; such a failure does not necessarily emit `Process.exited`.
- A successful `profiles list` response still replaces the fallback with the active profile label; normal nonzero exits retain the same fallback.
- Ran `omarchy plugin validate .`, `qmllint`, and `git diff --check`. Manifest validation and whitespace checks passed; `qmllint` returned success with the already-recorded unresolved packaged Omarchy module warnings.
- Copied the updated `BarWidget.qml` to the active temporary development-plugin directory. Omarchy’s local plugin watcher should hot-reload the widget without disabling it or manually rescanning.
- The automatic reload/rescan did not replace the already-mounted bar-widget instance. Restarting the Omarchy shell rebuilt it and confirmed that the new `Unavailable` fallback works as intended.
- Visual review found a UX issue in the unavailable state: the panel still leads with profile-selection copy even though the engine/configuration makes selection impossible. Replace this with a dedicated onboarding state that explains that the engine must be installed before profiles can be selected.

### 2026-09-12 — Phase 10: engine-install onboarding shell (awaiting visual confirmation)

- Replaced the visible `Unavailable` bar text with Omarchy’s microphone glyph (`󰍬`). The normal bar foreground is intentionally retained; engine activity colouring is deferred because no engine activity-status contract exists yet.
- Added an explicit `engineAvailable` state to separate an absent/unstartable engine from a working engine with profiles to display.
- When the engine is unavailable, the panel now displays only an `Install engine` button. The button is deliberately inert: Phase 11 will add its explicit confirmation and verified release-installation flow.
- When the engine is available, the existing Refresh, profile-selection guidance, error/empty states, and selector remain available.
- Ran `omarchy plugin validate .`, `qmllint`, and `git diff --check`. Manifest validation and whitespace checks passed; `qmllint` returned success with the recorded unresolved packaged Omarchy module warnings.
- Copied the updated `BarWidget.qml` and `Panel.qml` to the active temporary development-plugin directory. Restart the Omarchy shell to force a reliable reconstruction of the mounted widget for visual review.

### 2026-09-12 — Phase 10: themed unavailable state refinement (awaiting visual confirmation)

- Set the microphone icon to the Omarchy `WidgetButton` active state while the engine is unavailable. The icon therefore uses the theme’s `urgent` colour (red in the tested theme), without hard-coding a colour and while retaining automatic theme adaptation.
- Replaced the hand-drawn install control with Omarchy’s native `Button` component. The previous rectangle opacity also attenuated its child text and made it nearly illegible; the native button gives the text, hover, fill, and border theme-controlled contrast.
- Ran `omarchy plugin validate .`, `qmllint`, and `git diff --check`. Manifest validation and whitespace checks passed; `qmllint` returned success with the previously recorded packaged-module warnings.
- Copied the updated QML files to the active temporary development-plugin directory. Restart the Omarchy shell to force a reliable visual rebuild.

### 2026-09-12 — Phase 10: icon direction decision

- The microphone glyph was accepted as a useful temporary activity/status indicator but rejected as the product identity: it is visually associated with VoxType’s speech-to-text action and would be ambiguous when shown beside it.
- The approved product-icon direction is a small original vector persona mark: a minimal stylized head/profile with a subtle transformation accent. It should remain distinct at bar size and be tintable by Omarchy theme state (normal, urgent, and future activity states).

### 2026-09-12 — Phase 10: custom persona icon candidates (awaiting choice)

- Added three standalone, theme-tintable SVG candidates under `assets/icons/` for direct review in an editor rather than repeated shell reloads:
  - `persona-spark.svg`: an approachable person silhouette with transformation sparkles;
  - `persona-profile.svg`: a more distinctive head/profile with a transformation accent;
  - `persona-constellation.svg`: a person silhouette with connected choice/personality marks.
- The candidates use `currentColor`, have a 64×64 view box, and were XML-validated with `xmllint`. They are not wired into the active widget yet.

### 2026-09-12 — Phase 10: solid icon candidates (awaiting choice)

- The user requested denser, filled iconography for better legibility at bar size while retaining the Spark concept.
- Added three additional XML-validated, theme-tintable SVG candidates without removing the outline versions:
  - `persona-spark-solid.svg`: filled person silhouette with solid transformation sparkles;
  - `persona-profiles-solid.svg`: two overlapping filled personas with a transformation sparkle;
  - `persona-profile-solid.svg`: a filled side-profile persona mark with a transformation sparkle.
- These candidates are not wired into the active widget yet.

### 2026-09-12 — Phase 10: selected icon bar test (awaiting visual confirmation)

- The user selected `assets/icons/persona-spark-solid.svg` as the current leading candidate without removing any other icon drafts.
- Switched the bar control from text-only `WidgetButton` to Omarchy’s `BarIconButton` and rendered the selected SVG through a QML theme-tint effect.
- The custom icon uses the active theme’s urgent colour when the engine is unavailable and normal bar foreground when it is available; the SVG itself remains colour-neutral and reusable.
- Ran `omarchy plugin validate .`, `qmllint`, `xmllint`, and `git diff --check`. Manifest, SVG XML, and whitespace checks passed; `qmllint` returned success with the recorded packaged-module warnings.
- Copied the changed `BarWidget.qml` and selected SVG to the active temporary development-plugin directory. Restart the Omarchy shell to force a reliable visual rebuild.
- Initial visual testing showed the SVG rendered black: Qt resolved the SVG's `currentColor` before the first in-place effect was applied.
- Changed the rendering to Omarchy’s established symbolic-icon pattern: a hidden layered source `Image` plus a separate visible `MultiEffect` with `colorization: 1.0`. This should correctly tint the black SVG source with the bar's urgent or foreground colour.
- Re-ran manifest validation, `qmllint`, and `git diff --check`; the checks passed with the recorded packaged-module warnings. Copied the corrected `BarWidget.qml` to the active development-plugin directory.
- The revised effect still rendered black because `MultiEffect.colorization` preserves source luminance: the SVG's `currentColor` resolves to black in Qt and cannot become bright red through that operation.
- Changed only the selected SVG into a white symbolic mask, which retains its alpha shape while allowing the separate theme-tint effect to supply urgent or normal colour. XML, manifest, and whitespace validation passed; copied the revised SVG to the active development-plugin directory.

### 2026-09-12 — Phase 10 complete

- The user visually confirmed the selected `persona-spark-solid.svg` icon in the active Omarchy bar. It is now correctly tinted with the theme's urgent colour while the engine is unavailable.
- Removed the five unselected icon candidates; `assets/icons/persona-spark-solid.svg` is the sole retained project icon asset.
- Phase 10 acceptance is satisfied for the widget foundation: the manifest validates, the widget renders and anchors its own panel, the engine CLI integration and profile selection paths are implemented, and unavailable-engine onboarding is clear without claiming a failure state in the bar. End-to-end profile selection awaits the installed engine delivered by Phase 11.
- Committed Phase 10 as `743cc71 feat: complete Omarchy widget foundation` and pushed it to `origin/main`.

### 2026-09-13 — Phase 11: release contract proposal (awaiting approval)

- Confirmed that the repository has no published engine release, no release workflow, and no Minisign public key yet. This is expected because release tooling is intentionally deferred to Phase 14.
- Proposed a single stable GitHub Releases channel at `https://github.com/toorop/VoxTypePersonas/releases/latest`; prereleases must never be selected by the plugin in v1.
- Confirmed the already-specified fixed stable asset names: `voxtype-personas-x86_64-linux.tar.gz`, `voxtype-personas-aarch64-linux.tar.gz`, `checksums.txt`, and `checksums.txt.minisig`.
- Proposed strict SemVer comparison after stripping one optional leading `v`: installed and release versions must be `MAJOR.MINOR.PATCH` without prerelease or build metadata; only a strictly newer release is offered as an update. A malformed version is invalid, never treated as newer.
- A real Minisign public key remains intentionally undefined pending user approval or provision of a key. It must be generated and kept offline/private by the release owner; only its public key will later be embedded in the plugin.

### 2026-09-13 — Phase 11: local engine discovery (awaiting visual review)

- Added read-only host architecture detection through `uname -m`. Only `x86_64` and `aarch64` continue to engine discovery; every other result is treated as unsupported.
- The widget now resolves the managed engine path as `$XDG_DATA_HOME/voxtype-personas/bin/voxtype-personas`, falling back to `~/.local/share/voxtype-personas/bin/voxtype-personas`. This matches the Rust engine's existing XDG path contract and avoids depending on the Omarchy shell's `PATH`.
- It verifies the managed binary with `voxtype-personas version` before profile commands are enabled. A successful strict SemVer result marks the engine installed; a nonzero result is invalid; an unstartable binary remains safely classified as missing.
- Profile listing and activation now use the verified managed-engine path, not an unqualified command name. No download, installation, configuration write, or Voxtype change is performed.
- Ran `omarchy plugin validate .` and `git diff --check` successfully. `qmllint` exited successfully while retaining the previously recorded unresolved Omarchy-module warnings. The current host reports `x86_64` and has no managed engine installed, which is the expected missing-engine state for this step.

### 2026-09-13 — Phase 11: installation feedback and consent UI (awaiting visual review)

- Added a dedicated unavailable-engine panel with clear states for missing, invalid, checking, and unsupported-architecture engines.
- Selecting Install or Replace opens a confirmation view that names the target architecture and explains that signature and checksum validation occur before installation. Cancel returns to the prior state; no network action occurs before explicit confirmation.
- Added the user-visible installation sequence: preparing the secure download, checking release information, downloading the archive and verification files, verifying the Minisign signature, verifying the checksum, extracting and validating the engine, and installing it. The active stage will be highlighted by the forthcoming transaction implementation.
- Until a real embedded Minisign public key is approved and configured, confirmation safely stops at the first stage with the non-sensitive message `Secure downloads are not configured yet.` It performs no download or installation.
- Ran `omarchy plugin validate .` and `git diff --check` successfully. `qmllint` exited successfully with the existing unresolved Omarchy-module warnings from the packaged-shell layout.

### 2026-09-13 — Phase 11: blocked-installation UX correction (awaiting visual review)

- Visual review showed that displaying the full progress sequence during confirmation was misleading while a real signing key is absent: a user could mistake the static list for a failed or paused installation.
- The missing-engine state now explicitly says that secure releases are not configured and presents a disabled `Installation unavailable` control. The confirmation and progress sequence are withheld until a real embedded Minisign public key makes a secure transaction possible.
- The future confirmation's final action is now labelled `Download and verify engine`, distinct from the initial `Install engine` action. This removes the ambiguous repeated button label.
- Ran `omarchy plugin validate .` and `git diff --check` successfully; `qmllint` again exited successfully with the recorded packaged-module warnings. Copied the revised QML files to the existing temporary development-plugin directory and confirmed matching SHA-256 hashes.

### 2026-09-13 — Phase 11: release-key storage preparation (in progress)

- Decided that the release private key must remain outside the repository, including ignored files. It will be passphrase-protected and backed up separately; only the public key may enter the repository and plugin.
- Created the owner-only key directory at `~/.local/share/voxtype-personas/release-keys` with mode `0700`. No key material exists there yet.
- Minisign is not installed on the host. Its system installation requires an interactive `sudo` password, which this session cannot provide. Key generation is therefore pending local installation of the `minisign` package and an interactive passphrase chosen by the user.

### 2026-09-13 — Phase 11: release signing key established (awaiting visual review)

- Confirmed Minisign 0.12 is installed. The user generated a dedicated passphrase-protected release key pair in `~/.local/share/voxtype-personas/release-keys`.
- Verified only permissions and the public key: the private key is mode `0600`, the containing directory is mode `0700`, and the public key is mode `0644`. The private key was not read, copied, or added to the repository.
- Added `ReleaseConfig.qml` with the stable GitHub Releases URL, fixed asset names, checksum file names, and the generated public Minisign key. Only this public verification material is embedded.
- Renamed the disabled onboarding state to accurately say that this development build cannot install the engine yet. The public key exists, but no installation transaction is enabled until signature and checksum verification are implemented.
- Ran `omarchy plugin validate .`, `qmllint`, and `git diff --check`; validation passed, with only the previously recorded packaged Omarchy module warnings from `qmllint`.

### 2026-09-13 — Phase 11: release asset contract (complete)

- Confirmed that `curl`, `minisign`, `sha256sum`, `tar`, `mktemp`, `install`, and `uname` are available on the development host.
- Added `docs/release-format.md`, defining the stable-only channel, fixed asset names, strict tag/version equivalence, checksum format, and a deliberately minimal archive layout: one root executable named `voxtype-personas`, with no links, path traversal, or extra files.
- The next implementation step is the private-download and verification transaction. It will implement this contract and surface each real progress stage in the panel.

### 2026-09-13 — Phase 11: verification mechanism changed to OpenPGP (in progress)

- User review correctly identified that Minisign is not a guaranteed Omarchy runtime dependency. Requiring users to install it, or bundling a verifier binary, would weaken the onboarding experience.
- Confirmed that this Omarchy host includes `gpg`, `gpgv`, OpenSSL, OpenSSH, and coreutils. The project will use the standard detached OpenPGP signature `checksums.txt.asc`, verified locally with `gpgv` and a plugin-embedded public keyring; no personal keyring access or keyserver contact is required.
- Removed the generated Minisign public key from the plugin source. The Minisign private key remains outside the repository and is not deleted, but it is no longer a production release key.
- Updated the roadmap, specification, README, and release-asset contract from Minisign to detached OpenPGP signatures. A dedicated OpenPGP release key must now be generated before the installer transaction can be implemented.

### 2026-09-13 — Phase 11: OpenPGP release-key storage prepared (in progress)

- Created the dedicated GnuPG home at `~/.local/share/voxtype-personas/release-keys/openpgp` with mode `0700`, separate from the user's personal GnuPG home and from the repository.
- Confirmed GnuPG 2.4.9 is installed. The next action is an interactive generation of a passphrase-protected, signing-only Ed25519 release key; the user will enter the passphrase locally.

### 2026-09-13 — Phase 11: OpenPGP release key established (awaiting review)

- The user generated the dedicated passphrase-protected Ed25519 signing key in the separate GnuPG home. Its public fingerprint is `F344 FE4D EA72 07D5 D135 EF8D 8A6E 3B95 2E12 DF24`; it expires in September 2029.
- Exported only the armored public certificate to `assets/keys/release-signing.asc`. The private key remains outside the repository and was not read, copied, or exported.
- Verified the committed certificate parses to the expected fingerprint, signing capability, and release identity. The detached-signature installation transaction is still pending.

### 2026-09-13 — Phase 11: embedded OpenPGP verifier foundation (awaiting visual review)

- Added `GpgVerifier.qml`, a focused wrapper around `gpgv` that accepts only a plugin-owned public certificate plus manifest and detached-signature paths. It emits generic, non-sensitive verification failures.
- Connected the verifier to the plugin's packaged `assets/keys/release-signing.asc` path. It does not read the user's personal keyring and is not invoked until the download transaction is connected.
- Ran manifest validation, QML lint, and whitespace validation successfully; the QML linter retains only the known unresolved `QProcess::ExitStatus` warning from the packaged module metadata.
- Updated the temporary development-plugin copy with `BarWidget.qml`, `GpgVerifier.qml`, and the public certificate. Matching SHA-256 hashes were confirmed.

### 2026-09-13 — Phase 11: current handoff (incomplete)

- Phase 11 is **not complete**. Implemented: local engine discovery, single-action installation consent, stable/prerelease metadata probing, an embedded OpenPGP public keyring, and a `GpgVerifier.qml` wrapper around `gpgv`.
- The committed OpenPGP foundation is `2a6453a` and is pushed to `origin/main`. Subsequent work is uncommitted.
- A signed x86_64 development prerelease was published at `v0.1.0-test.1`. Its `checksums.txt.asc` was successfully verified with the plugin's binary public keyring. The test assets are staged in `/tmp/voxtype-personas-release-test.uDI6Ff` for the current machine session.
- GitHub CLI authentication was renewed by the user. The development test channel is opt-in through `VOXTYPE_PERSONAS_TEST_RELEASE_TAG=v0.1.0-test.1`; without it, the plugin requests only GitHub's stable `releases/latest` endpoint.
- Important correction: `gpgv` cannot use the armored `.asc` public certificate as a keyring. The correct plugin asset is the binary `assets/keys/release-signing.gpg`, exported from the dedicated external GnuPG home. The obsolete `.asc` key asset has been removed in the uncommitted changes.
- Still required to complete Phase 11: implement and wire one full asynchronous transaction that creates a private temporary directory; downloads the selected archive, `checksums.txt`, and `checksums.txt.asc`; runs `gpgv` against `assets/keys/release-signing.gpg`; checks the selected archive SHA-256; validates archive contents; extracts to staging; runs staged `voxtype-personas version`; atomically promotes it while preserving a rollback binary; cleans temporary files; and reports each real stage in the panel.
- Also still required: complete Update available UX (non-blocking version result, accented icon, panel action/notification), test the transaction against the signed prerelease, copy the final QML/assets to the development-plugin directory for visual testing, and only then mark Phase 11 complete.
- User instruction in force: continue autonomously within Phase 11. Do not pause after micro-changes; ask only for a meaningful UI test, a genuine product/security decision, or before starting Phase 12. The user is sensitive to repeated empty status messages; report only completed, verifiable work.

### 2026-09-13 — Phase 11: single-action installation consent

- User explicitly revised the installation consent policy for a smoother Omarchy onboarding: selecting Install or Update is the sole confirmation. Once selected, the verified transaction progresses automatically until success or a genuine failure.
- Updated the roadmap and specification accordingly. The panel now starts the transaction directly from the Install action; it no longer asks the user to confirm the same intent a second time.

### 2026-09-13 — Phase 11: update-availability UX decision

- A valid installed engine must trigger a non-blocking stable-release check when the plugin loads.
- If a strictly newer engine is available, the persona icon will use Omarchy's theme warning colour and a concise `Update available` notification/action will direct the user to the same single-action verified update flow.
- Update availability is advisory: it must never interrupt dictation, block profile selection, or download anything until the user selects Update.

### 2026-09-13 — Phase 11: verified installer transaction (in progress)

- Added `EngineInstaller.qml`, an asynchronous, argument-only installation state machine. It uses a mode-`0700` private download directory; downloads only the fixed expected asset names over HTTPS; verifies `checksums.txt.asc` with `gpgv` and the embedded binary keyring; and matches the selected archive hash against the signed manifest.
- The transaction rejects archives unless they contain exactly one root-level regular `voxtype-personas` file. It extracts to a mode-`0700` staging directory on the installation filesystem, verifies the staged executable and exact release version, then promotes it with a same-filesystem rename.
- When replacing a working engine, the transaction first preserves it as `voxtype-personas.previous`; a failure to promote the staged binary attempts an immediate restoration. Temporary download and staging directories are removed after success or failure.
- Wired the installer to the single-click Install/Update action and to real panel stages. Release metadata validates strict stable SemVer by default; the existing prerelease route remains opt-in only through `VOXTYPE_PERSONAS_TEST_RELEASE_TAG` and accepts the corresponding base engine version for this development artifact.
- Added a non-blocking update probe after valid engine discovery. A newer stable version marks the bar icon with the theme warning colour, sends a concise `Update available` notification, and exposes an explicit Update action; it does not download automatically.
- Manually validated the actual signed x86_64 prerelease artifact through the same OpenPGP verification, checksum comparison, archive-layout inspection, extraction, permission, and `version` checks used by the transaction. `gpgv` reported the expected release fingerprint and the staged binary reported `0.1.0`.
- Updated README and contributor signing terminology from Minisign to detached OpenPGP signatures.
- Ran `omarchy plugin validate .`, `qmllint`, `cargo fmt --check`, `cargo test --workspace`, and `git diff --check`; all commands succeeded. QML lint retains only the known warnings caused by the packaged Omarchy module metadata.
- Copied the final QML, manifest, icon, and binary keyring to the user-owned development plugin directory and confirmed matching SHA-256 hashes. The current environment has no running `omarchy-shell`, so a live click-through installation test remains the only meaningful outstanding Phase 11 validation.
- Test-channel correction: `omarchy restart shell` deliberately relaunches Quickshell through Hyprland and does not inherit transient terminal variables or the user systemd manager environment. To make `VOXTYPE_PERSONAS_TEST_RELEASE_TAG` visible to the test shell, it must be set in Hyprland's runtime environment before restarting the shell.

### 2026-09-13 — Phase 11 complete

- Performed the live end-to-end installation test using the signed `v0.1.0-test.1` prerelease through the opt-in Hyprland runtime variable. The user confirmed every panel stage completed and that the widget reached the active Raw profile state.
- The observed initial stable-release error was expected without a stable release. Corrected the developer test setup: the shell is launched from Hyprland, so the tag must be set with `hyprctl eval 'hl.env(...)'`, not through terminal or systemd user-manager environment variables.
- Documented the developer-only prerelease procedure, expected successful result, and cleanup commands in `docs/release-format.md`. The stable channel remains the default and never selects prereleases.
- Reviewed the Phase 11 roadmap acceptance criteria against the transaction implementation, signed-artifact validation, automated checks, and live installation. They are satisfied. The next proposed step is Phase 12, only after user approval.

### 2026-09-15 — Phase 12: anchored settings navigation (awaiting user validation)

- Added a `Settings…` entry to the existing anchored profile selector. It opens an in-panel settings view and never launches a separate application.
- Added the Phase 12 navigation structure with `Profiles`, `Prompts`, and `Providers` sections, plus a return action to the profile selector. The selected section is visually distinguished.
- Kept profile selection, engine installation/update states, and Escape-to-close behavior unchanged. Closing the panel returns it to the compact selector for the next opening.
- This focused first step intentionally adds no profile/prompt/provider form, no configuration mutation, and no Secret Service operation. Each section clearly identifies that its management UI is still pending.
- Ran `omarchy plugin validate .`, `qmllint`, and `git diff --check`. The manifest validation and whitespace check passed; QML lint completed with the pre-existing unresolved Omarchy-module metadata warnings.
- No live visual test was run during this implementation step because the user had removed `~/.config/omarchy/plugins/io.github.toorop.voxtype-personas`.

### 2026-09-15 — Omarchy development-plugin reinstall

- Recreated the user-owned development-plugin directory at `~/.config/omarchy/plugins/io.github.toorop.voxtype-personas` from the repository working tree, including the manifest, QML sources, persona icon, and embedded public OpenPGP keyring.
- Confirmed the installed copy matches the repository files by SHA-256.
- Restarted the Omarchy shell, rescanned plugins, and enabled `io.github.toorop.voxtype-personas` in the right bar section. Confirmed that `shell.json` contains the widget and that Omarchy reports it as enabled.
- Validated the installed plugin manifest with `omarchy plugin validate ~/.config/omarchy/plugins/io.github.toorop.voxtype-personas`.
- The widget is ready for iterative live UI testing; the next test should open `Settings…` and verify the Profiles, Prompts, Providers navigation and return action.

### 2026-09-15 — Phase 12 UI review: profile selector direction

- The user visually reviewed the anchored settings navigation and confirmed it is available for iterative testing.
- The current compact profile selector is not an acceptable final interaction: replace the list of clickable profile rows with a proper compact selection control in a future approved Phase 12 step.
- Preserve the existing eligibility rules in that control: Draft profiles remain visible but cannot be activated; Ready and Active profiles remain selectable; changing the selection must persist through the engine CLI.

### 2026-09-15 — Phase 12: compact profile selector (awaiting user validation)

- Replaced the compact selector's always-visible clickable profile list with a single select-style control that displays the active profile and opens a themed dropdown on click or keyboard activation.
- The dropdown displays every profile with its readiness state. Draft rows are visually subdued, use the forbidden cursor, and report a fixed configuration-required message instead of invoking the engine. Ready and Active rows retain the existing persisted engine-CLI activation path.
- While the dropdown owns focus, the enclosing panel keyboard catcher is suspended so its navigation shortcuts do not interfere with menu keys. Escape closes the dropdown first; closing the panel resets it to the compact selector view.
- Validated the plugin manifest, QML syntax with `/usr/lib/qt6/bin/qmlformat`, and whitespace with `git diff --check`. The installed `qmllint` still reports its known unresolved Omarchy-module imports and did not finish within a 20-second local timeout, so it is not recorded as a passing check.
- Copied the updated `Panel.qml` to the enabled user-owned development plugin and confirmed the repository and installed copies have the same SHA-256 hash. In this environment, the user must manually restart the Omarchy shell before a copied development-plugin change is reliably reflected in the live UI; do not treat file watching alone as sufficient visual-test deployment.

### 2026-09-15 — Phase 12: profile-management engine contract (in progress)

- Added atomic engine operations for `profiles create`, `profiles duplicate`, `profiles rename`, and `profiles delete`; each accepts only arguments and writes configuration through the existing private atomic persistence path.
- Creation produces a Draft profile with a separate initial prompt. Duplication gives the new profile an independent prompt copy, so later prompt editing cannot mutate its source profile.
- Deleting the active non-Raw profile safely selects Raw first. Deleting a profile removes its prompt only when no remaining profile references it. The mandatory Raw profile is protected from deletion and from reserved-ID creation.
- Added storage coverage for creation, renaming, independent duplication, prompt cleanup on deletion, and Raw protection.
- Ran `cargo fmt`, `cargo test --workspace`, and `git diff --check`; all checks passed with 50 unit tests and 12 integration tests.
- The panel controls for creating, duplicating, renaming, and confirming deletion remain the next focused sub-step of Phase 12 item 2.

### 2026-09-15 — Phase 12: Profiles management view (awaiting user validation)

- Replaced the Profiles settings placeholder with a selectable list of local profiles. Each row shows its name and readiness state; Raw is explicitly labelled as the mandatory protected profile.
- Selecting a profile reveals the intended management area. Selecting Raw explains that it cannot be renamed or deleted; selecting another profile reserves the area for the forthcoming create, duplicate, rename, and delete controls.
- This is a visual-layout step only: it does not yet invoke the newly added engine mutation commands or modify configuration from the panel.
- Validated QML syntax with `/usr/lib/qt6/bin/qmlformat`, the plugin manifest, and whitespace. Copied the updated `Panel.qml` to the enabled development plugin and confirmed matching SHA-256 hashes.
- The user manually restarted the Omarchy shell and visually validated the view. Draft `Example` remains correctly unavailable in the compact active-profile selector, but is selectable in `Settings… > Profiles` for future management actions.

### 2026-09-15 — Phase 12: Profiles mutation controls (awaiting user validation)

- Added create, rename, duplicate, and two-step delete controls to `Settings… > Profiles`. The delete action explains that it removes an unshared prompt and requires a second explicit click; Raw has no mutation controls.
- Each control invokes only the managed engine's argument-based `profiles` CLI contract and refreshes the profile list after a successful result. UI failures use a fixed non-sensitive message.
- Built the development engine and replaced the local managed test binary at `~/.local/share/voxtype-personas/bin/voxtype-personas` with mode `0700`, so it exposes the new profile mutation commands for live testing. This is a development-test replacement, not a signed release installation.
- Copied the updated `Panel.qml` to the enabled development plugin and confirmed matching SHA-256 hashes. QML syntax, plugin-manifest validation, and `git diff --check` passed.
- The user must manually restart the Omarchy shell before testing the controls.

### 2026-09-15 — Phase 12: profile deletion diagnosis (in progress)

- The user successfully created, renamed, duplicated, and deleted a created profile, but the second-click confirmation did not visibly delete a duplicated profile.
- Corrected an initial false diagnosis: the workspace sandbox mounts most of `/home` read-only while exposing the repository as a writable nested mount. That restriction does not apply to the user host. Host-level inspection confirms `/home` is read-write, the configuration is valid, and `voxtype-personas profiles delete copy-of-test` successfully removed the remaining duplicate.
- The fault is therefore in the panel interaction or its feedback, not Btrfs or engine persistence. Added a fixed non-sensitive mutation error display to the Settings Profiles view; further UI interaction diagnosis remains required.
- Confirmed there are no open locks on `config.toml`. Added a temporary QML diagnostic that records only the mutation verb, profile ID, and process exit code in the Omarchy shell log; it never records profile content, prompts, or secrets. Deployed it to the development plugin for the next user retry.
- Root cause identified: `BarWidget.qml` called `trim()` on the complete `profiles list` output before parsing rows. The first inactive row is sorted first and therefore lost its two leading marker spaces; the parser then removed two real ID characters, turning `copy-of-test` into `py-of-test`.
- Removed the whole-output trim while continuing to ignore blank lines. This preserves every row marker and ID. Validated the QML and plugin manifest, then deployed the updated `BarWidget.qml` to the development plugin with matching SHA-256 hashes. The user must restart the shell before retesting.
- Improved duplicate naming: the first copy is named `Copy of <name>` and subsequent copies increment visibly as `Copy of <name> 2`, `3`, and so on. The technical profile ID is generated independently from the unique display name. Validated and deployed the updated panel for a future user test.
- The user noted that broader profile-management ergonomics still feel unclear; defer that product-level UI review until the Phase 12 management surface is complete.

### 2026-09-15 — Phase 12 item 2 complete: profile management

- Completed and visually validated profile creation, renaming, duplication, and deletion with an explicit two-click destructive confirmation. Raw remains protected in both the engine and the panel.
- Fixed a first-row profile parsing defect that truncated the first inactive profile ID after whole-output whitespace trimming. This specifically affected duplicated profiles because their `copy-of-...` IDs sorted first.
- Confirmed repeated copies receive distinct visible names and IDs. The next Phase 12 item is to review and reinforce the Raw invariant at the data-model and panel levels.

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

After the user approves continuation, begin Phase 12 — extended settings panel. Before any live Omarchy UI test, reinstall the plugin into `~/.config/omarchy/plugins/io.github.toorop.voxtype-personas`, because the user removed that development-plugin directory after Phase 11 validation.

## Commit and push status

- Phase 0 repository baseline committed with message `chore: establish repository baseline`.
- Phase 3 was pushed to `origin/main` as part of commit `9f36468`.
