pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os VALORES das grandezas que o autor ligou.
//
// Campo comeca VAZIO e nao ganha padrao (arquitetura/34 §2.1): a conta nao roda
// com campo em branco, e a IDE nao supoe zero — supor zero seria preencher.
Column {
    id: root

    property var quantities: []
    property var values: ({})

    signal edited(string quantityId, string text)

    spacing: Theme.spacingXSmall

    Text {
        text: qsTr("Os valores")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Repeater {
        model: root.quantities

        delegate: Row {
            id: linha

            required property var modelData

            readonly property string texto: {
                const v = root.values[linha.modelData.id];
                return v === undefined ? "" : String(v);
            }

            spacing: Theme.spacingSmall

            Text {
                width: 190
                anchors.verticalCenter: parent.verticalCenter
                elide: Text.ElideRight
                text: linha.modelData.unit === ""
                      ? linha.modelData.label
                      : linha.modelData.label + " (" + linha.modelData.unit + ")"
                color: Theme.textSecondary
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus
            }

            Rectangle {
                width: 130
                height: 26
                radius: Theme.radiusXSmall
                color: Theme.backgroundEditor
                border.width: 1
                // Vazio nao e' erro: e' "ainda nao". A borda avisa sem acusar.
                border.color: linha.texto.trim() === "" ? Theme.borderSoft
                                                        : (isFinite(Number(linha.texto))
                                                           ? Theme.borderStrong : Theme.warningSoft)

                TextInput {
                    id: campo

                    anchors.fill: parent
                    anchors.margins: Theme.spacingXSmall
                    verticalAlignment: TextInput.AlignVCenter
                    clip: true
                    text: linha.texto
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeStatus
                    selectByMouse: true

                    onTextEdited: root.edited(linha.modelData.id, campo.text)
                }
            }
        }
    }
}
