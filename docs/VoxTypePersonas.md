# VoxTypePersonas — Project Background

The authoritative v1 contract is [specs.md](specs.md). This document records the product intent and host-installation constraints that motivated it.

## Intent

VoxTypePersonas is an Omarchy-only project for selecting the post-processing profile used by Voxtype dictation. It improves a local Whisper transcription without making every result formal, academic, or unlike the speaker.

The product consists of one anchored Omarchy `bar-widget` plugin and the Rust `voxtype-personas` engine. The plugin presents and configures profiles; the engine is the stable command invoked by Voxtype and is the only component that processes dictated text.

## Current profile model

`Raw` is the mandatory built-in profile and the sole active profile on first use. It copies standard input to standard output exactly and never takes a network-capable path.

`profiles/example.md` is the only shipped non-Raw profile. It is a generic portable Draft, intentionally not a provider or model recommendation. A Draft may be viewed, edited, duplicated, imported, or exported, but it cannot become Active until it has a prompt, compatible provider, explicit model, and a verified local Secret Service key when required.

Portable profiles use Markdown with YAML front matter. They exclude API keys, Secret Service references, dictated text, and other personal configuration. The catalog is human-editable and may be contributed through pull requests. See [profiles/README.md](../profiles/README.md) for its format and CLI workflow.

## Processing contract

Voxtype invokes one command:

```text
voxtype-personas process
```

The engine resolves the selected profile once for each invocation. It reads dictated text from standard input and writes only final pasteable text to standard output. A provider is optional. On a recoverable provider, network, authentication, timeout, parsing, or response-validation failure, the engine returns the raw transcription instead.

System prompts and dictated text are separate provider messages. Diagnostics never include dictated text, prompts, keys, or authorization headers.

## Omarchy interface

The existing Quickshell process loads the plugin's `BarWidget.qml`, which owns its compact selector and its larger anchored settings panel. No separate application or second Quickshell process is launched.

The QML interface can call the engine CLI for state and changes. In particular, the CLI provides profile listing, activation, portable-profile validation, Draft-only import, safe export, and provider testing. Keeping persistence, validation, Secret Service access, and provider processing in the engine avoids duplicating security-sensitive logic in the widget.

## Distribution and secrets

The plugin and engine are one product but separate distributable components. The plugin will install a confirmed release-engine download only after detached OpenPGP and SHA-256 verification. v1 supports Linux `x86_64` and `aarch64` releases.

API keys are stored only in Linux Secret Service. Local configuration contains opaque references, never keys. A remote profile cannot be activated until the referenced key verifies successfully. There is no file-based secret fallback.

## Existing host constraints

The current host already has Voxtype paste-mode dictation, a Hyprland push-to-talk binding, and a PipeWire global-mute relay that restores the prior state when the key is released. `duck_media = false` is set in Voxtype to avoid changing Chromium's individual volume.

VoxTypePersonas must not modify or restart these audio or binding mechanisms. Its future integration is limited to Voxtype's post-processing command, after detecting an existing command, creating a backup, and receiving explicit confirmation.

## References

- [Omarchy — Develop a Custom Plugin](https://plugins.omarchy.org/develop.html)
- [Omarchy — Publish a Plugin](https://plugins.omarchy.org/publish.html)
