import QtQuick
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

Panel {
    id: root

    moduleName: "io.github.toorop.voxtype-personas"
    manageIpc: false

    property var anchorItem: null
    property var hostWidget: null
    property var profileEntries: []
    property string profileError: ""
    property string actionError: ""
    property bool engineAvailable: false
    property string engineStatus: "checking"
    property string engineVersion: ""
    property string installationStage: ""
    property string installationError: ""
    property var installationSteps: []
    property bool installationTransactionReady: false
    property bool updateAvailable: false

    function installationMessage() {
        if (root.engineAvailable)
            return root.installationError !== ""
                ? "The engine update could not be completed."
                : "Updating the VoxTypePersonas engine…"

        if (root.engineStatus === "unsupported-architecture")
            return "This computer architecture is not supported."
        if (root.engineStatus === "checking")
            return "Checking the VoxTypePersonas engine…"

        if (root.engineStatus === "invalid") {
            if (!root.installationTransactionReady)
                return "The installed engine is invalid. This development build cannot replace it yet."

            return "The installed engine is invalid and cannot be used."
        }

        if (!root.installationTransactionReady)
            return "The engine is not installed. This development build cannot install it yet."

        return "The VoxTypePersonas engine is not installed."
    }

    function installationActionText() {
        return root.engineStatus === "invalid" ? "Replace engine" : "Install engine"
    }

    function confirmEngineInstallation() {
        if (root.hostWidget)
            root.hostWidget.beginEngineInstallation()
    }

    function open() {
        root.controller.show()
    }

    function close() {
        root.controller.hide()
    }

    function toggle() {
        if (root.opened)
            root.close()
        else
            root.open()
    }

    function closeForPopoutSwitch() {
        root.close()
    }

    function switchPanel(direction) {
        if (root.bar && typeof root.bar.switchPanelFrom === "function")
            return root.bar.switchPanelFrom(root.hostWidget || root, direction)

        return false
    }

    function activateProfile(entry) {
        if (!entry || (entry.state !== "Ready" && entry.state !== "Active"))
            return
        if (entry.active) {
            root.close()
            return
        }

        root.actionError = ""
        setActiveProcess.command = [root.hostWidget.engineExecutable, "profiles", "set-active", entry.id]
        setActiveProcess.running = true
    }

    function refreshProfiles() {
        root.actionError = ""
        if (root.hostWidget)
            root.hostWidget.refreshProfile()
    }

    KeyboardPanel {
        id: panel

        anchorItem: root.anchorItem
        owner: root.hostWidget || root
        bar: root.bar
        open: root.opened
        focusTarget: keyCatcher
        contentWidth: panel.fittedContentWidth(Style.space(260))
        contentHeight: panel.fittedContentHeight(content.implicitHeight)

        PanelKeyCatcher {
            id: keyCatcher

            anchors.fill: parent
            onCloseRequested: root.close()
            onTabRequested: function(direction) {
                root.switchPanel(direction)
            }

            Column {
                id: content

                width: parent.width
                spacing: Style.space(8)

                Text {
                    width: parent.width
                    text: "VoxTypePersonas"
                    visible: root.engineAvailable
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.subtitle
                    font.bold: true
                }

                Text {
                    width: parent.width
                    visible: root.engineAvailable && root.updateAvailable && root.installationStage === ""
                    text: "Update available"
                    color: root.bar ? root.bar.urgent : Color.urgent
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    font.bold: true
                }

                Button {
                    width: parent.width
                    visible: root.engineAvailable && root.updateAvailable && root.installationStage === ""
                    text: "Update engine"
                    foreground: root.bar ? root.bar.urgent : Color.urgent
                    fontFamily: root.bar ? root.bar.fontFamily : Style.font.family
                    bordered: true
                    onClicked: root.confirmEngineInstallation()
                }

                Text {
                    width: parent.width
                    text: "Refresh"
                    visible: root.engineAvailable
                    color: root.barForeground
                    opacity: refreshMouseArea.containsMouse ? 1 : 0.7
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption

                    MouseArea {
                        id: refreshMouseArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.refreshProfiles()
                    }
                }

                Text {
                    width: parent.width
                    text: "Select the profile for the next dictation."
                    visible: root.engineAvailable
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.body
                    wrapMode: Text.WordWrap
                }

                Rectangle {
                    width: parent.width
                    height: 1
                    visible: root.engineAvailable
                    color: root.barForeground
                    opacity: 0.2
                }

                Text {
                    width: parent.width
                    visible: root.engineAvailable && root.profileError !== ""
                    text: root.profileError
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Text {
                    width: parent.width
                    visible: root.engineAvailable && root.profileError === "" && root.profileEntries.length === 0
                    text: "No profiles are available."
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Repeater {
                    model: root.engineAvailable ? root.profileEntries : []

                    delegate: Item {
                        id: profileRow

                        required property var modelData

                        width: parent.width
                        height: profileName.implicitHeight + profileState.implicitHeight + Style.space(6)

                        Text {
                            id: profileName

                            width: parent.width
                            text: (profileRow.modelData.active ? "✓  " : "   ")
                                + profileRow.modelData.name
                            color: root.barForeground
                            font.family: root.bar ? root.bar.fontFamily : Style.font.family
                            font.pixelSize: Style.font.body
                        }

                        Text {
                            id: profileState

                            anchors.top: profileName.bottom
                            width: parent.width
                            text: "   " + profileRow.modelData.state
                                + (profileRow.modelData.state === "Draft" ? " · needs configuration" : "")
                            color: root.barForeground
                            opacity: 0.7
                            font.family: root.bar ? root.bar.fontFamily : Style.font.family
                            font.pixelSize: Style.font.caption
                        }

                        MouseArea {
                            anchors.fill: parent
                            enabled: profileRow.modelData.state === "Ready"
                                || profileRow.modelData.state === "Active"
                            onClicked: root.activateProfile(profileRow.modelData)
                        }
                    }
                }

                Text {
                    width: parent.width
                    visible: root.engineAvailable && root.actionError !== ""
                    text: root.actionError
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Text {
                    width: parent.width
                    visible: !root.engineAvailable || root.installationStage !== "" || root.installationError !== ""
                    text: "Engine installation"
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.subtitle
                    font.bold: true
                }

                Text {
                    width: parent.width
                    visible: !root.engineAvailable || root.installationStage !== "" || root.installationError !== ""
                    text: root.installationMessage()
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.body
                    wrapMode: Text.WordWrap
                }

                Repeater {
                    model: root.installationStage !== ""
                        ? root.installationSteps : []

                    delegate: Text {
                        required property var modelData

                        width: parent.width
                        text: (root.installationStage === modelData.id ? "•  " : "   ")
                            + modelData.label
                        color: root.barForeground
                        opacity: root.installationStage === modelData.id ? 1 : 0.6
                        font.family: root.bar ? root.bar.fontFamily : Style.font.family
                        font.pixelSize: Style.font.caption
                        wrapMode: Text.WordWrap
                    }
                }

                Text {
                    width: parent.width
                    visible: root.installationError !== ""
                    text: root.installationError
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Button {
                    width: parent.width
                    visible: !root.engineAvailable
                        && (root.installationStage === "" || root.installationError !== "")
                    enabled: root.installationTransactionReady
                        && root.engineStatus !== "unsupported-architecture"
                        && root.engineStatus !== "checking"
                    text: root.installationTransactionReady
                        ? root.installationActionText()
                        : "Installation unavailable"
                    foreground: root.barForeground
                    fontFamily: root.bar ? root.bar.fontFamily : Style.font.family
                    bordered: true
                    onClicked: root.confirmEngineInstallation()
                }

            }
        }
    }

    Process {
        id: setActiveProcess

        running: false

        onExited: function(exitCode, exitStatus) {
            if (exitCode !== 0) {
                root.actionError = "The selected profile could not be activated."
                return
            }

            if (root.hostWidget)
                root.hostWidget.refreshProfile()
            root.close()
        }
    }
}
