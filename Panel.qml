import QtQuick
import QtQuick.Controls as QQC
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
    property string view: "selector"
    property string settingsSection: "profiles"

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
        root.view = "selector"
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

    function openSettings() {
        if (!root.engineAvailable)
            return

        root.settingsSection = "profiles"
        root.view = "settings"
    }

    function closeSettings() {
        root.view = "selector"
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
            blocked: profileSelector.popupOpen
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
                    text: root.view === "settings" ? "Settings" : "VoxTypePersonas"
                    visible: root.engineAvailable
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.subtitle
                    font.bold: true
                }

                Text {
                    width: parent.width
                    visible: root.view === "selector" && root.engineAvailable
                        && root.updateAvailable && root.installationStage === ""
                    text: "Update available"
                    color: root.bar ? root.bar.urgent : Color.urgent
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    font.bold: true
                }

                Button {
                    width: parent.width
                    visible: root.view === "selector" && root.engineAvailable
                        && root.updateAvailable && root.installationStage === ""
                    text: "Update engine"
                    foreground: root.bar ? root.bar.urgent : Color.urgent
                    fontFamily: root.bar ? root.bar.fontFamily : Style.font.family
                    bordered: true
                    onClicked: root.confirmEngineInstallation()
                }

                Text {
                    width: parent.width
                    text: "Refresh"
                    visible: root.view === "selector" && root.engineAvailable
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
                    text: "Profile"
                    visible: root.view === "selector" && root.engineAvailable
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    font.bold: true
                }

                Item {
                    id: profileSelector

                    width: parent.width
                    height: Style.spacing.controlHeight
                    visible: root.view === "selector" && root.engineAvailable

                    readonly property bool popupOpen: profilePopup.opened

                    function activeEntry() {
                        for (var index = 0; index < root.profileEntries.length; index++) {
                            if (root.profileEntries[index].active)
                                return root.profileEntries[index]
                        }

                        return null
                    }

                    function choose(entry) {
                        if (!entry)
                            return

                        if (entry.state !== "Ready" && entry.state !== "Active") {
                            root.actionError = "Draft profiles must be configured before activation."
                            return
                        }

                        profilePopup.close()
                        root.activateProfile(entry)
                    }

                    BorderSurface {
                        id: profileTrigger

                        anchors.fill: parent
                        radius: Style.cornerRadius
                        color: Style.controlFill(activeFocus, triggerHover.hovered,
                            root.barForeground, root.bar ? root.bar.accent : Color.accent)
                        borderSpec: Border.controlSpec(activeFocus ? "focus"
                            : (triggerHover.hovered ? "hover-cursor" : "normal"),
                            root.barForeground, root.bar ? root.bar.accent : Color.accent)
                        activeFocusOnTab: true

                        HoverHandler {
                            id: triggerHover
                        }

                        Keys.onPressed: function(event) {
                            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter
                                || event.key === Qt.Key_Space || event.key === Qt.Key_Down) {
                                profilePopup.opened ? profilePopup.close() : profilePopup.open()
                                event.accepted = true
                            } else if (event.key === Qt.Key_Escape && profilePopup.opened) {
                                profilePopup.close()
                                event.accepted = true
                            }
                        }

                        Text {
                            anchors.left: parent.left
                            anchors.right: profileChevron.left
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.leftMargin: profileTrigger.borderLeft + Style.spacing.controlPaddingX
                            anchors.rightMargin: profileTrigger.borderRight + Style.spacing.md
                            text: {
                                var active = profileSelector.activeEntry()
                                return active ? active.name : "No active profile"
                            }
                            color: root.barForeground
                            font.family: root.bar ? root.bar.fontFamily : Style.font.family
                            font.pixelSize: Style.font.body
                            elide: Text.ElideRight
                        }

                        Text {
                            id: profileChevron

                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.rightMargin: profileTrigger.borderRight + Style.spacing.controlGap
                            text: "󰅀"
                            color: root.barForeground
                            opacity: 0.7
                            font.family: root.bar ? root.bar.fontFamily : Style.font.family
                            font.pixelSize: Style.font.body
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                profileTrigger.forceActiveFocus()
                                profilePopup.opened ? profilePopup.close() : profilePopup.open()
                            }
                        }
                    }

                    QQC.Popup {
                        id: profilePopup

                        x: 0
                        y: profileSelector.height + Style.spacing.xxs
                        width: profileSelector.width
                        implicitHeight: Math.min(profileOptions.contentHeight
                            + topPadding + bottomPadding, Style.spacing.popupRowHeight * 6)
                        padding: Style.spacing.hairline
                        leftPadding: Border.left(profilePopupBorderSpec) + Style.spacing.hairline
                        rightPadding: Border.right(profilePopupBorderSpec) + Style.spacing.hairline
                        topPadding: Border.top(profilePopupBorderSpec) + Style.spacing.hairline
                        bottomPadding: Border.bottom(profilePopupBorderSpec) + Style.spacing.hairline
                        focus: true

                        readonly property var profilePopupBorderSpec: Border.localOrSurfaceSpec(
                            "popups", "border", root.barForeground, Color.popups.border,
                            Style.normalBorderWidth)

                        background: BorderSurface {
                            color: Color.popups.background
                            borderSpec: profilePopup.profilePopupBorderSpec
                            radius: Style.cornerRadius
                        }

                        contentItem: ListView {
                            id: profileOptions

                            implicitHeight: contentHeight
                            clip: true
                            boundsBehavior: Flickable.StopAtBounds
                            spacing: Style.spacing.labelGap
                            model: root.profileEntries

                            delegate: Item {
                                id: profileOption

                                required property var modelData

                                width: profileOptions.width
                                height: profileOptionName.implicitHeight
                                    + profileOptionState.implicitHeight + Style.space(6)
                                opacity: profileOption.modelData.state === "Draft" ? 0.55 : 1

                                Text {
                                    id: profileOptionName

                                    width: parent.width
                                    text: (profileOption.modelData.active ? "✓  " : "   ")
                                        + profileOption.modelData.name
                                    color: root.barForeground
                                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                                    font.pixelSize: Style.font.body
                                    elide: Text.ElideRight
                                }

                                Text {
                                    id: profileOptionState

                                    anchors.top: profileOptionName.bottom
                                    width: parent.width
                                    text: "   " + profileOption.modelData.state
                                        + (profileOption.modelData.state === "Draft"
                                            ? " · needs configuration" : "")
                                    color: root.barForeground
                                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                                    font.pixelSize: Style.font.caption
                                    elide: Text.ElideRight
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: profileOption.modelData.state === "Draft"
                                        ? Qt.ForbiddenCursor : Qt.PointingHandCursor
                                    onClicked: profileSelector.choose(profileOption.modelData)
                                }
                            }
                        }
                    }
                }

                Text {
                    width: parent.width
                    visible: root.view === "selector" && root.engineAvailable && root.profileError !== ""
                    text: root.profileError
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Text {
                    width: parent.width
                    visible: root.view === "selector" && root.engineAvailable
                        && root.profileError === "" && root.profileEntries.length === 0
                    text: "No profiles are available."
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Text {
                    width: parent.width
                    visible: root.view === "selector" && root.engineAvailable && root.actionError !== ""
                    text: root.actionError
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Text {
                    width: parent.width
                    visible: root.view === "selector" && root.engineAvailable
                    text: "Settings…"
                    color: root.barForeground
                    opacity: settingsMouseArea.containsMouse ? 1 : 0.7
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption

                    MouseArea {
                        id: settingsMouseArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.openSettings()
                    }
                }

                Column {
                    width: parent.width
                    visible: root.view === "settings" && root.engineAvailable
                    spacing: Style.space(10)

                    Text {
                        width: parent.width
                        text: "Back to profile selection"
                        color: root.barForeground
                        opacity: backMouseArea.containsMouse ? 1 : 0.7
                        font.family: root.bar ? root.bar.fontFamily : Style.font.family
                        font.pixelSize: Style.font.caption

                        MouseArea {
                            id: backMouseArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.closeSettings()
                        }
                    }

                    Row {
                        width: parent.width
                        spacing: Style.space(10)

                        Repeater {
                            model: [
                                { id: "profiles", label: "Profiles" },
                                { id: "prompts", label: "Prompts" },
                                { id: "providers", label: "Providers" }
                            ]

                            delegate: Text {
                                required property var modelData

                                text: modelData.label
                                color: root.barForeground
                                opacity: root.settingsSection === modelData.id ? 1 : 0.6
                                font.family: root.bar ? root.bar.fontFamily : Style.font.family
                                font.pixelSize: Style.font.caption
                                font.bold: root.settingsSection === modelData.id

                                MouseArea {
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.settingsSection = parent.modelData.id
                                }
                            }
                        }
                    }

                    Rectangle {
                        width: parent.width
                        height: 1
                        color: root.barForeground
                        opacity: 0.2
                    }

                    Text {
                        width: parent.width
                        visible: root.settingsSection === "profiles"
                        text: "Profile management will be added in the next settings step."
                        color: root.barForeground
                        font.family: root.bar ? root.bar.fontFamily : Style.font.family
                        font.pixelSize: Style.font.body
                        wrapMode: Text.WordWrap
                    }

                    Text {
                        width: parent.width
                        visible: root.settingsSection === "prompts"
                        text: "Prompt editing will be added in a later settings step."
                        color: root.barForeground
                        font.family: root.bar ? root.bar.fontFamily : Style.font.family
                        font.pixelSize: Style.font.body
                        wrapMode: Text.WordWrap
                    }

                    Text {
                        width: parent.width
                        visible: root.settingsSection === "providers"
                        text: "Provider configuration will be added in a later settings step."
                        color: root.barForeground
                        font.family: root.bar ? root.bar.fontFamily : Style.font.family
                        font.pixelSize: Style.font.body
                        wrapMode: Text.WordWrap
                    }
                }

                Text {
                    width: parent.width
                    visible: root.view === "selector" && (!root.engineAvailable
                        || root.installationStage !== "" || root.installationError !== "")
                    text: "Engine installation"
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.subtitle
                    font.bold: true
                }

                Text {
                    width: parent.width
                    visible: root.view === "selector" && (!root.engineAvailable
                        || root.installationStage !== "" || root.installationError !== "")
                    text: root.installationMessage()
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.body
                    wrapMode: Text.WordWrap
                }

                Repeater {
                    model: root.view === "selector" && root.installationStage !== ""
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
                    visible: root.view === "selector" && root.installationError !== ""
                    text: root.installationError
                    color: root.barForeground
                    font.family: root.bar ? root.bar.fontFamily : Style.font.family
                    font.pixelSize: Style.font.caption
                    wrapMode: Text.WordWrap
                }

                Button {
                    width: parent.width
                    visible: root.view === "selector" && !root.engineAvailable
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
