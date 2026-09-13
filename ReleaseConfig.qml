import QtQml

QtObject {
    // Stable releases only. Prereleases are never selected automatically.
    readonly property string stableReleaseUrl:
        "https://github.com/toorop/VoxTypePersonas/releases/latest"
    readonly property string minisignPublicKey:
        "RWRMNSNAjT6hxxXWoMuQALOi+ZiKdgPR7e1TmF/6h7HORyLCdHdAOt3J"
    readonly property string x86_64Archive:
        "voxtype-personas-x86_64-linux.tar.gz"
    readonly property string aarch64Archive:
        "voxtype-personas-aarch64-linux.tar.gz"
    readonly property string checksumsFile: "checksums.txt"
    readonly property string checksumsSignatureFile: "checksums.txt.minisig"
}
