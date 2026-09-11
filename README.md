# VoxTypePersonas

VoxTypePersonas is an Omarchy plugin for selecting the post-processing persona used by Voxtype dictation. It pairs an Omarchy bar widget with a Rust engine named `voxtype-personas`.

The engine receives a local Whisper transcription on standard input and writes only the final text to standard output. A persona may pass the text through unchanged or send it to a configured local or remote LLM provider. If processing fails, the engine returns the original transcription so dictation remains usable.

## Status

Phase 1 of the roadmap is complete: the Rust workspace, configuration contract, path model, and CLI skeleton are established. No provider adapter, plugin, or Voxtype integration has been implemented yet.

## Scope

- Omarchy on Linux only.
- An anchored Omarchy `bar-widget` for quick persona selection and settings.
- A Rust engine distributed separately through signed GitHub Release assets.
- Linux `x86_64` and `aarch64` engine releases.
- Built-in Raw, Chat, Email, Technical, and Meeting / Notes personas.
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

## Development plan

The detailed implementation sequence is maintained in [ROADMAP.md](ROADMAP.md). Work proceeds one approved phase at a time. The current project status and the next proposed step are recorded in [state.md](state.md).

## Documentation

- [Product notes](docs/VoxTypePersonas.md)
- [Functional and technical specification](docs/specs.md)

## License

VoxTypePersonas is licensed under the [MIT License](LICENSE).
