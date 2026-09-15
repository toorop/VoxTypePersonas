import QtQml
import Quickshell.Io

QtObject {
    id: root

    property string publicKeyPath: ""
    property string installRoot: ""
    property string engineExecutable: ""
    property string architecture: ""
    property string releaseTag: ""
    property bool allowPrereleaseTest: false
    property string downloadBaseUrl: ""
    property string stage: ""
    property string downloadDirectory: ""
    property string stagingDirectory: ""
    property string commandOutput: ""
    property bool replacingExisting: false

    readonly property bool running: worker.running || cleanup.running

    readonly property string archiveName: "voxtype-personas-" + architecture + "-linux.tar.gz"
    readonly property string archivePath: downloadDirectory + "/" + archiveName
    readonly property string checksumPath: downloadDirectory + "/checksums.txt"
    readonly property string signaturePath: downloadDirectory + "/checksums.txt.asc"
    readonly property string stagedBinary: stagingDirectory + "/voxtype-personas"
    readonly property string expectedVersion: {
        var version = releaseTag.replace(/^v/, "");
        return allowPrereleaseTest ? version.split("-")[0] : version;
    }

    signal progress(string stage)
    signal succeeded
    signal failed(string message)

    function start() {
        if (worker.running || cleanup.running)
            return;
        if (publicKeyPath === "" || installRoot === "" || engineExecutable === "" || !/^(x86_64|aarch64)$/.test(architecture) || !/^v?[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?$/.test(releaseTag) || (!allowPrereleaseTest && releaseTag.includes("-")) || !/^https:\/\/github\.com\/toorop\/VoxTypePersonas\/releases\/download\/.+\/$/.test(downloadBaseUrl)) {
            root.fail("The published release information is invalid.");
            return;
        }

        root.downloadDirectory = "";
        root.stagingDirectory = "";
        root.replacingExisting = false;
        root.run("preparing", ["mktemp", "-d", "/tmp/voxtype-personas.XXXXXX"]);
    }

    function run(nextStage, nextCommand) {
        root.stage = nextStage;
        root.progress(nextStage);
        root.commandOutput = "";
        worker.command = nextCommand;
        worker.running = true;
    }

    function fail(message) {
        root.stage = "";
        root.cleanupAfterFailure = message;
        root.cleanupTemporaryFiles();
    }

    property string cleanupAfterFailure: ""

    function cleanupTemporaryFiles() {
        var paths = [];
        if (downloadDirectory.startsWith("/tmp/voxtype-personas."))
            paths.push(downloadDirectory);
        if (stagingDirectory.startsWith(installRoot + "/bin/.staging."))
            paths.push(stagingDirectory);

        if (paths.length === 0) {
            root.finishCleanup();
            return;
        }

        cleanup.command = ["rm", "-rf"].concat(paths);
        cleanup.running = true;
    }

    function finishCleanup() {
        if (cleanupAfterFailure !== "") {
            var message = cleanupAfterFailure;
            cleanupAfterFailure = "";
            root.failed(message);
            return;
        }

        root.succeeded();
    }

    function manifestContainsExpectedChecksum() {
        var escapedName = archiveName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        var match = String(commandOutput).match(new RegExp("^([0-9a-fA-F]{64})\\s+\\*?" + escapedName + "\\s*$", "m"));
        if (!match)
            return false;

        root.expectedChecksum = match[1].toLowerCase();
        return true;
    }

    property string expectedChecksum: ""

    function archiveLayoutIsSafe() {
        var names = String(commandOutput).trim().split("\n");
        return names.length === 1 && names[0] === "voxtype-personas";
    }

    function archiveContainsOneRegularFile() {
        var entries = String(commandOutput).trim().split("\n");
        return entries.length === 1 && entries[0].startsWith("-") && entries[0].endsWith(" voxtype-personas");
    }

    function handleWorkerExit(exitCode) {
        if (root.stage === "preserving") {
            root.replacingExisting = exitCode === 0;
            if (root.replacingExisting)
                root.run("promoting", ["mv", "--backup=numbered", "--", engineExecutable, engineExecutable + ".previous"]);
            else
                root.run("promoting-new", ["mv", "--", stagedBinary, engineExecutable]);
            return;
        }

        if (exitCode !== 0) {
            if (root.stage === "promoting" || (root.stage === "promoting-new" && root.replacingExisting)) {
                root.run("restoring", ["mv", "--", engineExecutable + ".previous", engineExecutable]);
                return;
            }
            if (root.stage === "restoring") {
                root.fail("The engine update failed and could not be restored automatically.");
                return;
            }
            root.fail(root.failureForStage(root.stage));
            return;
        }

        if (root.stage === "preparing") {
            var directory = String(commandOutput).trim();
            if (!directory.startsWith("/tmp/voxtype-personas.")) {
                root.fail("A private download directory could not be created.");
                return;
            }
            root.downloadDirectory = directory;
            root.run("downloading", ["curl", "--fail", "--location", "--silent", "--show-error", "--proto", "=https", "--tlsv1.2", "--max-time", "60", "--output", archivePath, downloadBaseUrl + archiveName]);
            return;
        }
        if (root.stage === "downloading") {
            root.run("downloading-manifest", ["curl", "--fail", "--location", "--silent", "--show-error", "--proto", "=https", "--tlsv1.2", "--max-time", "60", "--output", checksumPath, downloadBaseUrl + "checksums.txt"]);
            return;
        }
        if (root.stage === "downloading-manifest") {
            root.run("downloading-signature", ["curl", "--fail", "--location", "--silent", "--show-error", "--proto", "=https", "--tlsv1.2", "--max-time", "60", "--output", signaturePath, downloadBaseUrl + "checksums.txt.asc"]);
            return;
        }
        if (root.stage === "downloading-signature") {
            root.run("signature", ["gpgv", "--keyring", publicKeyPath, signaturePath, checksumPath]);
            return;
        }
        if (root.stage === "signature") {
            root.run("reading-manifest", ["cat", checksumPath]);
            return;
        }
        if (root.stage === "reading-manifest") {
            if (!root.manifestContainsExpectedChecksum()) {
                root.fail("The signed checksum manifest is invalid.");
                return;
            }
            root.run("checksum", ["sha256sum", archivePath]);
            return;
        }
        if (root.stage === "checksum") {
            var checksumParts = String(commandOutput).trim().split(/\s+/);
            if (checksumParts.length < 1 || checksumParts[0].toLowerCase() !== root.expectedChecksum) {
                root.fail("The downloaded engine checksum is invalid.");
                return;
            }
            root.run("validating", ["tar", "-tzf", archivePath]);
            return;
        }
        if (root.stage === "validating") {
            if (!root.archiveLayoutIsSafe()) {
                root.fail("The engine archive has an unsafe layout.");
                return;
            }
            root.run("validating-type", ["tar", "-tvzf", archivePath]);
            return;
        }
        if (root.stage === "validating-type") {
            if (!root.archiveContainsOneRegularFile()) {
                root.fail("The engine archive has an unsafe layout.");
                return;
            }
            root.run("installing", ["mkdir", "-p", "-m", "700", installRoot + "/bin"]);
            return;
        }
        if (root.stage === "installing") {
            root.run("staging", ["mktemp", "-d", installRoot + "/bin/.staging.XXXXXX"]);
            return;
        }
        if (root.stage === "staging") {
            var staging = String(commandOutput).trim();
            if (!staging.startsWith(installRoot + "/bin/.staging.")) {
                root.fail("A secure staging directory could not be created.");
                return;
            }
            root.stagingDirectory = staging;
            root.run("extracting", ["tar", "-xzf", archivePath, "-C", stagingDirectory, "--no-same-owner", "--no-same-permissions"]);
            return;
        }
        if (root.stage === "extracting") {
            root.run("validating-engine", ["chmod", "700", stagedBinary]);
            return;
        }
        if (root.stage === "validating-engine") {
            root.run("validating-version", [stagedBinary, "version"]);
            return;
        }
        if (root.stage === "validating-version") {
            if (String(commandOutput).trim() !== expectedVersion) {
                root.fail("The staged engine version does not match the release.");
                return;
            }
            root.run("preserving", ["test", "-f", engineExecutable]);
            return;
        }
        if (root.stage === "promoting") {
            root.run("promoting-new", ["mv", "--", stagedBinary, engineExecutable]);
            return;
        }
        if (root.stage === "promoting-new") {
            root.stage = "";
            root.cleanupTemporaryFiles();
            return;
        }
        if (root.stage === "restoring") {
            root.fail("The engine update failed; the previous engine was restored.");
        }
    }

    function failureForStage(failedStage) {
        if (failedStage.startsWith("downloading"))
            return "The engine download failed.";
        if (failedStage === "signature")
            return "The release signature could not be verified.";
        if (failedStage === "checksum")
            return "The downloaded engine checksum is invalid.";
        if (failedStage.startsWith("validating") || failedStage === "extracting")
            return "The downloaded engine could not be validated.";
        if (failedStage === "installing" || failedStage === "staging")
            return "The engine could not be installed.";
        return "The engine installation could not be completed.";
    }

    property Process worker: Process {
        id: worker
        running: false

        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: root.commandOutput = text
        }

        onExited: function (exitCode) {
            root.handleWorkerExit(exitCode);
        }
    }

    property Process cleanup: Process {
        id: cleanup
        running: false
        onExited: root.finishCleanup()
    }
}
