import QtQuick

// O FUNDO de um dialogo, menu ou painel de ambiente (2026-10-03, pedido do
// autor: "quando eu rolar o scroll do mouse dentro de um painel, nao rolar a
// parte do codigo tambem").
//
// Uma MouseArea so' aceita a roda do mouse se declarar `onWheel`. Sem isso o
// Qt entrega a roda ao proximo item por baixo — o editor —, e a rolagem
// atravessava o painel (ou chegava ao codigo quando a lista do painel
// batia no fim). Aqui a roda morre; o clique fica com quem usa (`onClicked`
// fecha o dialogo, por exemplo).
//
// Dentro da CAIXA de um dialogo sem fundo (paleta, renomear...), use com
// `acceptedButtons: Qt.NoButton`: o clique passa, so' a roda e' segurada.
//
// O PAIRAR TAMBEM MORRE AQUI (2026-10-04, relato do autor: com o menu
// Arquivo aberto, passar o mouse acendia as pastas da arvore por baixo, nas
// frestas entre os itens e fora do menu). `hoverEnabled` faz o fundo ficar
// com o pairar; o que esta' POR CIMA dele (o menu, o dialogo) continua
// recebendo — por isso ele e' sempre declarado antes do conteudo.
MouseArea {
    anchors.fill: parent
    acceptedButtons: Qt.LeftButton | Qt.RightButton
    hoverEnabled: true
    onWheel: wheel => wheel.accepted = true
}
