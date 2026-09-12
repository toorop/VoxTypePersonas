import QtQuick
import Quickshell
import Quickshell.Io
import qs.Ui

BarWidget {
    id: root

    moduleName: "io.github.toorop.voxtype-personas"

    property string profileLabel: "Loading…"
    property string profileError: ""
    property var profileEntries: []

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

    WidgetButton {
        id: button

        anchors.fill: parent
        bar: root.bar
        text: root.profileLabel
        tooltipText: root.profileError === ""
            ? "VoxTypePersonas: " + root.profileLabel
            : root.profileError

        onPressed: function(buttonCode) {
            if (buttonCode === Qt.LeftButton)
                root.toggle()
        }
    }
}
