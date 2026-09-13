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

`checksums.txt` is a SHA-256 manifest in the standard `sha256sum` format. It must list both archive base names without directory components. `checksums.txt.asc` is a detached OpenPGP signature verified locally with `gpgv` and the embedded armored release public key in `assets/keys/release-signing.asc`.

## Archive contents

Each archive must contain exactly one regular executable file at its root:

```text
voxtype-personas
```

It must contain no symbolic links, hard links, absolute paths, parent-directory paths, or additional files. The staged binary must return a strict `MAJOR.MINOR.PATCH` value from `voxtype-personas version`, and that value must equal the release tag after its optional leading `v` is removed.

## Installer sequence

After explicit user confirmation, the installer creates a private temporary directory, downloads the selected archive and both verification assets, verifies the detached OpenPGP signature, then verifies only the selected archive against the trusted checksum manifest. It validates the archive layout before extraction, tests the staged binary, and only then atomically promotes it while retaining the previous working binary for rollback.
