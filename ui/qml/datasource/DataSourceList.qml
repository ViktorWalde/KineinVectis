pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A coluna de LOCAIS do navegador do Banco (2026-10-03, no modelo do seletor
// de pastas): CONEXÕES salvas em cima — e' para onde se volta —, depois o que
// responde NESTA MÁQUINA (socket, porta, container, .sqlite). Cada linha diz
// o motor pelo icone e pela etiqueta; a selecionada fica marcada. No pe', as
// duas portas de criacao: uma conexao nova (o formulario) e um banco novo.
Item {
    id: root

    property var profiles: []
    property string selectedName: ""
    property var candidates: []
    property bool discovering: false
    property string discoverHint: ""

    signal profileSelected(string name)
    signal candidateSelected(int index)
    signal discoverRequested()
    signal newRequested()
    signal createRequested()

    // Sem host e' arquivo: a linha e' o caminho.
    function profileDetail(profile) {
        return profile.host === "" ? profile.database
                                   : profile.user + "@" + profile.host + ":" + profile.port + "/" + profile.database;
    }

    component SectionTitle: Text {
        width: parent ? parent.width : 0
        leftPadding: Theme.spacingSmall
        topPadding: Theme.spacingSmall
        bottomPadding: Theme.spacingXSmall
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
        font.weight: Font.DemiBold
        font.letterSpacing: 0.8
    }

    // Uma linha: icone do motor, nome, detalhe em mono e a etiqueta do motor.
    component PlaceRow: Rectangle {
        id: row

        property string iconName: "database"
        property bool iconLive: true
        property string title: ""
        property string detail: ""
        property string badge: ""
        property bool current: false
        property bool production: false

        signal clicked()

        width: parent ? parent.width : 0
        height: 38
        radius: Theme.radius
        color: row.current ? Theme.surfaceSelected : (rowArea.containsMouse ? Theme.surface2 : "transparent")

        KvIcon {
            id: rowIcon

            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            size: 18
            name: row.iconName
            active: row.current
            disabled: !row.iconLive
        }

        Column {
            anchors.left: rowIcon.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: badgeLabel.left
            anchors.rightMargin: Theme.spacingXSmall
            anchors.verticalCenter: parent.verticalCenter

            Text {
                width: parent.width
                text: row.title
                textFormat: Text.PlainText
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeSmall
                font.weight: row.current ? Font.DemiBold : Font.Normal
                elide: Text.ElideRight
            }

            Text {
                width: parent.width
                visible: row.detail !== ""
                text: row.detail
                textFormat: Text.PlainText
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeMicro
                elide: Text.ElideMiddle
            }
        }

        Text {
            id: badgeLabel

            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            text: row.badge
            color: row.production ? Theme.errorSoft : Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro
        }

        MouseArea {
            id: rowArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: row.clicked()
        }
    }

    Flickable {
        id: rolagem

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: rodape.top
        anchors.bottomMargin: Theme.spacingSmall
        clip: true
        contentWidth: width
        contentHeight: coluna.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: coluna

            width: rolagem.width
            spacing: 2

            SectionTitle {
                text: qsTr("CONEXÕES")
            }

            Repeater {
                model: root.profiles

                delegate: PlaceRow {
                    required property var modelData

                    iconName: DataSourceKinds.engineIcon(modelData.engine)
                    title: modelData.name
                    detail: root.profileDetail(modelData) + (DataSourceKinds.policyLabel(modelData) ? " · " + DataSourceKinds.policyLabel(modelData) : "")
                    production: modelData.production === true
                    badge: modelData.production === true ? qsTr("PROD") : DataSourceKinds.engineShort(modelData.engine)
                    current: modelData.name === root.selectedName
                    onClicked: root.profileSelected(modelData.name)
                }
            }

            Text {
                width: parent.width
                visible: root.profiles.length === 0
                leftPadding: Theme.spacingSmall
                rightPadding: Theme.spacingSmall
                wrapMode: Text.WordWrap
                text: qsTr("Nenhuma conexão salva. Escolha uma desta máquina, abaixo, ou crie uma.")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
            }

            Item {
                width: parent.width
                height: discoverTitle.implicitHeight

                SectionTitle {
                    id: discoverTitle

                    text: root.discovering ? qsTr("NESTA MÁQUINA — PROCURANDO…") : qsTr("NESTA MÁQUINA")
                }

                KvIconButton {
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    compact: true
                    iconName: "refresh"
                    iconSize: 14
                    tooltip: qsTr("Procurar de novo (sockets, portas, containers, .sqlite)")
                    enabled: !root.discovering
                    onClicked: root.discoverRequested()
                }
            }

            Repeater {
                model: root.candidates

                delegate: PlaceRow {
                    required property int index
                    required property var modelData

                    iconName: DataSourceKinds.engineIcon(modelData.profile.engine)
                    iconLive: modelData.running
                    title: modelData.label
                    detail: modelData.detail
                    badge: modelData.running ? DataSourceKinds.engineShort(modelData.profile.engine) : qsTr("parado")
                    onClicked: root.candidateSelected(index)
                }
            }

            Text {
                width: parent.width
                visible: !root.discovering && root.candidates.length === 0
                leftPadding: Theme.spacingSmall
                rightPadding: Theme.spacingSmall
                wrapMode: Text.WordWrap
                text: root.discoverHint !== "" ? root.discoverHint : qsTr("nada respondeu")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
            }
        }
    }

    Column {
        id: rodape

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        KvButton {
            width: parent.width
            compact: true
            iconName: "add"
            text: qsTr("Conectar banco")
            onClicked: root.newRequested()
        }

        KvButton {
            width: parent.width
            compact: true
            iconName: "database"
            text: qsTr("Criar banco…")
            onClicked: root.createRequested()
        }
    }
}
