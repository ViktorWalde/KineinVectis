pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A PREVIA DE MARKDOWN (fatia V5, 2026-09-25).
//
// NAO e' uma imagem do documento: o texto continua selecionavel e copiavel, o
// conteudo vem do BUFFER (inclusive o que ainda nao foi salvo) e os links tem
// politica. A §1 da especificacao diz isso com todas as letras, e e' o que
// separa isto de um "preview" que vira figura de onde nao se copia um comando.
//
// A largura de leitura e' limitada (§3.2): paragrafo esticado pela janela
// inteira e' desconfortavel de ler, e nenhum leitor de documentacao faz isso.
Item {
    id: root

    // O BUFFER como ele esta' agora. NAO vai direto para o renderer: ver o
    // debounce abaixo.
    property string content: ""
    property string documentPath: ""
    property string workspaceRoot: ""
    // O documento da aba. Uma renderizacao atrasada de OUTRA aba nao pode
    // pintar aqui — §10, "resposta atrasada nunca aparece na aba errada".
    property int docId: 0

    // O que o portao das imagens RECUSOU, para a tela poder dizer em vez de
    // deixar um buraco silencioso (§4.2).
    readonly property var blockedResources: ponte.blockedResources

    // Quando o texto chega ao renderer e' regra, e mora no
    // MarkdownRenderGate — que por isso tem harness.
    readonly property alias rendered: gate.rendered
    readonly property alias renderedDocId: gate.renderedDocId

    // Trocar de conteudo apaga a recusa: ela era sobre o documento de antes.
    onContentChanged: root.refusedMessage = ""

    MarkdownRenderGate {
        id: gate

        content: root.content
        docId: root.docId
    }

    // A RECUSA DE UM LINK aparece AQUI, e nao num log distante: quem clicou
    // esta' olhando para esta tela. Silencio seria o mesmo defeito do id sem
    // dono, uma camada adiante.
    property string refusedMessage: ""

    signal localFileRequested(string path)
    signal webUrlRequested(string url)

    MarkdownDocument {
        id: ponte

        target: texto.textDocument
        markdown: root.rendered
        documentPath: root.documentPath
        workspaceRoot: root.workspaceRoot
        // O tema entra como FORMATO, nao como CSS: o `setMarkdown` do Qt monta
        // blocos com propriedades proprias, e folha de estilo nao os alcanca.
        linkColor: Theme.accent
        quoteColor: Theme.textSecondary
        codeBackground: Theme.surface2
    }

    Flickable {
        id: rolagem

        anchors.fill: parent
        contentWidth: width
        contentHeight: texto.height + 2 * Theme.spacingLarge
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        TextEdit {
            id: texto

            // A largura de leitura: 760 px e' o que cabe sem cansar; abaixo
            // disso a janela manda.
            readonly property int larguraDeLeitura: 760

            x: Math.max(Theme.spacingLarge, (rolagem.width - width) / 2)
            y: Theme.spacingLarge
            width: Math.min(root.width - 2 * Theme.spacingLarge, larguraDeLeitura)
            readOnly: true
            selectByMouse: true
            wrapMode: TextEdit.Wrap
            textFormat: TextEdit.RichText
            color: Theme.textPrimary
            selectionColor: Theme.accent
            font.pixelSize: Theme.fontSizeEditor
            // O `text` sai do `MarkdownDocument`, que escreve direto no
            // textDocument: atribuir aqui sobrescreveria o que ele montou.

            onLinkActivated: function(link) {
                const veredito = ponte.decideLink(link);
                switch (veredito.kind) {
                case "anchor":
                    // §4.1: ancora navega dentro do proprio preview. Sem mapa
                    // de ancoras ainda; o que NAO pode e' virar outra coisa.
                    break;
                case "localFile":
                    root.refusedMessage = "";
                    root.localFileRequested(veredito.target);
                    break;
                case "web":
                    root.refusedMessage = "";
                    root.webUrlRequested(veredito.target);
                    break;
                default:
                    root.refusedMessage = veredito.reason;
                    break;
                }
            }
        }
    }

    // O QUE FOI BLOQUEADO APARECE. Um documento com imagem recusada nao pode
    // ficar com um buraco silencioso: a §4.2 pede alt text e motivo.
    Rectangle {
        id: aviso

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        height: visible ? avisoTexto.implicitHeight + 2 * Theme.spacingSmall : 0
        visible: root.blockedResources.length > 0 || root.refusedMessage !== ""
        radius: Theme.radius
        color: Theme.surface2
        border.width: 1
        border.color: Theme.borderSoft

        Text {
            id: avisoTexto

            // NAO e' `anchors.fill`: a altura do retangulo sai do
            // `implicitHeight` deste texto, e preencher o pai faria a medida
            // depender do que ela mesma determina. O gate pegou como
            // "Binding loop detected for property height".
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WordWrap
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeStatus
            text: {
                if (root.refusedMessage !== "") {
                    return qsTr("Link não aberto: %1").arg(root.refusedMessage);
                }
                return qsTr("%n recurso(s) não carregado(s): %1", "",
                            root.blockedResources.length)
                       .arg(root.blockedResources.join("; "));
            }
        }
    }

    Text {
        anchors.centerIn: parent
        visible: root.rendered === ""
        text: qsTr("Nada para mostrar.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeEditor
    }
}
