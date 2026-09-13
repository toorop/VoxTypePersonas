import QtQml
import Quickshell.Io

QtObject {
    id: root

    property string publicKeyPath: ""
    property string errorMessage: ""

    signal verified()
    signal verificationFailed()

    function verify(manifestPath, signaturePath) {
        if (root.publicKeyPath === "" || manifestPath === "" || signaturePath === "") {
            root.errorMessage = "Release verification could not start."
            root.verificationFailed()
            return
        }

        root.errorMessage = ""
        verifier.command = [
            "gpgv",
            "--keyring", root.publicKeyPath,
            signaturePath,
            manifestPath
        ]
        verifier.running = true
    }

    property Process verifier: Process {
        id: verifier

        running: false

        onExited: function(exitCode, exitStatus) {
            if (exitCode === 0) {
                root.verified()
                return
            }

            root.errorMessage = "The release signature could not be verified."
            root.verificationFailed()
        }
    }
}
