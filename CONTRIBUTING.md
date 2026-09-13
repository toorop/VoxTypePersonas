# Contributing to VoxTypePersonas

## Project conventions

- Write all documentation and code comments in English.
- Keep changes focused and aligned with the currently approved roadmap phase.
- Do not introduce features from later phases without explicit review.
- Do not commit credentials, API keys, private signing keys, dictated text, or production configuration.
- Preserve the existing Hyprland dictation binding and PipeWire mute relay; VoxTypePersonas integrates only through Voxtype's post-processing command.

## Development workflow

1. Review `AGENTS.md`, `ROADMAP.md`, and `state.md` before starting work.
2. Work on one small, approved step at a time.
3. Add or update relevant tests when implementation begins.
4. Record completed work, validation, decisions, and the next step in `state.md`.
5. Request explicit approval before creating commits or pushing changes.

## Security requirements

- Store API keys only in Secret Service/keyring integration.
- Keep secret references, not secret values, in configuration files.
- Never log dictated text, system prompts, secrets, or authorization headers.
- Treat a provider failure as recoverable and preserve the original transcription.
- Verify release checksums and detached OpenPGP signatures before engine installation.
