// UM GESTO PRIMARIO POR ESTADO — no painel, nao so' na regra.
//
// Por que existe (2026-09-26): `GrafanaActionRules` ja' tinha harness e dizia
// certo, e mesmo assim a tela mostrava DOIS gestos para a mesma intencao no
// estado "o servidor pediu token": o botao `Fornecer token`, vindo da acao
// primaria, e o campo de token logo abaixo com o seu proprio `Sondar`. A regra
// estava certa; a fiacao e' que desenhava as duas. Nenhum gate via — qmllint
// acha o QML impecavel, porque ele E' impecavel.
//
// Foi a cena de inspecao (os quatro estados lado a lado, em PNG) que mostrou.
// Este teste e' essa cena virada assercao, para nao depender de eu olhar.
//
// MUTACOES QUE PROVAM O GATE: tire o `visible: root.acao.kind !== "provideToken"`
// da Row `grafanaPrimaria` no GrafanaPanel.qml e a terceira assercao cai; troque
// as ancoras da Column `coluna` por `anchors.fill: parent` e a dos achados cai.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 560
    height: 640

    GrafanaController {
        id: controlador

        panelVisible: true
    }

    GrafanaPanel {
        id: painel

        anchors.fill: parent
        controller: controlador
    }

    // Procura por `objectName` e nao por ordem de filhos: a coluna do painel
    // muda de forma a cada fatia, e um indice fixo mentiria em silencio.
    function achar(item, nome) {
        if (item.objectName === nome) {
            return item;
        }
        for (let indice = 0; indice < item.children.length; ++indice) {
            const achado = root.achar(item.children[indice], nome);
            if (achado !== null) {
                return achado;
            }
        }
        return null;
    }

    // Visivel de verdade: um pai invisivel esconde o filho que se diz visivel.
    function mostrado(item) {
        return item !== null && item.visible && item.height > 0;
    }

    // `Qt.exit` NAO INTERROMPE: ele marca o codigo e a funcao segue. Uma
    // assercao que chamasse `Qt.exit(1)` no meio teria o codigo apagado pelo
    // `Qt.exit(0)` do fim — um gate que imprime FALHOU e devolve sucesso.
    // Mesmo idioma do `tst_grafana_state`: acumula e sai uma vez so'.
    property int falhas: 0

    function conferir(condicao, mensagem) {
        if (!condicao) {
            console.error("FALHOU: " + mensagem);
            root.falhas += 1;
        }
    }

    Component.onCompleted: {
        const primaria = root.achar(painel, "grafanaPrimaria");
        const pedido = root.achar(painel, "grafanaTokenPrompt");
        if (primaria === null || pedido === null) {
            // Sem os dois nao ha' o que comparar: as assercoes seguintes
            // estourariam em `null.visible` e esconderiam esta causa.
            console.error("FALHOU: o painel nao tem mais grafanaPrimaria/grafanaTokenPrompt");
            Qt.exit(1);
            return;
        }

        // 1 · primeira abertura: um botao, nenhum campo de token.
        root.conferir(root.mostrado(primaria),
                      "sem instancia, a acao primaria precisa estar na tela");
        root.conferir(!root.mostrado(pedido),
                      "sem instancia, o campo de token nao foi pedido por ninguem");

        // 2 · o servidor pediu token: o campo E' a acao, e o botao sai.
        controlador.handleProfile({ url: "http://grafana.lab:3000", tokenSource: "none" }, true);
        controlador.handleFailed("grafana.probe", "token exigido", "SECRET_REQUIRED");
        root.conferir(root.mostrado(pedido),
                      "o servidor pediu token e o campo nao apareceu");
        root.conferir(!root.mostrado(primaria),
                      "token pedido: o botao primario e o campo estavam na tela juntos");
        root.conferir(pedido.labelText === controlador.primaryAction.label,
                      "o botao do campo nao fala a lingua do estado");
        root.conferir(pedido.reasonText !== "",
                      "o campo de token apareceu sem dizer por que");

        // 3 · token recusado: continua sendo o campo, com outra frase.
        const antes = pedido.labelText;
        controlador.probeWithToken("errado");
        controlador.handleFailed("grafana.probe", "token exigido", "SECRET_REQUIRED");
        root.conferir(root.mostrado(pedido) && !root.mostrado(primaria),
                      "token recusado: de novo dois gestos na tela");
        root.conferir(pedido.labelText !== antes,
                      "recusado e pedido diziam a mesma coisa");

        // 4 · medido com sucesso: volta o botao, some o campo, e os ACHADOS
        // cabem na tela. A lista tinha altura ZERO desde sempre, porque a
        // coluna do formulario preenchia o pai inteiro — sem erro e sem
        // warning, so' um espaco preto no lugar do que a IDE foi buscar.
        controlador.handleProbed({
            reachable: true, authenticated: true, version: "11.2.0",
            dataSources: [{ name: "postgres-kinein", typeName: "PostgreSQL",
                            typeId: "postgres", url: "localhost:5432", isDefault: true }],
            dashboards: [{ title: "fila", folderTitle: "Dev", url: "/d/abc/fila" }],
            matches: [{ profileName: "kinein_dev", dataSourceName: "postgres-kinein",
                        reason: "mesmo host e mesma base" }]
        });
        const achados = root.achar(painel, "grafanaAchados");
        root.conferir(achados !== null && achados.height >= 60,
                      "a area dos achados nao tem altura para mostrar o cruzamento (altura="
                      + (achados === null ? "ausente" : achados.height) + ")");
        root.conferir(root.mostrado(primaria),
                      "depois de medir, a acao primaria precisa voltar");
        root.conferir(!root.mostrado(pedido),
                      "o campo de token ficou na tela depois de autenticar");

        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
