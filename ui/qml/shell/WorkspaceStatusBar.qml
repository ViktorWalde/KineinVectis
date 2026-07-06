import QtQuick
import KineinVectis

Rectangle {
    id: bar

    property string workspaceRoot: ""
    property string workspaceKindLabel: ""
    property bool logsActive: false
    property bool building: false
    property bool testing: false
    property bool analyzing: false
    property bool scanningEnvironment: false
    property bool running: false
    property bool coreConnected: false
    property string coreProtocolVersion: ""
    property string coreStatus: ""

    signal logsRequested()
    signal cancelBuildRequested()
    signal cancelTestsRequested()
    signal cancelQualityRequested()
    signal cancelEnvironmentScanRequested()

    height: 26
    color: Theme.background1

    Row {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        spacing: Theme.spacingMedium

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.workspaceRoot !== ""
            text: bar.workspaceKindLabel + "  ·  " + bar.workspaceRoot
            color: Theme.textMuted
            font.pixelSize: 10
            font.family: Theme.monoFont
        }
    }

    Row {
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        spacing: Theme.spacingMedium

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: logsToggleText.width + 2 * Theme.spacingSmall
            height: 18
            radius: Theme.radius
            color: bar.logsActive ? Theme.accentDim
                                  : (logsToggleArea.containsMouse
                                     ? Theme.surface2 : "transparent")
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: logsToggleText

                anchors.centerIn: parent
                text: qsTr("IDE")
                color: bar.logsActive ? Theme.accent : Theme.textSecondary
                font.pixelSize: 10
            }

            MouseArea {
                id: logsToggleArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: bar.logsRequested()
            }
        }

        Row {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.building
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("compilando...")
                color: Theme.accent
                font.pixelSize: 10
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "×"
                color: cancelBuildArea.containsMouse ? Theme.errorSoft : Theme.textMuted
                font.pixelSize: 12

                MouseArea {
                    id: cancelBuildArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: bar.cancelBuildRequested()
                }
            }
        }

        Row {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.testing
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("testando...")
                color: Theme.accent
                font.pixelSize: 10
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "×"
                color: cancelTestsArea.containsMouse ? Theme.errorSoft : Theme.textMuted
                font.pixelSize: 12

                MouseArea {
                    id: cancelTestsArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: bar.cancelTestsRequested()
                }
            }
        }

        Row {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.analyzing
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("analisando...")
                color: Theme.accent
                font.pixelSize: 10
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "×"
                color: cancelQualityArea.containsMouse ? Theme.errorSoft : Theme.textMuted
                font.pixelSize: 12

                MouseArea {
                    id: cancelQualityArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: bar.cancelQualityRequested()
                }
            }
        }

        Row {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.scanningEnvironment
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("scan de ambiente...")
                color: Theme.accent
                font.pixelSize: 10
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "×"
                color: cancelEnvironmentArea.containsMouse
                       ? Theme.errorSoft : Theme.textMuted
                font.pixelSize: 12

                MouseArea {
                    id: cancelEnvironmentArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: bar.cancelEnvironmentScanRequested()
                }
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.running
            text: qsTr("executando...")
            color: Theme.accent
            font.pixelSize: 10
        }

        Rectangle {
            width: 7
            height: 7
            radius: 4
            anchors.verticalCenter: parent.verticalCenter
            color: bar.coreConnected ? Theme.successSoft : Theme.errorSoft
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: bar.coreConnected
                  ? qsTr("core conectado · IPC %1").arg(bar.coreProtocolVersion)
                  : qsTr("core %1").arg(bar.coreStatus)
            color: Theme.textMuted
            font.pixelSize: 10
        }
    }
}
