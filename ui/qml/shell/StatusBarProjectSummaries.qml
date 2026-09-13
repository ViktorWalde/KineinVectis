import QtQuick
import KineinVectis

// Os resumos do PROJETO na barra de status: o indice do projeto inteiro, o
// contexto de compilador do arquivo ativo (detalhe ao pairar) e — desde
// 2026-09-13 — o Python do projeto (interpretador, ambiente, modulo nativo).
// Saiu da WorkspaceStatusBar quando ela chegou a 298/300: o Python nao cabia,
// e "resumos do projeto" e' uma responsabilidade que a barra so' posiciona.
// Burro: tres textos, tres propriedades.
Row {
    id: root

    property string indexSummary: ""
    property string contextSummary: ""
    property string contextDetail: ""
    property string pythonSummary: ""

    spacing: Theme.spacingMedium

    Text {
        anchors.verticalCenter: parent.verticalCenter
        visible: root.indexSummary !== ""
        text: qsTr("índice: %1").arg(root.indexSummary)
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeStatus
    }

    Text {
        id: contextoTexto

        anchors.verticalCenter: parent.verticalCenter
        visible: root.contextSummary !== ""
        text: contextoArea.containsMouse && root.contextDetail !== ""
              ? root.contextDetail
              : qsTr("contexto: %1").arg(root.contextSummary)
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeStatus
        elide: Text.ElideMiddle
        width: Math.min(implicitWidth, 520)

        MouseArea {
            id: contextoArea

            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.NoButton
        }
    }

    // "python: .venv · 3.14.7 · pybind11 (scikit-build-core)" — o que o
    // PythonController resume; vazio fora de projeto Python.
    Text {
        anchors.verticalCenter: parent.verticalCenter
        visible: root.pythonSummary !== ""
        text: root.pythonSummary
        color: root.pythonSummary.indexOf("⚠") >= 0 ? Theme.warningSoft : Theme.textMuted
        font.pixelSize: Theme.fontSizeStatus
    }
}
