pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Painel lateral do preview (spec 9.1 §12): o que faz, os campos, o DIFF, a
// documentacao e o Apply.
//
// O realce do diff usa prefixo/sufixo comum de linhas — exato para o que estas
// acoes fazem (inserir e acrescentar bloco) e honesto quando nao e: uma
// reescrita do arquivo inteiro aparece como um bloco grande alterado, que e a
// verdade. A UI nao inventa um diff mais bonito do que o efeito real; o texto
// mostrado E o que sera gravado, porque veio do mesmo plano do core.
Item {
    id: root

    property var controller: null

    readonly property var action: root.controller.selectedAction
    readonly property var file: root.controller.previewFiles.length > 0
                                ? root.controller.previewFiles[0] : null

    // Linhas do resultado, marcadas como alteradas ou nao.
    function diffLines() {
        if (root.file === null) {
            return [];
        }
        const after = root.file.after.split("\n");
        const before = root.file.before !== undefined ? root.file.before.split("\n") : [];
        let head = 0;
        while (head < before.length && head < after.length && before[head] === after[head]) {
            ++head;
        }
        let tail = 0;
        while (tail < before.length - head && tail < after.length - head
               && before[before.length - 1 - tail] === after[after.length - 1 - tail]) {
            ++tail;
        }
        const lines = [];
        for (let index = 0; index < after.length; ++index) {
            lines.push({
                text: after[index],
                added: index >= head && index < after.length - tail
            });
        }
        return lines;
    }

    Column {
        id: header

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Text {
            width: parent.width
            text: root.action !== null ? root.action.title : qsTr("Selecione uma ação")
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: root.controller.previewSummary !== ""
                  ? root.controller.previewSummary
                  : (root.action !== null ? root.action.description : "")
            color: Theme.textSecondary
            font.pixelSize: 11
            wrapMode: Text.WordWrap
        }

        // Campos declarados pela acao. Sem parametro, nada aparece.
        Repeater {
            model: root.action !== null ? root.action.params : []

            Row {
                id: paramRow

                required property var modelData

                width: header.width
                spacing: Theme.spacingSmall

                Text {
                    width: 110
                    anchors.verticalCenter: parent.verticalCenter
                    text: paramRow.modelData.label + (paramRow.modelData.required ? " *" : "")
                    color: Theme.textSecondary
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }

                Rectangle {
                    width: paramRow.width - 110 - Theme.spacingSmall
                    height: 26
                    radius: Theme.radius
                    color: Theme.background0
                    border.color: input.activeFocus ? Theme.accent : Theme.borderSoft
                    border.width: 1

                    TextInput {
                        id: input

                        anchors.fill: parent
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.rightMargin: Theme.spacingSmall
                        verticalAlignment: TextInput.AlignVCenter
                        color: Theme.textPrimary
                        selectionColor: Theme.accentDim
                        selectedTextColor: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: 12
                        clip: true
                        selectByMouse: true
                        onTextChanged: {
                            root.controller.setParam(paramRow.modelData.name, text);
                            root.controller.requestPreview();
                        }
                    }

                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        text: paramRow.modelData.placeholder
                        color: Theme.textDisabled
                        font.family: Theme.monoFont
                        font.pixelSize: 12
                        visible: input.text === ""
                    }
                }
            }
        }
    }

    Rectangle {
        id: diffBox

        anchors.top: header.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: footer.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        radius: Theme.radius
        color: Theme.backgroundEditor
        border.color: Theme.borderSoft
        border.width: 1
        clip: true

        Text {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            text: root.file !== null
                  ? (root.file.before === undefined
                     ? qsTr("%1 (arquivo novo)").arg(root.file.path)
                     : root.file.path)
                  : ""
            color: Theme.textMuted
            font.pixelSize: 10
            visible: root.file !== null
            id: diffPath
        }

        ListView {
            anchors.top: root.file !== null ? diffPath.bottom : parent.top
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            model: root.file !== null ? root.diffLines() : root.controller.previewReport
            boundsBehavior: Flickable.StopAtBounds
            clip: true

            delegate: Text {
                required property var modelData

                text: typeof modelData === "string" ? modelData : modelData.text
                color: (typeof modelData !== "string" && modelData.added)
                       ? Theme.successSoft : Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 11
            }
        }

        Text {
            anchors.centerIn: parent
            width: parent.width - 2 * Theme.spacingMedium
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: root.controller.previewLoading
                  ? qsTr("Calculando o plano...")
                  : (root.controller.missingRequiredParam() !== ""
                     ? qsTr("Informe: %1").arg(root.controller.missingRequiredParam())
                     : qsTr("Esta ação não altera arquivo nenhum."))
            color: Theme.textMuted
            font.pixelSize: 11
            visible: root.file === null && root.controller.previewReport.length === 0
        }
    }

    Column {
        id: footer

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Repeater {
            model: root.controller.previewNotes

            Text {
                required property string modelData

                width: footer.width
                text: "⚠ " + modelData
                color: Theme.warningSoft
                font.pixelSize: 10
                wrapMode: Text.WordWrap
            }
        }

        Repeater {
            model: root.action !== null ? root.action.docs : []

            Text {
                required property var modelData

                width: footer.width
                text: qsTr("docs: %1").arg(modelData.title)
                color: Theme.textMuted
                font.pixelSize: 10
                elide: Text.ElideRight
            }
        }

        Row {
            anchors.right: parent.right
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.action !== null ? qsTr("risco: %1").arg(root.action.risk) : ""
                color: Theme.textMuted
                font.pixelSize: 10
            }

            Rectangle {
                readonly property bool ready: root.controller.previewId !== ""
                                              && root.controller.previewId === root.controller.selectedId

                width: applyText.width + 2 * Theme.spacingMedium
                height: 26
                radius: Theme.radius
                color: !ready ? Theme.surface1
                              : (applyArea.pressed ? Theme.accentDim : Theme.accent)
                border.color: ready ? "transparent" : Theme.borderSoft
                border.width: 1

                Text {
                    id: applyText

                    anchors.centerIn: parent
                    text: qsTr("Aplicar")
                    color: parent.ready ? Theme.background0 : Theme.textDisabled
                    font.pixelSize: 11
                    font.bold: true
                }

                MouseArea {
                    id: applyArea

                    anchors.fill: parent
                    enabled: parent.ready
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.controller.apply()
                }
            }
        }
    }
}
