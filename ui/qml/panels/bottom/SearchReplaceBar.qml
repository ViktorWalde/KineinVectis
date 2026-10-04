import QtQuick
import KineinVectis

// A linha de "substituir em todo o projeto", com a confirmacao em DOIS PASSOS.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). O SearchPanel tinha 325 linhas e
// juntava tres coisas: a barra de consulta, a lista de resultados e esta barra
// — que e' a unica com ESTADO e a unica DESTRUTIVA. Substituir em todo o
// projeto reescreve arquivos que o autor talvez nem tenha aberto; a guarda
// contra isso e' regra desta barra, nao do painel que a hospeda.
//
// A GUARDA: o primeiro clique ARMA (o botao vira "Confirmar", em vermelho) e
// so' o segundo executa. Editar a consulta ou o texto de substituicao
// DESARMA — senao um clique herdado de uma busca anterior aplicaria a nova.
//
// O rotulo diz "vazio remove" porque substituicao vazia e' apagamento legitimo
// e nao pode parecer engano; "\n" no texto vira quebra de linha (o core
// interpreta a sequencia, ver DocsPublic/arquitetura/33 §8).
Row {
    id: root

    // Enquanto a substituicao roda, o botao esfria e nao aceita clique.
    property bool busy: false

    readonly property alias replacement: replaceInput.text
    readonly property bool armed: replaceButton.replaceArmed

    signal replaceRequested(string replacement)

    height: visible ? 30 : 0
    spacing: Theme.spacingSmall

    function focusInput() {
        replaceInput.forceActiveFocus();
        replaceInput.selectAll();
    }

    // Chamado tambem quando a CONSULTA muda: a confirmacao pendente vale para
    // a busca que o autor tinha na tela, nao para a proxima.
    function disarm() {
        replaceButton.replaceArmed = false;
    }

    KvTextField {
        id: replaceInput

        width: root.width - replaceButton.width - Theme.spacingSmall
        height: 30
        pixelSize: Theme.fontSizeTerminal
        placeholder: qsTr("Substituir por (vazio remove) — \\n quebra linha")
        onTextChanged: root.disarm()
    }

    // Dois cliques: o primeiro arma (vermelho, "Confirmar"), o segundo
    // substitui.
    KvButton {
        id: replaceButton

        property bool replaceArmed: false

        height: 30
        primary: !replaceButton.replaceArmed
        danger: replaceButton.replaceArmed
        enabled: !root.busy
        text: root.busy ? qsTr("Substituindo...")
              : (replaceButton.replaceArmed ? qsTr("Confirmar") : qsTr("Substituir tudo"))
        onClicked: {
            if (!replaceButton.replaceArmed) {
                replaceButton.replaceArmed = true;
                return;
            }
            replaceButton.replaceArmed = false;
            root.replaceRequested(replaceInput.text);
        }
    }
}
