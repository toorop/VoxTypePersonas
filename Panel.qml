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
        setActiveProcess.command = ["voxtype-personas", "profiles", "set-active", entry.id]
        setActiveProcess.running = true
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
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.subtitle
                    font.bold: true
                }

                Text {
                    width: parent.width
                    text: "Select the profile for the next dictation."
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.body
                    wrapMode: Text.WordWrap
                }

                Rectangle {
                    width: parent.width
                    height: 1
                    color: root.barForeground
                    opacity: 0.2
                }

                Text {
                    width: parent.width
                    visible: root.profileError !== ""
                    text: root.profileError
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Repeater {
                    model: root.profileEntries

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
                    visible: root.actionError !== ""
                    text: root.actionError
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
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
