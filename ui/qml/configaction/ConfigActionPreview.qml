pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Painel lateral do preview (spec 9.1 §12): o que faz, os campos, o DIFF, a
// documentacao e o Ativar.
//
// QUEM CEDE ESPACO (2026-09-04). Ate' aqui o diff nascia com o que SOBRASSE:
// os campos em cima, o rodape embaixo, e a previa herdava o resto. Quando os
// campos ganharam descricao e chips de sugestao, o resto encolheu — e a previa
// do CMakeLists, a unica coisa que o usuario le antes de consentir, virou uma
// janelinha de nove linhas. Relato de uso do autor.
//
// A ordem se inverteu: o diff tem PISO (o cabecalho nao passa de 40% do
// painel) e sao os campos que rolam quando nao cabem. Cede espaco quem ja' foi
// lido, nao quem ainda precisa ser lido.
Item {
    id: root

    property var controller: null

    readonly property var action: root.controller.selectedAction
    readonly property var file: root.controller.previewFiles.length > 0
                                ? root.controller.previewFiles[0] : null

    Flickable {
        id: headerScroll

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: Math.min(header.implicitHeight, root.height * 0.4)
        contentHeight: header.implicitHeight
        contentWidth: width
        boundsBehavior: Flickable.StopAtBounds
        clip: true

        Column {
            id: header

            width: headerScroll.width
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
            // Cada campo mostra o que FAZ e o que o projeto oferece; ver
            // ConfigActionField.qml.
            Repeater {
                model: root.action !== null ? root.action.params : []

                delegate: ConfigActionField {
                    id: campo

                    required property var modelData

                    width: header.width
                    param: campo.modelData
                    value: root.controller.paramValue(campo.modelData.name)

                    onEdited: texto => {
                        root.controller.setParam(campo.modelData.name, texto);
                        root.controller.requestPreview();
                    }
                }
            }
        }

        VerticalScrollBar {
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom

            contentSize: headerScroll.contentHeight
            viewportSize: headerScroll.height
            position: headerScroll.contentY

            onMoveRequested: function (position) {
                headerScroll.contentY = position;
            }
        }
    }

    ConfigActionDiffView {
        anchors.top: headerScroll.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: footer.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right

        file: root.file
        reportLines: root.controller.previewReport
        loading: root.controller.previewLoading
        missingParam: root.controller.missingRequiredParam()
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
                // A palavra crua do enum ("medium") nao dizia nada a quem
                // clica. A frase vem do core, ao lado da definicao do valor.
                text: root.action !== null ? root.action.riskExplanation : ""
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
                    // O mesmo par de verbos do painel de bibliotecas: quem
                    // desfaz nao "aplica", desativa.
                    text: root.action !== null
                          && root.action.id === "cmake.removeTargetLinkLibraries"
                          ? qsTr("Desativar") : qsTr("Ativar")
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
