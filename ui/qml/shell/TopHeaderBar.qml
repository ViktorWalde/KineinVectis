import QtQuick

Rectangle {
    id: root

    property bool workspaceOpen: false
    property bool coreConnected: false
    property bool building: false
    property bool testing: false
    property bool analyzing: false
    property bool running: false

    signal openWorkspaceRequested()
    signal buildRequested()
    signal testsRequested()
    signal qualityRequested()
    signal runRequested()
    signal stopRunRequested()

    height: 44
    color: Theme.background1

    Row {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        spacing: Theme.spacingMedium

        Image {
            width: 26
            height: 26
            anchors.verticalCenter: parent.verticalCenter
            source: "qrc:/KineinVectis/assets/app-icon.png"
            fillMode: Image.PreserveAspectFit
            smooth: true
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Kinein Vectis")
            color: Theme.textPrimary
            font.pixelSize: 15
            font.bold: true
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: openFolderText.width + 2 * Theme.spacingMedium
            height: 28
            radius: Theme.radius
            color: openFolderArea.containsMouse ? Theme.surface2 : Theme.surface1
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: openFolderText

                anchors.centerIn: parent
                text: qsTr("Abrir pasta...")
                color: Theme.textPrimary
                font.pixelSize: 12
            }

            MouseArea {
                id: openFolderArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.openWorkspaceRequested()
            }
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: buildButtonText.width + 2 * Theme.spacingMedium
            height: 28
            radius: Theme.radius
            visible: root.workspaceOpen
            enabled: !root.building && root.coreConnected
            opacity: enabled ? 1.0 : 0.6
            color: root.building
                   ? Theme.surface1
                   : (buildArea.pressed ? Theme.accentDim : Theme.accent)

            Text {
                id: buildButtonText

                anchors.centerIn: parent
                text: root.building ? qsTr("Compilando...") : qsTr("Compilar")
                color: root.building ? Theme.textSecondary : Theme.background0
                font.pixelSize: 12
                font.bold: true
            }

            MouseArea {
                id: buildArea

                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.buildRequested()
            }
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: testButtonText.width + 2 * Theme.spacingMedium
            height: 28
            radius: Theme.radius
            visible: root.workspaceOpen
            enabled: !root.testing && root.coreConnected
            opacity: enabled ? 1.0 : 0.6
            color: testArea.pressed ? Theme.surface2 : Theme.surface1
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: testButtonText

                anchors.centerIn: parent
                text: root.testing ? qsTr("Testando...") : qsTr("Testes")
                color: root.testing ? Theme.textSecondary : Theme.textPrimary
                font.pixelSize: 12
                font.bold: true
            }

            MouseArea {
                id: testArea

                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.testsRequested()
            }
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: qualityButtonText.width + 2 * Theme.spacingMedium
            height: 28
            radius: Theme.radius
            visible: root.workspaceOpen
            enabled: !root.analyzing && root.coreConnected
            opacity: enabled ? 1.0 : 0.6
            color: qualityArea.pressed ? Theme.surface2 : Theme.surface1
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: qualityButtonText

                anchors.centerIn: parent
                text: root.analyzing ? qsTr("Analisando...") : qsTr("Análise")
                color: root.analyzing ? Theme.textSecondary : Theme.textPrimary
                font.pixelSize: 12
                font.bold: true
            }

            MouseArea {
                id: qualityArea

                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.qualityRequested()
            }
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: runButtonText.width + 2 * Theme.spacingMedium
            height: 28
            radius: Theme.radius
            visible: root.workspaceOpen
            enabled: root.coreConnected
            opacity: enabled ? 1.0 : 0.6
            color: root.running
                   ? (runArea.pressed ? Theme.surface1 : Theme.surface2)
                   : (runArea.pressed ? Theme.accentDim : Theme.accent)

            Text {
                id: runButtonText

                anchors.centerIn: parent
                text: root.running ? qsTr("■ Parar") : qsTr("▶ Iniciar")
                color: root.running ? Theme.textPrimary : Theme.background0
                font.pixelSize: 12
                font.bold: true
            }

            MouseArea {
                id: runArea

                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.running ? root.stopRunRequested() : root.runRequested()
            }
        }
    }
}
