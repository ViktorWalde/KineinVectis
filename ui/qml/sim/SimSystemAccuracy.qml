pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// OS DOIS SINAIS DE EXATIDAO de uma corrida de sistema.
//
// Eles sao coisas DIFERENTES, e a `arquitetura/34` §13.5 os separa por isso:
//
//   SOLUCAO FECHADA  quando existe. O erro de verdade, componente a componente,
//                    mais a norma do maximo — que e' a grandeza em que a ordem
//                    de convergencia se mede, porque grandeza DERIVADA cancela
//                    erro (o raio da orbita circular da' ordem 5 num metodo de
//                    ordem 4)
//   INVARIANTE       uma grandeza que a fisica CONSERVA. Inicial, final e a
//                    DERIVA — e a tela NAO a chama de erro, porque nao e'
//
// O segundo existe porque o primeiro cobre pouco aqui: a orbita so' tem forma
// fechada no caso circular e o pendulo duplo nao tem nenhuma. E o limite vai
// junto, dito na tela: invariante conservado NAO significa resultado certo.
//
// Arquivo proprio desde 2026-09-07, quando a catraca reprovou o
// `SimSystemResultView` em 324/300. O corte foi por RESPONSABILIDADE e nao por
// tamanho: "o quanto isto e' confiavel" e "como isto se desenha" sao duas
// perguntas, e a §13.5 ja' as tratava separadas no desenho.
Column {
    id: root

    // O resultado de `sim.runSystem`.
    property var run: null
    // O conceito, de onde saem os nomes dos componentes e dos invariantes.
    property var concept: null
    // Por que o valor exato responde pelo CONCEITO e nao pelas equacoes
    // digitadas. Hoje ele sempre responde: o oraculo resolve EDO escalar, e
    // `dsolve` sobre sistema nao foi medido.
    property string oracleNote: ""

    spacing: Theme.spacingXSmall

    // --- primeiro sinal: a solucao fechada, quando ela existe ---------------
    Column {
        visible: root.run !== null && root.run.accuracy !== undefined
                 && root.run.accuracy !== null
        width: root.width
        spacing: 1

        Text {
            text: qsTr("erro contra a solução exata, por componente")
            color: Theme.textMuted
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }

        Repeater {
            model: root.run === null || !root.run.accuracy ? []
                                                           : root.run.accuracy.absoluteError

            delegate: Row {
                id: linhaErro

                required property var modelData
                required property int index

                spacing: Theme.spacingSmall

                Text {
                    width: 60
                    elide: Text.ElideRight
                    text: root.concept === null
                          || root.concept.components[linhaErro.index] === undefined
                          ? String(linhaErro.index)
                          : root.concept.components[linhaErro.index].id
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeStatus
                }

                Text {
                    text: SimFormat.number(linhaErro.modelData)
                    color: Theme.textSecondary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeStatus
                }
            }
        }

        // A NORMA DO MAXIMO em destaque, e o rotulo diz qual grandeza e'. Medir
        // ordem numa grandeza DERIVADA (raio, energia) cancela erro: o raio da
        // orbita circular da' ordem 5,00 para um metodo de ordem 4
        // (`arquitetura/34` §13.4).
        Text {
            text: root.run === null || !root.run.accuracy ? "" :
                  qsTr("maior erro   %1").arg(SimFormat.number(root.run.accuracy.maxError))
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
            font.bold: true
        }
    }

    // A PROCEDENCIA, pela mesma razao da forma escalar: a coluna acima vem da
    // solucao do CONCEITO, e o checador aprova as equacoes por ligacao, nao por
    // fisica. Se o autor escreveu outra equacao, o numero e' a distancia ate'
    // outra pergunta.
    SimAccuracyProvenance {
        visible: root.run !== null && root.run.accuracy !== undefined
                 && root.run.accuracy !== null
        width: root.width
        source: root.run === null || !root.run.accuracy ? "" : root.run.accuracy.source
        note: root.oracleNote
    }

    Text {
        visible: root.run !== null && (root.run.accuracy === undefined
                                       || root.run.accuracy === null)
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("Este conceito não tem solução fechada conhecida neste caso, então "
                   + "a IDE não sabe dizer o quanto este número erra.")
        color: Theme.warningSoft
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    // --- segundo sinal: o que a fisica CONSERVA -----------------------------
    Column {
        visible: root.run !== null && root.run.invariants !== undefined
                 && root.run.invariants.length > 0
        width: root.width
        spacing: 1

        Text {
            width: root.width
            wrapMode: Text.WordWrap
            text: qsTr("o que a física conserva — deriva ao longo da corrida")
            color: Theme.textMuted
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }

        Repeater {
            model: root.run === null || root.run.invariants === undefined
                   ? [] : root.run.invariants

            delegate: Text {
                id: linhaInvariante

                required property var modelData

                function rotulo(id) {
                    if (root.concept === null || root.concept.invariants === undefined) {
                        return id;
                    }
                    for (let i = 0; i < root.concept.invariants.length; ++i) {
                        if (root.concept.invariants[i].id === id) {
                            return root.concept.invariants[i].label;
                        }
                    }
                    return id;
                }

                width: root.width
                wrapMode: Text.WordWrap
                text: linhaInvariante.rotulo(linhaInvariante.modelData.id) + ":  "
                      + SimFormat.number(linhaInvariante.modelData.initial) + "  →  "
                      + SimFormat.number(linhaInvariante.modelData.finalValue)
                      + qsTr("   (deriva %1)")
                          .arg(SimFormat.number(linhaInvariante.modelData.drift))
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeStatus
            }
        }

        // O limite dito junto com o numero, e nao em nota de rodape: um erro que
        // respeita a simetria conservada passa por este sinal inteiro.
        Text {
            width: root.width
            wrapMode: Text.WordWrap
            text: qsTr("Deriva pequena não significa resultado certo: um erro que "
                       + "respeita a grandeza conservada passa por aqui sem aparecer.")
            color: Theme.textMuted
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus - 1
        }
    }
}
