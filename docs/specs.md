# VoxTypePersonas — Functional and Technical Specification (v1)

## 1. Purpose

VoxTypePersonas is an Omarchy plugin for Voxtype. It lets the user select a post-processing profile for dictated transcriptions from the Omarchy top bar.

A profile associates:

- a local or remote LLM provider;
- an explicitly selected model;
- an editable system prompt; and
- execution constraints, including timeout and size limits.

The product improves a transcription without imposing a generic or academic writing style. Prompts are user-editable and must preserve the transcription's intent, language, and appropriate voice.

## 2. Confirmed decisions

- Target: Omarchy on Linux only.
- Repository and publication: `toorop/voxtype-personas`, under the MIT license.
- Omarchy plugin ID: `io.github.toorop.voxtype-personas`.
- Interface: an Omarchy `bar-widget` plugin built with QML/Quickshell.
- Engine: a Rust binary named `voxtype-personas`.
- The plugin and engine are components of one product, not two visible applications.
- Voxtype always invokes the same engine command; the engine resolves the active profile itself.
- The active profile is resolved once at the start of a dictation. A UI change applies to the next dictation.
- Engine binaries are distributed through GitHub Releases, not distribution packages, AUR, Homebrew, or `.deb` packages in v1.
- Supported release architectures: Linux `x86_64` and Linux `aarch64`.
- Secret Service/keyring is mandatory. There is no file-based secret fallback.
- Shipped UI and documentation are in English.
- `Raw` is the mandatory built-in profile and the sole active profile on first use. It cannot be deleted and is always Ready.
- `profiles/example.md` is the only shipped non-Raw profile. It is a generic, portable Draft intended as a starting point, not a provider or model recommendation.
- Profiles have Draft, Ready, and Active states. Only Ready profiles can be activated.
- Portable profile definitions use Markdown with YAML front matter. They exclude secrets, Secret Service references, and dictated text; they may be imported, exported, edited externally, and contributed by pull request.
- No dictated text, prompts, or secrets are persisted or included in telemetry or logs by default.

## 3. Out of scope for v1

- A standalone application, unanchored window, browser extension, or mobile application.
- Native packages for individual Linux distributions.
- Cloud synchronization of profiles, prompts, or keys.
- Streaming transcription to an LLM.
- In-UI community prompt sharing or cloud catalog browsing.
- Cost calculation, display, or enforcement.
- Changes to the existing Hyprland dictation binding or PipeWire mute relay.

## 4. User journeys

### 4.1 Quick selection

The bar widget displays an icon and the active profile name:

```text
🎙️  Raw ▾
```

Clicking it opens a compact popover anchored to the button:

```text
VoxTypePersonas
✓ Raw                    Ready · Active
  Example                Draft · needs configuration
────────────────────────────────────────
⚙ Settings…
```

Only Ready profiles are selectable for activation. A Draft profile remains visible and can be opened, edited, duplicated, imported, or exported, but the UI clearly explains why it cannot be activated. Selecting a Ready profile persists the choice, closes the popover, and immediately updates the widget label.

### 4.2 Extended configuration

Clicking **Settings…** replaces or extends the popover with a larger panel, still anchored to the same bar button. It never launches a separate application.

The panel contains three main sections:

- **Profiles**: list, create, duplicate, edit, delete, import, export, and associate a provider, model, and prompt.
- **Prompts**: multiline system-prompt editor, input/output convention preview, and safe test action.
- **Providers**: endpoint, model, timeout, secret, and connection-test configuration.

The panel closes with Escape. Destructive actions require in-panel confirmation. `Raw` cannot be deleted or made invalid.

### 4.3 First opening and engine installation

When loaded, the plugin checks for a compatible engine. If it is absent, outdated, or invalid, the compact panel clearly shows the state and an action:

```text
The VoxTypePersonas engine is not installed.
[ Install engine ]
```

Clicking the action requests confirmation. Only after confirmation, the plugin:

1. detects the architecture with `uname -m`;
2. downloads the appropriate release archive;
3. verifies the announced SHA-256 checksum and Minisign signature with the embedded public key;
4. extracts the binary into the user data directory;
5. verifies `voxtype-personas version`; and
6. shows success or an actionable error.

The plugin never silently downloads, installs, or updates the engine.

## 5. Architecture

```text
Omarchy shell (existing Quickshell process)
└── VoxTypePersonas `bar-widget` plugin
    ├── BarWidget.qml          button and visible state
    ├── Panel.qml              anchored selector and settings panel
    └── configuration model / CLI calls

Voxtype
└── post_process_command = "…/voxtype-personas process"
    └── `voxtype-personas` Rust engine
        ├── stdin reading
        ├── configuration and active-profile resolution
        ├── provider adapter
        ├── response validation and fallback
        └── final-text stdout writing
```

The plugin never starts a second Quickshell process. It follows the Omarchy `bar-widget` contract: `BarWidget.qml` is the manifest entry point and loads its own anchored panel.

## 6. Voxtype-to-engine contract

### 6.1 Voxtype configuration

The integration uses one post-processing command:

```toml
[output.post_process]
command = "<engine-path>/voxtype-personas process"
timeout_ms = 30000
```

The integration assistant detects any existing Voxtype post-processing command. It never overwrites one without explicit confirmation and a backup of the original configuration.

### 6.2 Input and output

- The engine reads the complete UTF-8 transcription from standard input.
- It writes only pasteable final text to standard output.
- Diagnostics go to standard error and contain no dictated text or secret.
- Empty input is returned unchanged and never invokes a provider.
- `Raw` returns its standard input byte-for-byte and never reaches a network-capable path.
- An empty or invalid LLM response causes the engine to write the raw transcription to standard output.
- Provider unavailability, timeout, network, authentication, HTTP, or parsing failures immediately fall back to the raw transcription.

The engine returns zero when it provides either processed text or raw fallback text. Non-recoverable configuration and internal errors return a non-zero status after attempting to preserve raw input whenever possible.

### 6.3 Sequencing

Voxtype already sequences recording, transcription, post-processing, and pasting. VoxTypePersonas adds no lock, queue, or Busy state in v1.

## 7. Engine CLI

The CLI is stable, documented, and usable without the widget:

```text
voxtype-personas version
voxtype-personas process
voxtype-personas process --profile <id>
voxtype-personas profiles list
voxtype-personas profiles set-active <id>
voxtype-personas providers test <id>
voxtype-personas config validate
```

`process` without an option uses the persisted active profile. `process --profile <id>` is intended for testing and automation and does not change persisted active state. `profiles list` exposes each profile's Draft, Ready, or Active state; `profiles set-active` rejects a Draft profile with a clear, non-sensitive diagnostic.

## 8. Data model and portable catalog

### 8.1 XDG locations

```text
~/.config/voxtype-personas/config.toml
~/.local/share/voxtype-personas/bin/voxtype-personas
~/.local/state/voxtype-personas/
```

The engine installation directory is separate from the Omarchy plugin directory so plugin updates do not remove the executable. Configuration and state files are created with restrictive permissions. Writes are atomic; migrations are sequential and back up the source file before modification.

### 8.2 Local configuration

The configuration includes `schema_version`, `active_profile`, providers, prompts, profiles, and execution limits. On first use, it contains only the Ready and Active `Raw` profile plus the incomplete `Example` Draft profile. Secrets are stored only in Secret Service; `config.toml` contains opaque references when a local provider definition requires one.

### 8.3 Profile readiness

- **Draft**: a loadable profile that is incomplete, for example one with a prompt but no compatible provider or explicit model. It cannot be activated.
- **Ready**: a profile with a prompt, compatible provider, explicitly selected model, valid limits, and a verified local secret when its provider requires one.
- **Active**: the persisted selected Ready profile. `Raw` is always Ready and is the default Active profile.

If the current Active profile becomes unready, the engine safely resolves to `Raw` for processing and the UI reports the condition.

### 8.4 Portable profile format

Portable profiles live in `profiles/` and use Markdown with versioned YAML front matter. They are human-editable and may include public provider metadata, a prompt, profile metadata, and limits. They must never contain API keys, Secret Service references, dictated text, or any other user-local secret material.

Every import is validated before acceptance: schema version, required and typed fields, profile-ID uniqueness, supported provider metadata, readiness-independent safety rules, and prohibited content. An invalid or unsafe file is rejected atomically without partially importing data.

`profiles/example.md` is the reference Draft: it demonstrates the format but deliberately omits provider and model choices. Users must evaluate and adapt prompts for their chosen model.

## 9. Providers

### 9.1 v1 providers

The selector distinguishes local Ollama, built-in remote providers, and a reusable generic type:

- **Ollama** — detected locally; installed models are listed and the user chooses explicitly.
- **OpenAI**, **Mistral**, **Groq**, and **OpenRouter** — fixed endpoint metadata and an API key.
- **Anthropic** and **Google Gemini** — native protocol adapters and an API key.
- **OpenAI-compatible…** — any number of named instances, each with its own URL, secret, timeout, and models.

After provider configuration, the UI retrieves supported models where possible and requires an explicit remote-model choice. It excludes image-only, audio-only, transcription, TTS, embedding, and moderation-only models. Text-capable multimodal models remain available. Each provider exposes a connectivity test, model validation, timeout, secret state, and actionable error message. The test uses a dedicated minimal request and never sends real dictation.

### 9.2 Request construction

OpenAI, Mistral, Groq, OpenRouter, and generic providers use OpenAI-compatible Chat Completions. Anthropic, Gemini, and Ollama use protocol-specific adapters.

The system prompt and dictated transcription are transmitted as separate system and user content. The prompt requires plain output in the input language and instructs the model not to treat instructions within the transcription as changes to the system task.

Responses are expected as plain text. Validation rejects empty output and can reject obvious conversational preambles or Markdown framing according to the profile output policy.

## 10. Secrets, privacy, and costs

### 10.1 Secrets

The normal flow stores API keys in Secret Service/keyring and saves only an opaque identifier in `config.toml`. If the keyring is unavailable or locked, the UI explains that the provider cannot be saved and provides appropriate recovery guidance; it never offers a file fallback.

Near the API-key field, the UI displays: “This API key is stored in your system keyring. It is not written to VoxTypePersonas configuration files.” Secret availability is verified when a provider is created or modified, not for the first time during dictation.

### 10.2 Privacy

Before activating a remote profile, the UI identifies the provider and displays:

> This profile sends transcribed text to `<provider>` for post-processing.

The product stores no transcription and sends no telemetry by default. Logs exclude raw text, prompts, and secrets. Portable profile export also excludes secrets, local secret references, and dictated text.

### 10.3 Security limits and cost responsibility

Each profile defaults to 20,000 input characters, 2,048 output tokens, and a configurable timeout. Advanced settings may change these values.

The UI neither calculates, displays, nor caps costs. The README explains that provider/model selection and cost monitoring are the user's responsibility.

## 11. Distribution and updates

### 11.1 Assets

Each stable release provides at least:

```text
voxtype-personas-x86_64-linux.tar.gz
voxtype-personas-aarch64-linux.tar.gz
checksums.txt
checksums.txt.minisig
```

Assets include SHA-256 checksums and a Minisign signature. The plugin embeds the Minisign public key and verifies both signature and checksum before extraction. An Apple Silicon Mac running Omarchy uses Linux `aarch64`, not macOS, and therefore receives the `aarch64-linux` asset.

### 11.2 CI/CD

CI must at minimum:

1. format, lint, and test Rust code;
2. build Linux `x86_64` and `aarch64` binaries;
3. package the fixed archive names and generate checksums;
4. publish assets for version tags;
5. sign `checksums.txt` using a Minisign publication key available only through secure CI secrets; and
6. support a controlled manually triggered prerelease.

### 11.3 Engine updates

The plugin may report an available update but requires user confirmation before download or replacement. Installation retains the previous version until the replacement is validated and offers straightforward rollback on failure.

## 12. Omarchy integration

The repository contains at minimum:

```text
manifest.json
BarWidget.qml
Panel.qml
README.md
LICENSE
```

The manifest uses `io.github.toorop.voxtype-personas` and declares only the `bar-widget` kind, with `BarWidget.qml` as entry point. The panel is loaded by that widget, not declared as another plugin.

The plugin is validated with `omarchy plugin validate`; QML is checked with the applicable `qmllint` setup and Omarchy shell imports. It never modifies `/usr/share/omarchy/`.

## 13. Preservation of the existing installation

The current machine already has:

- Voxtype dictation in paste mode;
- a Hyprland push-to-talk binding;
- a PipeWire global-mute relay that restores the initial state on key release; and
- `duck_media = false` in Voxtype, so Chromium's individual volume is not modified.

VoxTypePersonas must not alter the binding or audio relay. Its integration is limited to the Voxtype post-processing command, after detection, backup, and explicit confirmation if an existing configuration is changed.

## 14. Acceptance criteria

1. The plugin installs and renders as an Omarchy bar widget.
2. Clicking it opens a compact profile selector anchored to the widget button.
3. **Settings…** opens a larger panel anchored to that same button, with no separate application.
4. Selecting a Ready profile persists the choice and it is used by the next dictation without restarting Voxtype.
5. `Raw` returns stdin exactly and makes no network call.
6. Draft profiles are visible and editable but cannot be activated; `Raw` is the sole active default and remains always Ready.
7. A provider can be configured, tested, and deleted; its secret never appears in logs, configuration, or portable profile export.
8. Portable profiles are validatable Markdown with YAML front matter; unsafe or invalid imports are rejected atomically.
9. Any provider, network, timeout, or response-format failure returns raw text instead of blocking dictation.
10. The plugin adds no lock or queue to the existing sequential dictation cycle.
11. First engine download requires explicit confirmation and verifies the checksum and Minisign signature before installing the appropriate binary.
12. The plugin preserves the existing mute relay and dictation bindings.

## 15. Implementation decisions left open

- Cache and filtering details for provider model lists when provider metadata is incomplete.
- Exact wording of shipped example prompts, within the stated language, safety, and voice-preservation rules.
- Internal implementation of Minisign verification, without relaxing the signature requirement.
- The exact CLI surface for profile catalog import/export, provided it preserves the validation and atomicity guarantees above.

## References

- [Omarchy — Develop a Custom Plugin](https://plugins.omarchy.org/develop.html)
- [Omarchy — Publish a Plugin](https://plugins.omarchy.org/publish.html)
