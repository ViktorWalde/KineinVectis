pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A lista densa da spec 9.1 §11.3: nome, uma linha de descricao, arquivo
// afetado e badge. Sem card grande, sem icone grande, sem banner.
//
// A lista MOSTRA acoes indisponiveis em vez de escondê-las, com o motivo do
// core ao lado. Esconder faria o usuario procurar uma acao que existe; dizer
// "o preset debug ja existe" responde a pergunta dele de uma vez.
Rectangle {
    id: root

    property var actionsModel: null
    property string scopeFilter: ""
    property string searchQuery: ""
    property string selectedId: ""

    signal actionSelected(string id)

    radius: Theme.radius
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1
    clip: true

    function matches(entry) {
        if (root.scopeFilter !== "" && entry.scope !== root.scopeFilter) {
            return false;
        }
        const query = root.searchQuery.trim().toLowerCase();
        if (query === "") {
            return true;
        }
        return entry.title.toLowerCase().includes(query)
                || entry.description.toLowerCase().includes(query)
                || entry.actionId.toLowerCase().includes(query)
                || entry.category.toLowerCase().includes(query);
    }

    // A MESMA convencao do painel de bibliotecas, e de proposito: verde e'
    // "ja' esta no seu projeto", cinza e' "disponivel para ativar". Duas telas
    // com a mesma bolinha significando coisas diferentes seria pior que nao
    // ter bolinha (relato de uso, 2026-09-04).
    function stateColor(state) {
        if (state === "alreadyApplied") {
            return Theme.successSoft;
        }
        if (state === "recommended") {
            return Theme.accent;
        }
        if (state === "partiallyAvailable") {
            return Theme.warningSoft;
        }
        return Theme.textDisabled;
    }

    // O que a bolinha quer dizer, em palavra.
    function stateLabel(state) {
        if (state === "alreadyApplied") {
            return qsTr("ativa neste projeto");
        }
        if (state === "recommended") {
            return qsTr("recomendada");
        }
        if (state === "partiallyAvailable") {
            return qsTr("falta condição");
        }
        if (state === "unavailable") {
            return qsTr("não se aplica aqui");
        }
        return "";
    }

    function riskColor(risk) {
        if (risk === "high") {
            return Theme.errorSoft;
        }
        return risk === "medium" ? Theme.warningSoft : Theme.textMuted;
    }

    ListView {
        id: view

        anchors.fill: parent
        anchors.margins: 1
        model: root.actionsModel
        spacing: 0
        boundsBehavior: Flickable.StopAtBounds
        clip: true

        delegate: Rectangle {
            id: entry

            required property int index
            required property string actionId
            required property string title
            required property string description
            required property string scope
            required property string category
            required property string risk
            required property string riskLabel
            required property string actionState
            required property string reason
            required property string affects

            readonly property bool selected: entry.actionId === root.selectedId
            readonly property bool shown: root.matches(entry)

            width: view.width
            height: shown ? column.height + 2 * Theme.spacingSmall : 0
            visible: shown
            color: selected ? Theme.surfaceSelected
                            : (hover.containsMouse ? Theme.surface2 : "transparent")

            Rectangle {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: 2
                color: entry.selected ? Theme.accent : "transparent"
            }

            Column {
                id: column

                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                anchors.topMargin: Theme.spacingSmall
                spacing: 2

                Row {
                    width: parent.width
                    spacing: Theme.spacingXSmall

                    Rectangle {
                        width: 6
                        height: 6
                        radius: 3
                        anchors.verticalCenter: parent.verticalCenter
                        color: root.stateColor(entry.actionState)
                    }

                    Text {
                        text: entry.title
                        color: entry.actionState === "unavailable"
                               ? Theme.textMuted : Theme.textPrimary
                        font.pixelSize: 12
                        font.bold: entry.selected
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.stateLabel(entry.actionState)
                        color: entry.actionState === "alreadyApplied"
                               ? Theme.successSoft : Theme.textMuted
                        font.pixelSize: 10
                    }
                }

                Text {
                    width: parent.width
                    text: entry.reason !== "" ? entry.reason : entry.description
                    color: entry.reason !== "" ? Theme.textMuted : Theme.textSecondary
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }

                Text {
                    width: parent.width
                    text: (entry.affects !== "" ? entry.affects : entry.category) + " · "
                          + entry.riskLabel
                    color: root.riskColor(entry.risk)
                    font.pixelSize: 10
                    elide: Text.ElideRight
                }
            }

            MouseArea {
                id: hover

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.actionSelected(entry.actionId)
            }
        }
    }

    Text {
        anchors.centerIn: parent
        width: parent.width - 2 * Theme.spacingMedium
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: qsTr("Nenhuma ação para este projeto. Abra um workspace com CMakeLists.txt ou Cargo.toml.")
        color: Theme.textMuted
        font.pixelSize: 11
        visible: root.actionsModel === null || root.actionsModel.count === 0
    }
}
