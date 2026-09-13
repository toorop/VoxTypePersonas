# Engine Release Format

This contract defines the only engine assets accepted by the v1 plugin installer.

## Stable release channel

The installer uses the stable GitHub Releases channel for `toorop/VoxTypePersonas`. Drafts and prereleases are never selected. A stable release tag must be `vMAJOR.MINOR.PATCH`; the leading `v` is removed only for comparison with the engine's `version` output.

## Required assets

Every stable release must include exactly these verification assets and both architecture archives:

```text
voxtype-personas-x86_64-linux.tar.gz
voxtype-personas-aarch64-linux.tar.gz
checksums.txt
checksums.txt.asc
```

`checksums.txt` is a SHA-256 manifest in the standard `sha256sum` format. It must list both archive base names without directory components. `checksums.txt.asc` is a detached OpenPGP signature verified locally with `gpgv` and the embedded release public keyring in `assets/keys/release-signing.gpg`.

## Archive contents

Each archive must contain exactly one regular executable file at its root:

```text
voxtype-personas
```

It must contain no symbolic links, hard links, absolute paths, parent-directory paths, or additional files. The staged binary must return a strict `MAJOR.MINOR.PATCH` value from `voxtype-personas version`, and that value must equal the release tag after its optional leading `v` is removed.

## Installer sequence

After explicit user confirmation, the installer creates a private temporary directory, downloads the selected archive and both verification assets, verifies the detached OpenPGP signature, then verifies only the selected archive against the trusted checksum manifest. It validates the archive layout before extraction, tests the staged binary, and only then atomically promotes it while retaining the previous working binary for rollback.

## Developer prerelease testing

Prereleases are never selected by the normal installer. A developer can opt into one published, signed prerelease by setting `VOXTYPE_PERSONAS_TEST_RELEASE_TAG` in Hyprland's runtime environment before restarting the Omarchy shell. The tag must match the prerelease tag exactly.

On the Lua-based Hyprland configuration used by Omarchy, use `hyprctl eval`; legacy `hyprctl keyword env` syntax is not supported:

```bash
hyprctl eval 'hl.env("VOXTYPE_PERSONAS_TEST_RELEASE_TAG", "vMAJOR.MINOR.PATCH-test.N")'
omarchy restart shell
```

The panel should then offer `Install engine`. Selecting it once starts the complete verified transaction. A successful test shows the Raw profile, and the managed binary reports its expected base version:

```bash
~/.local/share/voxtype-personas/bin/voxtype-personas version
```

Clear the runtime value and restart the shell after testing. The normal stable-only channel is then restored:

```bash
hyprctl eval 'hl.env("VOXTYPE_PERSONAS_TEST_RELEASE_TAG", "")'
omarchy restart shell
systemctl --user unset-environment VOXTYPE_PERSONAS_TEST_RELEASE_TAG
```

This opt-in is a developer test hook only. Do not add a prerelease tag to persistent user configuration, release documentation for end users, or the production stable channel.
