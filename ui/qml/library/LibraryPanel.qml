pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O catalogo de bibliotecas na tela: escolher, ENTENDER e aplicar.
//
// Componente burro: recebe por property, pede por signal. O "entender" nao e
// enfeite — e a razao do painel existir. Quem nao sabe qual biblioteca usar nao
// e ajudado por um inventario, entao cada linha mostra o que a biblioteca FAZ,
// a licenca, a versao PINADA e a data do release. Idade visivel e' informacao:
// uma lib parada ha' tres anos pode ser madura ou abandonada, e quem decide e'
// quem le, nao a IDE.
Item {
    id: root

    property var libraries: []
    property string selectedId: ""
    property string target: ""
    property var targets: []
    property string targetsOrigin: ""
    property var plan: null
    property string errorText: ""

    signal librarySelected(string id)
    signal targetEdited(string name)
    signal closeRequested()
    signal applyStepRequested(string actionId, var params)

    Text {
        id: titulo

        anchors.top: parent.top
        anchors.left: parent.left
        text: qsTr("Bibliotecas")
        color: Theme.textPrimary
        font.pixelSize: 12
        font.weight: Font.DemiBold
    }

    Text {
        id: subtitulo

        anchors.top: titulo.bottom
        anchors.topMargin: 2
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        text: qsTr("● verde = já está no seu projeto.  ● cinza = disponível "
                   + "para ativar.  Licença verificada na fonte, versão fixada.")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    LibraryTargetPicker {
        id: alvoBox

        anchors.top: subtitulo.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right

        targets: root.targets
        origin: root.targetsOrigin
        target: root.target

        onTargetEdited: name => root.targetEdited(name)
    }

    ListView {
        id: lista

        anchors.top: alvoBox.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: rodape.top
        anchors.bottomMargin: Theme.spacingSmall
        clip: true
        spacing: 2
        model: root.libraries

        delegate: Rectangle {
            id: linha

            required property var modelData

            width: lista.width
            height: 58
            radius: Theme.radius
            color: linha.modelData.id === root.selectedId ? Theme.surface2 : "transparent"

            MouseArea {
                anchors.fill: parent
                onClicked: root.librarySelected(linha.modelData.id)
            }

            // A BOLINHA responde "esta no meu projeto?", nao "existe nesta
            // maquina?". As duas perguntas sao diferentes e a tela mostrava so'
            // a segunda — foi a confusao que o autor relatou em 2026-09-04.
            Rectangle {
                id: bolinha

                anchors.verticalCenter: nome.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                width: 8
                height: 8
                radius: 4
                color: linha.modelData.applied === true
                       ? Theme.successSoft : Theme.textDisabled
            }

            Text {
                id: nome

                anchors.top: parent.top
                anchors.topMargin: 4
                anchors.left: bolinha.right
                anchors.leftMargin: Theme.spacingSmall
                text: linha.modelData.name
                color: Theme.textPrimary
                font.pixelSize: 11
                font.weight: Font.DemiBold
            }

            // O sinal FORTE, quando existe. Nao ha certificacao oficial de
            // biblioteca C++; o que existe e' o comite ter adotado a lib no
            // padrao. Quando isso aconteceu, e' o que mais importa na linha.
            Text {
                anchors.verticalCenter: nome.verticalCenter
                anchors.left: nome.right
                anchors.leftMargin: Theme.spacingSmall
                visible: linha.modelData.standardLineage !== undefined
                         && linha.modelData.standardLineage !== ""
                text: "★ " + (linha.modelData.standardLineage || "")
                color: Theme.accent
                font.pixelSize: 10
            }

            Text {
                anchors.top: parent.top
                anchors.topMargin: 4
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                // Duas informacoes, nesta ordem de importancia: se ja' esta
                // NO PROJETO (o que o autor quer saber primeiro) e, so' quando
                // nao esta, de onde ela viria. "seria baixada" sozinho nao
                // dizia nem uma coisa nem outra — relato de 2026-09-04.
                text: linha.modelData.applied === true
                      ? qsTr("ativa neste projeto")
                      : (linha.modelData.status === "detected"
                         ? qsTr("instalada no sistema")
                         : qsTr("baixa junto do projeto"))
                color: linha.modelData.applied === true
                       ? Theme.successSoft : Theme.textMuted
                font.pixelSize: 10
            }

            Text {
                id: resumo

                anchors.top: nome.bottom
                anchors.topMargin: 1
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                elide: Text.ElideRight
                text: linha.modelData.summary
                color: Theme.textMuted
                font.pixelSize: 10
            }

            Text {
                id: ficha

                anchors.top: resumo.bottom
                anchors.topMargin: 2
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                text: linha.modelData.license + " · " + linha.modelData.pinnedVersion
                      + " · " + linha.modelData.releasedAt
                color: Theme.textMuted
                font.pixelSize: 10
            }

            // O REPOSITORIO a um clique, para o autor olhar ANTES de ativar.
            //
            // Pedido explicito em 2026-09-04: "o link do github deveria ficar
            // acessivel la na biblioteca, para o usuario olhar antes de clicar
            // em ativar". Ate' entao o catalogo TINHA a URL e a tela nunca a
            // mostrava — auditar uma dependencia exigia procurar fora da IDE.
            Row {
                anchors.verticalCenter: ficha.verticalCenter
                anchors.left: ficha.right
                anchors.leftMargin: Theme.spacingSmall
                spacing: Theme.spacingSmall

                Repeater {
                    model: [
                        { rotulo: qsTr("repositório"), url: linha.modelData.repository },
                        { rotulo: qsTr("documentação"), url: linha.modelData.documentation }
                    ]

                    delegate: Text {
                        id: atalhoExterno

                        required property var modelData

                        visible: atalhoExterno.modelData.url !== undefined
                                 && atalhoExterno.modelData.url !== ""
                        text: atalhoExterno.modelData.rotulo + " ↗"
                        color: linkArea.containsMouse ? Theme.accent : Theme.textSecondary
                        font.pixelSize: 10
                        font.underline: linkArea.containsMouse

                        MouseArea {
                            id: linkArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: Qt.openUrlExternally(atalhoExterno.modelData.url)
                        }
                    }
                }
            }
        }
    }

    LibraryPlanView {
        id: rodape

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 96

        plan: root.plan
        errorText: root.errorText
        hasTarget: root.target !== ""
        hasSelection: root.selectedId !== ""

        onApplyStepRequested: function(actionId, params) {
            root.applyStepRequested(actionId, params);
        }
    }
}
