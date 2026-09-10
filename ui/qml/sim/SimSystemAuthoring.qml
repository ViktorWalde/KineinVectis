pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A MONTAGEM DE UM SISTEMA: `n` equacoes, `n` ligacoes e o estado em t = 0.
//
// A forma escalar tem uma formula e uma tabela; esta tem uma de cada POR
// COMPONENTE, e o que o core cobra do sistema inteiro (toda grandeza
// obrigatoria ligada em ALGUMA equacao) nao e' o que ele cobra de cada uma.
// Por isso ela e' um arquivo, e nao um `visible:` dentro do `SimPanel`.
//
// Ela recebe o CONTROLLER, e nao vinte propriedades soltas: o que a tela le e'
// por componente (a ligacao daquela equacao, as variaveis daquela formula, os
// problemas daquela linha), e achatar isso em propriedades produziria uma lista
// que cresce com o numero de componentes. Quem deriva continua sendo o
// controller — aqui nao se calcula nada.
Column {
    id: root

    // O `SimSystemController`.
    property var controller: null
    // Os valores dos parametros, que moram no `SimController` porque sao os
    // mesmos das outras formas: { idDaGrandeza: texto }.
    property var values: ({})

    signal valueEdited(string quantityId, string text)

    readonly property var concept: root.controller === null ? null : root.controller.concept

    spacing: Theme.spacingXSmall

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        text: root.concept === null ? "" : root.concept.summary
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        // A FONTE, com data — mesma regra do resto: sem fonte, a IDE nao afirma.
        text: root.concept === null ? "" : qsTr("Formulação: %1").arg(root.concept.source)
        color: Theme.textDisabled
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus - 1
    }

    Text {
        text: qsTr("As equações")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
        topPadding: Theme.spacingXSmall
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        // A frase explica a reducao a primeira ordem, que e' a duvida que este
        // formato causa em quem chega: "onde escrevo a segunda derivada?".
        text: qsTr("Este conceito é um sistema de primeira ordem: você escreve a "
                   + "derivada de cada componente. A equação de um componente pode "
                   + "usar qualquer outro — em uma órbita, a derivada de vx depende "
                   + "de x e de y.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Repeater {
        model: root.concept === null ? [] : root.concept.components

        delegate: SimComponentEquation {
            id: linha

            required property var modelData

            // As leituras abaixo sao FUNCOES do controller, e uma funcao sozinha
            // nao cria dependencia de binding. Por isso cada uma cita a
            // propriedade que a faz mudar (`equations`, `checkResult`): sem essa
            // citacao a linha congelaria na primeira checagem e a tela mentiria
            // em silencio, que e' a forma de falha que este dominio persegue.
            width: root.width
            stateComponent: linha.modelData
            formula: {
                const e = root.controller.equations.length === 0
                          ? null : root.controller.equationFor(linha.modelData.id);
                return e === null ? "" : e.formula;
            }
            checkOk: root.controller.checkResult !== null
                     && root.controller.componentOkFor(linha.modelData.id)
            variables: root.controller.checkResult === null
                       ? [] : root.controller.variablesFor(linha.modelData.id)
            issues: root.controller.checkResult === null
                    ? [] : root.controller.issuesFor(linha.modelData.id)
            targets: root.concept === null ? [] : root.controller.bindingTargets()
            bindings: root.controller.equations.length === 0
                      ? ({}) : root.controller.bindingMapFor(linha.modelData.id)

            onEdited: texto => root.controller.setFormula(linha.modelData.id, texto)
            onBindRequested: (variavel, alvo) =>
                root.controller.setBinding(linha.modelData.id, variavel, alvo)
        }
    }

    // O AVISO da §5.1, uma vez so' e no fim: ele aparece quando a checagem
    // PASSA, que e' justamente o momento em que da' para achar que esta' tudo
    // certo.
    Rectangle {
        visible: root.controller !== null && root.controller.caveat !== ""
        width: root.width
        height: aviso.implicitHeight + 2 * Theme.spacingSmall
        radius: Theme.radiusXSmall
        color: Theme.background2
        border.width: 1
        border.color: Theme.warningSoft

        Text {
            id: aviso

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WordWrap
            text: root.controller === null ? "" : root.controller.caveat
            color: Theme.textSecondary
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }

    Text {
        text: qsTr("O estado em t = 0")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
        topPadding: Theme.spacingXSmall
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("Um valor por componente. Campo em branco não vira zero: a "
                   + "integração não parte enquanto faltar um.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Grid {
        columns: 4
        spacing: Theme.spacingSmall
        verticalItemAlignment: Grid.AlignVCenter

        Repeater {
            model: root.concept === null ? [] : root.concept.components

            delegate: SimNumberField {
                id: campo

                required property var modelData
                required property int index

                label: campo.modelData.unit === ""
                       ? campo.modelData.id + "(0)"
                       : campo.modelData.id + "(0)   " + campo.modelData.unit
                text: {
                    const v = root.controller.initialState[campo.index];
                    return v === undefined || v === null ? "" : String(v);
                }
                onEdited: t => root.controller.setInitial(campo.index, t)
            }
        }
    }

    SimValueTable {
        width: root.width
        visible: root.controller !== null && root.controller.equations.length > 0
        // So' as grandezas que o autor LIGOU em alguma equacao pedem valor:
        // componente vem do estado, tempo vem do integrador.
        quantities: root.controller === null || root.controller.equations.length === 0
                    ? [] : root.controller.boundQuantities()
        values: root.values
        onEdited: (grandeza, texto) => root.valueEdited(grandeza, texto)
    }
}
