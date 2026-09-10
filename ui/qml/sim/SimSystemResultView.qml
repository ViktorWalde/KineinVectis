pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O RESULTADO DE UMA CORRIDA DE SISTEMA: a procedencia e o DESENHO.
//
// O quanto o numero e' confiavel mora no `SimSystemAccuracy` — sao os dois
// sinais de exatidao que a `arquitetura/34` §13.5 separa (solucao fechada e
// invariante), e "o quanto isto erra" e "como isto se desenha" sao duas
// perguntas. Este arquivo responde a segunda, mais a procedencia que nunca
// pode faltar: o metodo, o passo e a amostragem.
//
// O grafico tem DOIS modos, os dois em `Canvas` raster 2D — sem GPU, sem tocar
// o invariante do `verificar-appimage.sh`. A trajetoria so' e' oferecida quando
// o CONCEITO declara o par de plano: sem ele nao ha' o que pôr nos dois eixos.
Column {
    id: root

    // O `SimSystemController`.
    property var controller: null

    readonly property var run: root.controller === null ? null : root.controller.runResult
    readonly property var concept: root.controller === null ? null : root.controller.concept

    spacing: Theme.spacingXSmall

    Text {
        text: qsTr("A trajetória")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Rectangle {
        visible: root.controller !== null && root.controller.runError !== ""
        width: root.width
        height: erro.implicitHeight + 2 * Theme.spacingSmall
        radius: Theme.radiusXSmall
        color: Theme.background2
        border.width: 1
        border.color: Theme.errorSoft

        Text {
            id: erro

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WordWrap
            text: root.controller === null ? "" : root.controller.runError
            color: Theme.textSecondary
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }

    // Numero sem procedencia mente: o metodo e o passo aparecem sempre.
    Text {
        visible: root.run !== null
        text: root.run === null ? "" : root.run.methodLabel
        color: Theme.textSecondary
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeStatus
    }

    // A AMOSTRAGEM VAI A TELA: uma tabela amostrada parece completa.
    Text {
        visible: root.run !== null
        width: root.width
        wrapMode: Text.WordWrap
        text: root.run === null ? "" :
              qsTr("%1 passos; mostrando 1 a cada %2.")
                .arg(root.run.stepsTaken).arg(root.run.sampleEvery)
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    // AS UNIDADES, uma por componente. Aqui a checagem vale mais que na forma
    // escalar: sao `n` equacoes, e trocar a derivada de uma POSICAO pela de uma
    // VELOCIDADE passa no checador de ligacao — ele confere ligacao, nao fisica.
    SimDimensionsView {
        visible: root.run !== null
        width: root.width
        checks: root.run === null || root.run.dimensions === undefined
                ? [] : root.run.dimensions
        note: root.run === null || root.run.oracleNote === undefined
              ? "" : root.run.oracleNote
    }

    SimSystemAccuracy {
        width: root.width
        run: root.run
        concept: root.concept
        oracleNote: root.run === null || root.run.oracleNote === undefined
                    ? "" : root.run.oracleNote
    }

    // --- o desenho ----------------------------------------------------------
    Row {
        visible: root.run !== null
        spacing: Theme.spacingXSmall

        Repeater {
            model: [
                { id: "components", label: qsTr("componentes no tempo") },
                { id: "trajectory", label: qsTr("trajetória no plano") }
            ]

            delegate: Rectangle {
                id: aba

                required property var modelData

                // A trajetoria so' existe quando o CONCEITO declara o par de
                // plano. Sem ele nao ha' o que pôr nos dois eixos, e oferecer a
                // aba seria prometer um desenho que nao sai.
                readonly property bool disponivel:
                    aba.modelData.id === "components"
                    || (root.controller !== null && root.controller.plane !== null)
                readonly property bool ativa:
                    root.controller !== null && root.controller.plotMode === aba.modelData.id

                visible: aba.disponivel
                width: nome.implicitWidth + 2 * Theme.spacingSmall
                height: 26
                radius: Theme.radiusXSmall
                color: aba.ativa ? Theme.accent : Theme.surface1
                border.width: 1
                border.color: aba.ativa ? Theme.accentActive : Theme.borderSoft

                Text {
                    id: nome

                    anchors.centerIn: parent
                    text: aba.modelData.label
                    color: aba.ativa ? Theme.background0 : Theme.textSecondary
                    font.family: Theme.uiFont
                    font.pixelSize: Theme.fontSizeStatus
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.controller.setPlotMode(aba.modelData.id)
                }
            }
        }
    }

    SimPlotSystem {
        visible: root.run !== null
        width: root.width
        height: 220
        trail: root.run === null ? [] : root.run.trail
        plane: root.controller === null ? null : root.controller.plane
        mode: root.controller === null ? "components" : root.controller.plotMode
        componentLabels: root.concept === null ? []
                                              : root.concept.components.map(c => c.label)
    }

    // A trilha em numeros, para conferir.
    Rectangle {
        visible: root.run !== null
        width: root.width
        height: Math.min(180, Math.max(40, lista.contentHeight + 2))
        radius: Theme.radiusXSmall
        color: Theme.backgroundEditor
        border.width: 1
        border.color: Theme.borderSoft

        ListView {
            id: lista

            anchors.fill: parent
            anchors.margins: Theme.spacingXSmall
            clip: true
            model: root.run === null ? [] : root.run.trail

            delegate: Text {
                id: amostra

                required property var modelData

                function corpo() {
                    let saida = "t=" + SimFormat.number(amostra.modelData.t);
                    const valores = amostra.modelData.values;
                    for (let i = 0; i < valores.length; ++i) {
                        const nome = root.concept === null
                                     || root.concept.components[i] === undefined
                                     ? String(i) : root.concept.components[i].id;
                        saida += "   " + nome + "=" + SimFormat.number(valores[i]);
                    }
                    return saida;
                }

                text: amostra.corpo()
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeStatus - 1
            }
        }
    }
}
