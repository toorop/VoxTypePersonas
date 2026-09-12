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

    readonly property bool opened: panelLoader.item
        ? panelLoader.item.opened === true
        : false
    readonly property bool popoutSwitchClosing: panelLoader.item
        ? panelLoader.item.popoutSwitchClosing === true
        : false

    function open() {
        if (panelLoader.item)
            panelLoader.item.open()
    }

    function close() {
        if (panelLoader.item)
            panelLoader.item.close()
    }

    function toggle() {
        if (panelLoader.item)
            panelLoader.item.toggle()
    }

    function closeForPopoutSwitch() {
        if (panelLoader.item)
            panelLoader.item.closeForPopoutSwitch()
    }

    function injectPanel() {
        if (!panelLoader.item)
            return

        panelLoader.item.bar = root.bar
        panelLoader.item.anchorItem = button
        panelLoader.item.hostWidget = root
        panelLoader.item.profileEntries = root.profileEntries
        panelLoader.item.profileError = root.profileError
        panelLoader.item.engineAvailable = root.engineAvailable
    }

    function refreshProfile() {
        if (!profileProcess.running)
            profileProcess.running = true
    }

    function applyProfileList(output) {
        var lines = String(output || "").trim().split("\n")
        var entries = []
        var hasActiveProfile = false

        root.profileError = ""
        root.engineAvailable = true

        for (var index = 0; index < lines.length; index++) {
            var line = lines[index]
            if (line.length < 3)
                continue

            var active = line.startsWith("* ")
            var fields = line.slice(2).split("\t")
            if (fields.length < 3 || fields[0].trim() === "" || fields[1].trim() === "")
                continue

            entries.push({
                id: fields[0].trim(),
                name: fields[1].trim(),
                state: fields[2].trim(),
                active: active
            })

            if (!active)
                continue

            root.profileLabel = fields[1].trim()
            root.profileError = ""
            hasActiveProfile = true
        }

        root.profileEntries = entries
        if (!hasActiveProfile) {
            root.profileLabel = "Unavailable"
            root.profileError = "No active profile is available."
        }
        root.injectPanel()
    }

    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight

    onBarChanged: injectPanel()
    Component.onCompleted: refreshProfile()

    Process {
        id: profileProcess

        command: ["voxtype-personas", "profiles", "list"]
        running: false

        stdout: StdioCollector {
            waitForEnd: true
            onStreamFinished: root.applyProfileList(text)
        }

        onExited: function(exitCode, exitStatus) {
            if (exitCode === 0)
                return

            root.profileLabel = "Unavailable"
            root.profileError = "The VoxTypePersonas engine or configuration is unavailable."
            root.profileEntries = []
            root.engineAvailable = false
            root.injectPanel()
        }
    }

    Loader {
        id: panelLoader

        active: true
        source: Qt.resolvedUrl("Panel.qml")
        visible: false

        onLoaded: {
            root.injectPanel()
            Qt.callLater(root.injectPanel)
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
                    colorizationColor: root.engineAvailable
                        ? (root.bar ? root.bar.barForeground : Color.foreground)
                        : (root.bar ? root.bar.urgent : Color.urgent)
                }
            }
        }
        tooltipText: root.engineAvailable
            ? "VoxTypePersonas: " + root.profileLabel
            : "VoxTypePersonas"

        onPressed: function(buttonCode) {
            if (buttonCode === Qt.LeftButton)
                root.toggle()
        }
    }
}
