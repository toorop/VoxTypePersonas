import QtQuick
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

BarWidget {
    id: root

    moduleName: "io.github.toorop.voxtype-personas"

    // A failed process start does not emit Process.exited. Start in the safe
    // fallback state so an unavailable release engine never leaves the bar in
    // a permanent loading state.
    property string profileLabel: "Unavailable"
    property string profileError: "The VoxTypePersonas engine or configuration is unavailable."
    property var profileEntries: []
    property bool engineAvailable: false
    // Starting from the missing state also handles a Process that cannot be
    // started: Quickshell does not guarantee an exited signal in that case.
    property string engineStatus: "missing"
    property string engineVersion: ""
    property string hostArchitecture: ""
    property string architectureOutput: ""
    property string versionOutput: ""
    property string installationStage: ""
    property string installationError: ""
    // The OpenPGP public key and installation transaction must be embedded
    // before the UI can offer a safe installation.
    property bool installationTransactionReady: true
    property string releaseMetadata: ""
    property string selectedReleaseTag: ""
    property string selectedReleaseDownloadUrl: ""
    property bool updateAvailable: false
    property string releaseCheckMode: ""

    readonly property string releaseApiUrl: {
        var testTag = Quickshell.env("VOXTYPE_PERSONAS_TEST_RELEASE_TAG");
        if (testTag)
            return "https://api.github.com/repos/toorop/VoxTypePersonas/releases/tags/" + testTag;

        return "https://api.github.com/repos/toorop/VoxTypePersonas/releases/latest";
    }

    readonly property var installationSteps: [
        {
            id: "preparing",
            label: "Preparing secure download"
        },
        {
            id: "metadata",
            label: "Checking release information"
        },
        {
            id: "downloading",
            label: "Downloading engine and verification files"
        },
        {
            id: "signature",
            label: "Verifying release signature"
        },
        {
            id: "checksum",
            label: "Verifying archive checksum"
        },
        {
            id: "validating",
            label: "Extracting and validating engine"
        },
        {
            id: "installing",
            label: "Installing engine"
        }
    ]

    readonly property string engineExecutable: {
        var dataHome = Quickshell.env("XDG_DATA_HOME");
        if (!dataHome)
            dataHome = Quickshell.env("HOME") + "/.local/share";

        return dataHome + "/voxtype-personas/bin/voxtype-personas";
    }

    readonly property string engineInstallRoot: {
        var dataHome = Quickshell.env("XDG_DATA_HOME");
        if (!dataHome)
            dataHome = Quickshell.env("HOME") + "/.local/share";

        return dataHome + "/voxtype-personas";
    }

    readonly property string releasePublicKeyPath: {
        var url = String(Qt.resolvedUrl("assets/keys/release-signing.gpg"));
        return decodeURIComponent(url.replace(/^file:\/\//, ""));
    }

    readonly property bool opened: panelLoader.item ? panelLoader.item.opened === true : false
    readonly property bool popoutSwitchClosing: panelLoader.item ? panelLoader.item.popoutSwitchClosing === true : false

    function open() {
        if (panelLoader.item)
            panelLoader.item.open();
    }

    function close() {
        if (panelLoader.item)
            panelLoader.item.close();
    }

    function toggle() {
        if (panelLoader.item)
            panelLoader.item.toggle();
    }

    function closeForPopoutSwitch() {
        if (panelLoader.item)
            panelLoader.item.closeForPopoutSwitch();
    }

    function injectPanel() {
        if (!panelLoader.item)
            return;
        panelLoader.item.bar = root.bar;
        panelLoader.item.anchorItem = button;
        panelLoader.item.hostWidget = root;
        panelLoader.item.profileEntries = root.profileEntries;
        panelLoader.item.profileError = root.profileError;
        panelLoader.item.engineAvailable = root.engineAvailable;
        panelLoader.item.engineStatus = root.engineStatus;
        panelLoader.item.engineVersion = root.engineVersion;
        panelLoader.item.installationStage = root.installationStage;
        panelLoader.item.installationError = root.installationError;
        panelLoader.item.installationSteps = root.installationSteps;
        panelLoader.item.installationTransactionReady = root.installationTransactionReady;
        panelLoader.item.updateAvailable = root.updateAvailable;
    }

    function refreshProfile() {
        if (root.engineAvailable && !profileProcess.running)
            profileProcess.running = true;
    }

    function resetUnavailableEngine(status) {
        root.profileLabel = "Unavailable";
        root.profileError = "The VoxTypePersonas engine is unavailable.";
        root.profileEntries = [];
        root.engineAvailable = false;
        root.engineVersion = "";
        root.engineStatus = status;
        root.injectPanel();
    }

    function beginEngineInstallation() {
        if (releaseProbe.running || engineInstaller.running)
            return;
        root.installationStage = "metadata";
        root.installationError = "";
        root.releaseCheckMode = "install";
        releaseProbe.running = true;
        root.injectPanel();
    }

    function checkForUpdate() {
        if (releaseProbe.running)
            return;
        root.releaseCheckMode = "update";
        releaseProbe.running = true;
    }

    function isNewerVersion(candidate, installed) {
        var next = candidate.replace(/^v/, "").split(".");
        var current = installed.replace(/^v/, "").split(".");
        for (var index = 0; index < 3; index++) {
            var difference = Number(next[index]) - Number(current[index]);
            if (difference !== 0)
                return difference > 0;
        }
        return false;
    }

    function clearEngineInstallationStatus() {
        root.installationStage = "";
        root.installationError = "";
        root.injectPanel();
    }

    function validReleaseTag(tag) {
        var testTag = Quickshell.env("VOXTYPE_PERSONAS_TEST_RELEASE_TAG");
        if (testTag)
            return tag === testTag && /^v?[0-9]+\.[0-9]+\.[0-9]+-[0-9A-Za-z.-]+$/.test(tag);

        return /^v?[0-9]+\.[0-9]+\.[0-9]+$/.test(tag);
    }

    function installationStageFor(stage) {
        if (stage.startsWith("downloading"))
            return "downloading";
        if (stage === "signature")
            return "signature";
        if (stage === "checksum" || stage === "reading-manifest")
            return "checksum";
        if (stage === "installing" || stage === "staging" || stage.startsWith("promoting") || stage === "preserving")
            return "installing";
        if (stage.startsWith("validating") || stage === "extracting")
            return "validating";
        return stage;
    }

    GpgVerifier {
        id: releaseVerifier

        publicKeyPath: root.releasePublicKeyPath

        onVerificationFailed: {
            root.installationError = errorMessage;
            root.injectPanel();
        }
    }

    EngineInstaller {
        id: engineInstaller

        publicKeyPath: root.releasePublicKeyPath
        installRoot: root.engineInstallRoot
        engineExecutable: root.engineExecutable
        architecture: root.hostArchitecture
        releaseTag: root.selectedReleaseTag
        allowPrereleaseTest: Boolean(Quickshell.env("VOXTYPE_PERSONAS_TEST_RELEASE_TAG"))
        downloadBaseUrl: root.selectedReleaseDownloadUrl

        onProgress: function (stage) {
            root.installationStage = root.installationStageFor(stage);
            root.injectPanel();
        }
        onSucceeded: {
            root.installationStage = "";
            root.installationError = "";
            versionProcess.command = [root.engineExecutable, "version"];
            versionProcess.running = true;
            root.injectPanel();
        }
        onFailed: function (message) {
            root.installationStage = "";
            root.installationError = message;
            root.injectPanel();
        }
    }

    Process {
        id: releaseProbe

        command: ["curl", "--fail", "--location", "--silent", "--show-error", "--proto", "=https", "--tlsv1.2", "--max-time", "15", root.releaseApiUrl]
        running: false

        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: root.releaseMetadata = text
        }

        onExited: function (exitCode, exitStatus) {
            if (exitCode !== 0) {
                if (root.releaseCheckMode === "update") {
                    root.releaseCheckMode = "";
                    return;
                }
                root.installationStage = "";
                root.installationError = "No stable engine release is available yet.";
                root.injectPanel();
                return;
            }

            var match = root.releaseMetadata.match(/"tag_name"\s*:\s*"([^"]+)"/);
            if (!match || !root.validReleaseTag(match[1])) {
                if (root.releaseCheckMode === "update") {
                    root.releaseCheckMode = "";
                    return;
                }
                root.installationStage = "";
                root.installationError = "The published release information is invalid.";
                root.injectPanel();
                return;
            }

            if (root.releaseCheckMode === "update") {
                root.updateAvailable = root.isNewerVersion(match[1], root.engineVersion);
                root.releaseCheckMode = "";
                if (root.updateAvailable && !updateNotification.running)
                    updateNotification.running = true;
                root.injectPanel();
                return;
            }

            root.selectedReleaseTag = match[1];
            root.selectedReleaseDownloadUrl = "https://github.com/toorop/VoxTypePersonas/releases/download/" + root.selectedReleaseTag + "/";
            root.releaseCheckMode = "";
            engineInstaller.start();
        }
    }

    function detectArchitecture(output) {
        var machine = String(output || "").trim();
        if (machine === "x86_64" || machine === "aarch64") {
            root.hostArchitecture = machine;
            versionProcess.command = [root.engineExecutable, "version"];
            versionProcess.running = true;
            return;
        }

        root.resetUnavailableEngine("unsupported-architecture");
    }

    function detectInstalledEngine(output) {
        var version = String(output || "").trim();
        if (!/^v?[0-9]+\.[0-9]+\.[0-9]+$/.test(version)) {
            root.resetUnavailableEngine("invalid");
            return;
        }

        root.engineVersion = version.startsWith("v") ? version.slice(1) : version;
        root.engineStatus = "installed";
        root.engineAvailable = true;
        root.refreshProfile();
        root.checkForUpdate();
        root.injectPanel();
    }

    function applyProfileList(output) {
        // Preserve each row's two-character active marker. Trimming the
        // complete output would strip the marker from the first inactive row.
        var lines = String(output || "").split("\n");
        var entries = [];
        var hasActiveProfile = false;

        root.profileError = "";
        root.engineAvailable = true;

        for (var index = 0; index < lines.length; index++) {
            var line = lines[index];
            if (line.trim().length === 0)
                continue;
            var active = line.startsWith("* ");
            var fields = line.slice(2).split("\t");
            if (fields.length < 3 || fields[0].trim() === "" || fields[1].trim() === "")
                continue;
            entries.push({
                id: fields[0].trim(),
                name: fields[1].trim(),
                state: fields[2].trim(),
                promptId: fields.length >= 4 ? fields[3].trim() : "",
                active: active
            });

            if (!active)
                continue;
            root.profileLabel = fields[1].trim();
            root.profileError = "";
            hasActiveProfile = true;
        }

        root.profileEntries = entries;
        if (!hasActiveProfile) {
            root.profileLabel = "Unavailable";
            root.profileError = "No active profile is available.";
        }
        root.injectPanel();
    }

    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight

    onBarChanged: injectPanel()
    Component.onCompleted: architectureProcess.running = true

    Process {
        id: architectureProcess

        command: ["uname", "-m"]
        running: false

        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: root.architectureOutput = text
        }

        onExited: function (exitCode, exitStatus) {
            if (exitCode !== 0) {
                root.resetUnavailableEngine("unsupported-architecture");
                return;
            }

            root.detectArchitecture(root.architectureOutput);
        }
    }

    Process {
        id: versionProcess

        running: false

        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: root.versionOutput = text
        }

        onExited: function (exitCode, exitStatus) {
            if (exitCode !== 0) {
                root.resetUnavailableEngine("invalid");
                return;
            }

            root.detectInstalledEngine(root.versionOutput);
        }
    }

    Process {
        id: profileProcess

        command: [root.engineExecutable, "profiles", "list"]
        running: false

        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: root.applyProfileList(text)
        }

        onExited: function (exitCode, exitStatus) {
            if (exitCode === 0)
                return;
            root.profileLabel = "Unavailable";
            root.profileError = "The VoxTypePersonas configuration is unavailable.";
            root.profileEntries = [];
            root.injectPanel();
        }
    }

    Process {
        id: updateNotification

        command: ["notify-send", "--app-name=VoxTypePersonas", "--urgency=normal", "--expire-time=5000", "VoxTypePersonas", "Update available"]
        running: false
    }

    Loader {
        id: panelLoader

        active: true
        source: Qt.resolvedUrl("Panel.qml")
        visible: false

        onLoaded: {
            root.injectPanel();
            Qt.callLater(root.injectPanel);
        }
    }

    BarIconButton {
        id: button

        anchors.fill: parent
        bar: root.bar
        active: !root.engineAvailable
        iconComponent: Component {
            Item {
                Image {
                    id: personaIconSource

                    anchors.fill: parent
                    source: Qt.resolvedUrl("assets/icons/persona-spark-solid.svg")
                    fillMode: Image.PreserveAspectFit
                    visible: false
                    layer.enabled: true
                }

                MultiEffect {
                    anchors.fill: personaIconSource
                    source: personaIconSource
                    colorization: 1.0
                    colorizationColor: root.engineAvailable ? (root.updateAvailable ? (root.bar ? root.bar.urgent : Color.urgent) : (root.bar ? root.bar.barForeground : Color.foreground)) : (root.bar ? root.bar.urgent : Color.urgent)
                }
            }
        }
        tooltipText: root.engineAvailable ? "VoxTypePersonas: " + root.profileLabel : "VoxTypePersonas"

        onPressed: function (buttonCode) {
            if (buttonCode === Qt.LeftButton)
                root.toggle();
        }
    }
}
