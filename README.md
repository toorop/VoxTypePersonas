# VoxTypePersonas

VoxTypePersonas is an Omarchy plugin for selecting the post-processing persona used by Voxtype dictation. It pairs an Omarchy bar widget with a Rust engine named `voxtype-personas`.

The engine receives a local Whisper transcription on standard input and writes only the final text to standard output. A persona may pass the text through unchanged or send it to a configured local or remote LLM provider. If processing fails, the engine returns the original transcription so dictation remains usable.

## Status

Phase 8 is complete. The engine supports local Ollama, OpenAI-compatible providers, and native Anthropic and Gemini adapters with keys read only from Secret Service.

## Scope

- Omarchy on Linux only.
- An anchored Omarchy `bar-widget` for quick persona selection and settings.
- A Rust engine distributed separately through signed GitHub Release assets.
- Linux `x86_64` and `aarch64` engine releases.
- The mandatory Raw profile plus an importable, non-activatable Example draft profile.
- A versioned `profiles/` catalog of portable Markdown profiles that users can edit externally and contribute through pull requests.
- Local Ollama and configured remote providers.
- Secret Service/keyring storage for API keys.

## Non-goals for v1

- A standalone desktop application.
- Distribution-specific packages.
- Cloud synchronization, persona sharing, or telemetry.
- Streaming text to a provider.
- Changes to existing Hyprland dictation bindings or PipeWire audio handling.

## Privacy and safety principles

- The Raw persona never makes a network request.
- Dictated text, prompts, and secrets are not written to logs by default.
- API keys are stored only in the system keyring; configuration files store secret references, never key values.
- Every provider, timeout, network, parsing, or invalid-output failure falls back to the original transcription.
- The plugin asks for confirmation before it downloads, installs, or updates an engine binary.

## Secret storage

VoxTypePersonas requires the Linux Secret Service API for API keys and has no file-based fallback. The engine uses the stable `org.voxtype-personas/provider/<provider-id>` reference convention; only the reference belongs in configuration. The selected D-Bus adapter is the Rust `secret-service` client with its blocking API and encrypted session support; it will be connected when provider configuration begins to use keys.

## Profile readiness and shared profiles

Raw is the only profile active on first use. The bundled Example profile is a generic draft intended for learning and adaptation, not a prompt optimized for a specific provider or model.

A profile may be saved while incomplete. Draft profiles can be viewed, edited, duplicated, imported, exported, and loaded by the application, but cannot be activated. A profile becomes Ready only when it has a prompt, a compatible provider, an explicit model selection, and a verified local key when that provider requires one. The user interface must disable activation for drafts and explain what is missing.

Portable profiles live in [profiles/](profiles/README.md). They use Markdown with YAML front matter so they can be edited and tested outside VoxTypePersonas. The portable format includes public provider/model metadata, limits, output policy, and compatibility notes; it excludes API keys and local Secret Service references.

## Planned repository layout

```text
.
├── engine/                 Rust package for the voxtype-personas binary
├── plugin/                 Omarchy manifest and QML bar-widget sources
├── tests/
│   └── fixtures/           Non-sensitive deterministic test inputs and outputs
├── scripts/                Development and release helper scripts
├── .github/workflows/      Continuous-integration and release workflows
├── docs/                   Product notes and authoritative v1 specification
├── AGENTS.md               Persistent project working agreement for Codex
├── ROADMAP.md              Approved, incremental delivery plan
├── state.md                Continuity record for completed work
├── CONTRIBUTING.md         Contribution and safety conventions
├── LICENSE                 MIT license
└── README.md               Project overview and user-facing documentation
```

The directories above are populated only when their corresponding roadmap phase begins.

## Development prerequisites

Phase 1 requires a current Rust toolchain with Cargo. Future phases will additionally document the exact Omarchy/Quickshell validation tools, Secret Service requirements, Minisign tooling, and CI environment. No development dependency is installed automatically by this repository.

## Available engine commands

The following configuration commands are available during the current implementation stage:

```text
voxtype-personas version
voxtype-personas config validate
voxtype-personas config validate --defaults
voxtype-personas config validate --file <path>
voxtype-personas profiles list
voxtype-personas profiles set-active <id>
voxtype-personas process
voxtype-personas process --profile raw
```

On first use, configuration is created at the XDG path `~/.config/voxtype-personas/config.toml` with owner-only file permissions. The default active persona is Raw. Provider processing is introduced in later roadmap phases.

`process` resolves its selected profile once when the command begins. Raw copies standard input to standard output byte-for-byte and produces no diagnostics. A recoverable provider failure preserves the raw text on standard output, returns success, and emits only a non-sensitive diagnostic on standard error.

Provider output must be non-empty and free of NUL characters. By default, obvious preambles and Markdown framing are also rejected; the per-profile output policy can explicitly allow either form when needed.

## Development plan

The detailed implementation sequence is maintained in [ROADMAP.md](ROADMAP.md). Work proceeds one approved phase at a time. The current project status and the next proposed step are recorded in [state.md](state.md).

## Documentation

- [Product notes](docs/VoxTypePersonas.md)
- [Functional and technical specification](docs/specs.md)

## License

VoxTypePersonas is licensed under the [MIT License](LICENSE).
