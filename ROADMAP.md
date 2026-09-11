# VoxTypePersonas Roadmap

## Purpose and working method

This roadmap is the ordered delivery plan for VoxTypePersonas v1. Each numbered phase is intentionally small and independently reviewable. Work stops after every phase until the user validates it and explicitly asks to continue.

The project delivers one Omarchy `bar-widget` plugin and one Rust engine binary. The widget selects and configures post-processing personas; the engine is the stable command invoked by Voxtype and is the only component that processes dictated text.

## Fixed product constraints

- Target only Omarchy on Linux.
- Plugin ID: `io.github.toorop.voxtype-personas`.
- Engine command: `voxtype-personas`.
- Engine implementation language: Rust.
- User interface and shipped documentation: English.
- Engine assets: Linux `x86_64` and `aarch64`.
- Secrets: Secret Service/keyring only; no file-based fallback.
- Default active profile: `Raw` (the UI name for the mandatory built-in raw profile).
- No telemetry and no persisted dictated text, prompts, or secrets in logs.
- No changes to the existing Hyprland binding or PipeWire mute relay.
- No automatic installation or update of the engine.

## Delivery sequence

### Phase 0 — Repository baseline

Goal: establish a minimal, reviewable repository contract before implementation.

1. Inspect the repository after Git initialization and preserve existing documentation.
2. Add the project-level files required for distribution: MIT license, English README outline, ignore rules, and contributor-facing development conventions where useful.
3. Define the repository layout for the Rust engine, Omarchy plugin sources, fixtures, CI, and release tooling.
4. Record supported host assumptions and required local tools without installing dependencies automatically.
5. Review the proposed layout with the user before generating application code.

Acceptance criteria:

- The repository structure is explicit and contains no implementation beyond project scaffolding.
- The README accurately describes scope, non-goals, privacy, and the incremental workflow.
- No commit is created unless the user explicitly asks for one.

### Phase 1 — Configuration contract and Rust workspace

Goal: create the engine workspace and define the configuration model without any provider networking.

1. Create the Rust package for the `voxtype-personas` binary.
2. Add a stable CLI skeleton for `version`, `process`, `profiles list`, `profiles set-active`, `providers test`, and `config validate`.
3. Define typed, versioned TOML configuration structures:
   - `schema_version`;
   - active profile ID;
   - provider definitions;
   - prompt definitions;
   - profile definitions;
   - execution limits.
4. Define XDG paths for configuration, state, engine installation, temporary downloads, backups, and logs.
5. Implement safe path discovery and restrictive directory/file permissions where the platform allows it.
6. Define the initial default configuration in code, including Raw, Chat, Email, Technical, and Meeting / Notes profiles.
7. Add unit tests for parsing, validation, default values, invalid references, duplicate IDs, and unsupported schema versions.

Acceptance criteria:

- `config validate` gives clear, secret-free diagnostics.
- The configuration model can represent every v1 provider and profile requirement.
- No real API key, network call, or UI is needed for this phase.

### Phase 2 — Atomic persistence and configuration migration

Goal: make profile selection and future configuration changes reliable.

1. Implement first-run creation of the default configuration.
2. Implement atomic TOML writes using a same-directory temporary file, flush/sync where appropriate, restrictive permissions, and atomic replacement.
3. Preserve the previous configuration before every schema migration.
4. Implement sequential migration dispatch from older supported schemas to the current schema.
5. Implement loading behavior for malformed, inaccessible, or partially written configuration files.
6. Implement `profiles list` and `profiles set-active <id>` against persisted configuration.
7. Prevent deletion or invalidation of the mandatory Raw profile at the data-model level.
8. Add failure-oriented tests for interrupted writes, malformed TOML, invalid active profiles, and migration backup creation.

Acceptance criteria:

- Selecting a profile survives a new engine process.
- A failed write does not replace the last valid configuration.
- Migration failures are diagnosable and do not silently discard user configuration.

### Phase 3 — Raw processing path and CLI behavioral contract

Goal: provide a safe usable baseline with no LLM dependency.

1. Implement complete UTF-8 input reading from standard input.
2. Resolve the active profile exactly once at the start of each `process` invocation.
3. Implement `process` and `process --profile <id>` without changing persisted active state.
4. Implement the Raw profile as an exact `stdin` to `stdout` pass-through.
5. Ensure empty input is returned unchanged and does not invoke a provider.
6. Enforce the output protocol: final pasteable text only on standard output; diagnostics only on standard error.
7. Define recoverable versus non-recoverable exit semantics, preserving raw input whenever possible.
8. Add integration tests that assert byte-preserving Raw behavior and clean standard output.

Acceptance criteria:

- `printf 'text' | voxtype-personas process` returns precisely `text` when Raw is active.
- No network-capable code path is reached for Raw.
- Profile resolution is stable for the duration of a single invocation.

### Phase 4 — Fallback policy and response validation

Goal: guarantee that a failed post-processing attempt never blocks a dictation.

1. Centralize the processing outcome and fallback decision in the engine.
2. Define recoverable provider failures: unavailable provider, timeout, network error, authentication error, HTTP error, parsing error, and invalid response.
3. Return the raw transcription on every recoverable provider failure with a zero exit status.
4. Define validation rules for a provider response:
   - non-empty text;
   - valid text encoding;
   - no obvious conversational preamble;
   - no Markdown framing when the profile requires plain output.
5. Keep diagnostics concise and never include dictated text, prompts, secret values, or authorization headers.
6. Add tests for each fallback condition and for standard-output cleanliness.

Acceptance criteria:

- Every recoverable failure returns the original transcription.
- Diagnostics never disclose sensitive or dictated content.
- The fallback behavior is covered by automated tests before provider implementations are added.

### Phase 5 — Secret Service abstraction

Goal: add mandatory keyring support without allowing secret leakage.

1. Select and document the Rust integration approach for Secret Service on Omarchy.
2. Define a small secret-store abstraction with read, write, verify, replace, and delete operations.
3. Store only opaque secret references in `config.toml`.
4. Define a stable service and item naming convention scoped to VoxTypePersonas providers.
5. Verify a secret during provider creation or modification, not for the first time during dictation.
6. Treat an unavailable or locked keyring as a configuration/provider error with an actionable, non-sensitive message.
7. Add tests using a fake secret store; add integration coverage only where an isolated Secret Service test environment is practical.

Acceptance criteria:

- Configuration files never contain API keys.
- The engine has no file-based secret fallback.
- Error messages identify the required corrective action without revealing key material.

### Phase 6 — Provider interface and Ollama adapter

Goal: establish the first real LLM path using a local provider.

1. Define the provider adapter interface: capability discovery, model listing, connection test, and text processing.
2. Define a normalized request model containing system prompt, delimited user transcription, selected model, timeout, and limits.
3. Implement the Ollama protocol adapter.
4. Implement Ollama availability detection and installed-model discovery.
5. Send the prompt as the system instruction and dictated text as separate user content.
6. Apply profile-specific input-character and output-token limits before sending requests.
7. Map all adapter failures to the centralized fallback policy.
8. Build a mock HTTP test suite for request shape, timeout handling, response parsing, and fallbacks.

Acceptance criteria:

- A configured local Ollama profile processes a test transcription.
- Ollama failures return the original transcription.
- The test suite proves that system and dictated user content are distinct.

### Phase 7 — OpenAI-compatible providers

Goal: support OpenAI, Mistral, Groq, OpenRouter, and named reusable compatible endpoints.

1. Implement the shared Chat Completions-compatible adapter.
2. Add built-in endpoint metadata for OpenAI, Mistral, Groq, and OpenRouter.
3. Add a generic named OpenAI-compatible provider with configurable endpoint, models, secret reference, and timeout.
4. Implement model-list retrieval where supported and normalized text-capability filtering.
5. Require explicit remote model selection; do not silently choose a remote default model.
6. Implement a minimal dedicated provider test request that does not contain actual dictation.
7. Validate URLs, timeouts, model IDs, and provider references before saving configuration.
8. Add mock-server tests for request bodies, auth handling, error mapping, and model filtering.

Acceptance criteria:

- Each compatible provider can be configured, tested, and used through the same profile contract.
- The generic provider can have multiple named instances.
- Authentication data never appears in configuration or diagnostics.

### Phase 8 — Native Anthropic and Gemini adapters

Goal: add providers whose protocols differ from Chat Completions.

1. Implement the Anthropic Messages API adapter with correct system and user separation.
2. Implement the Gemini text-generation adapter with correct system instruction and user content separation.
3. Implement capability/model discovery and filtering as allowed by each API.
4. Reuse the common request, limits, output validation, keyring, and fallback layers.
5. Add protocol-specific mock tests, including unsuccessful responses and malformed payloads.

Acceptance criteria:

- Anthropic and Gemini are behaviorally equivalent to other providers from the profile and CLI perspective.
- Protocol differences do not weaken privacy, fallback, or output requirements.

### Phase 9 — Built-in prompts and profile rules

Goal: make the shipped personas useful while preserving user voice.

1. Draft English system prompts for Chat, Email, Technical, and Meeting / Notes.
2. Require output in the input language.
3. Explicitly state that instructions embedded in dictated text do not change the post-processing task.
4. Make the prompts focused on correction and intent preservation rather than generic formalization.
5. Define profile defaults for provider/model assignment only when a local setup can be represented safely; otherwise leave profiles clearly unconfigured.
6. Add prompt fixtures covering conversational speech, email tone, technical vocabulary, and instruction-injection-like input.
7. Review prompt wording with the user before treating it as a shipped default.

Acceptance criteria:

- Chat retains a natural, direct tone.
- Technical preserves commands, code, product names, and jargon.
- Built-in prompts are editable through the configuration model.

### Phase 10 — Omarchy plugin foundation

Goal: create a valid, non-invasive `bar-widget` plugin shell.

1. Read and apply the current Omarchy plugin contract before implementation.
2. Add `manifest.json` with the fixed plugin ID and `bar-widget` entry point.
3. Add `BarWidget.qml` using Omarchy/Quickshell conventions.
4. Add `Panel.qml` loaded by the widget itself, never as a second Quickshell process or separate plugin type.
5. Display the active profile name and the compact anchored selector.
6. Wire profile selection to the engine CLI and refresh the widget state after success.
7. Add clear empty/error states for a missing or invalid configuration.
8. Validate the manifest with `omarchy plugin validate` and QML with the applicable `qmllint` setup.

Acceptance criteria:

- The plugin validates and renders a bar button.
- The compact menu is anchored to its own button.
- Selecting a profile persists and updates the visible label without restarting Voxtype.

### Phase 11 — Engine discovery, installation, and rollback

Goal: safely install a released engine from the plugin UI only after consent.

1. Define the embedded release channel, public Minisign key, asset naming, and version comparison policy.
2. Detect the host architecture using `uname -m` and map it to `x86_64-linux` or `aarch64-linux`.
3. Detect whether the installed engine is absent, invalid, outdated, or compatible.
4. Add UI states that explain the condition and offer an explicit Install or Update action.
5. Require a confirmation step before every download, installation, or update.
6. Download the correct archive, `checksums.txt`, and `checksums.txt.minisig` into a private temporary directory.
7. Verify the Minisign signature using the embedded public key.
8. Verify the selected archive SHA-256 against the verified checksum manifest.
9. Extract into a staging directory; verify the binary using `voxtype-personas version`.
10. Promote the staged version atomically while preserving the previous working binary for rollback.
11. Surface actionable errors and leave the previous binary intact on any failure.

Acceptance criteria:

- No download happens without explicit confirmation.
- An untrusted, malformed, or checksum-mismatched archive is never installed.
- A failed update does not remove a working installed engine.

### Phase 12 — Extended settings panel

Goal: configure personas without exposing secrets or opening a standalone app.

1. Extend the anchored panel with Profiles, Prompts, and Providers sections.
2. Implement profile creation, duplication, editing, and deletion, with confirmation for destructive actions.
3. Enforce that Raw cannot be deleted and cannot make the configuration invalid.
4. Implement prompt editing with a proper multiline editor, validation guidance, and a safe test action.
5. Implement provider creation/editing for all v1 types.
6. Add secure API-key capture that writes directly to Secret Service and never pre-fills or redisplays the key.
7. Add provider model discovery, filtering, explicit model selection, timeout controls, and connection testing.
8. Display the remote-provider privacy disclosure before a remote profile is activated.
9. Implement Escape-to-close and clear save/cancel/error states.
10. Ensure panel errors never include dictated text, prompts, or secrets.

Acceptance criteria:

- All v1 entities can be managed in the anchored panel.
- Destructive operations require confirmation.
- The panel remains part of the widget and no separate application is launched.

### Phase 13 — Voxtype integration assistant

Goal: connect the installed engine to Voxtype safely and reversibly.

1. Detect the existing Voxtype configuration and current post-processing command.
2. Present the precise proposed change to use `voxtype-personas process`.
3. Never overwrite an existing command without explicit confirmation.
4. Back up the original Voxtype configuration before a confirmed modification.
5. Write the integration change atomically and validate the resulting syntax.
6. Provide a non-destructive status view explaining whether integration is active.
7. Define a restoration flow that uses the recorded backup and asks for confirmation.
8. Confirm that no Hyprland binding or PipeWire configuration is read, modified, or restarted by the integration assistant.

Acceptance criteria:

- The only integration change is Voxtype's post-processing command.
- Existing configuration is backed up before modification.
- The existing audio mute relay and dictation binding are unaffected.

### Phase 14 — Release tooling and CI/CD

Goal: produce signed, reproducible engine assets and validate plugin quality.

1. Add Rust formatting, linting, unit tests, and integration tests to CI.
2. Build release binaries for Linux `x86_64` and `aarch64`.
3. Package each binary into the fixed release archive names.
4. Generate `checksums.txt` with SHA-256 values.
5. Add a protected signing workflow for `checksums.txt` using the release Minisign private key supplied only through secure CI secrets.
6. Publish release assets on version tags.
7. Add a manually triggered, controlled prerelease workflow.
8. Validate plugin manifest and QML in CI when a suitable Omarchy-compatible environment is available; otherwise document the required local validation explicitly.
9. Document release, signing-key rotation, rollback, and emergency revocation procedures.

Acceptance criteria:

- Every stable release includes two architecture archives, checksums, and a valid Minisign signature.
- No signing secret is committed or printed in CI logs.
- A failed build or test blocks publication.

### Phase 15 — End-to-end validation and documentation

Goal: verify the acceptance criteria on supported Omarchy environments and prepare v1 delivery.

1. Test first-run behavior with no engine installed.
2. Test confirmed installation and rejected installation flows.
3. Test Raw processing with no network access.
4. Test each provider with a non-sensitive controlled fixture and forced failure conditions.
5. Test timeout, offline, invalid output, unavailable keyring, and invalid configuration fallbacks.
6. Test profile selection during a dictation cycle to confirm that the next invocation, not the current one, observes the new selection.
7. Test the widget compact menu, extended panel, Escape behavior, and destructive confirmations.
8. Test preservation and restoration of a pre-existing Voxtype post-processing configuration.
9. Confirm no changes to existing Hyprland/PipeWire behavior.
10. Complete the English README: installation, prerequisites, privacy, providers, cost responsibility, configuration, Voxtype integration, troubleshooting, uninstall, and development/release steps.
11. Run all documented validation commands and record the results in `state.md`.

Acceptance criteria:

- All specification acceptance criteria are verified.
- The README lets an Omarchy user install, configure, troubleshoot, and uninstall the project safely.
- The v1 release scope remains limited to the documented scope.

## Deferred work after v1

- A versioned, repository-hosted prompt catalog for sharing curated personas. It should keep reusable prompt definitions separate from each user's local providers, models, configuration, and secrets. Its design should later cover metadata, examples, review criteria, compatibility, and an optional import flow.
- Import and export of profiles and prompts.
- Cloud synchronization.
- Community prompt sharing.
- Streaming post-processing.
- Native packages for specific Linux distributions.
- Cost display, calculation, or enforcement.
- Additional architectures and operating systems.

## Change-control rules

- A phase may be split further if implementation reveals a material risk.
- A change that affects privacy, secret storage, engine distribution integrity, or Voxtype configuration requires explicit user review before implementation.
- Each completed phase is recorded in `state.md` with changed files, validation performed, decisions made, and the next proposed step.
- Commits and pushes occur only after explicit user instruction.
