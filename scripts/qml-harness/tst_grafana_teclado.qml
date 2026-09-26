// TECLADO E TAMANHO DO PAINEL DO GRAFANA (G4: "teste de tab order, Enter,
// Escape, setas e resize").
//
// Tres coisas que so' se veem com o painel montado de verdade:
//
// 1. ESC FECHA. A `KvPanelFrame` — moldura dos CINCO paineis de ambiente —
//    sempre soube dispensar por clique fora e nunca ouviu o teclado: quem
//    abria por `Ctrl+Alt+O` tinha de ir ao mouse para sair.
// 2. O FOCO CAI ONDE SERVE ao abrir: no endereco no primeiro uso, no filtro
//    no uso diario.
// 3. A AREA DOS ACHADOS NAO COLAPSA em tamanho nenhum. Ela ja' nasceu com
//    altura negativa uma vez (§7.109), sem erro e sem warning.
//
// MUTACAO QUE PROVA O GATE: troque o `ajustes.visible` do `takeFocus` por
// `false` e a assercao do primeiro uso cai; tire o `campoFiltro.forceActiveFocus`
// e a do uso diario cai.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 900
    height: 800

    property int falhas: 0
    property int dispensas: 0

    GrafanaController {
        id: controlador

        workspaceRoot: "/tmp/projeto"
        panelVisible: true
    }

    KvPanelFrame {
        id: moldura

        anchors.fill: parent
        panelWidth: 560
        panelHeight: 620
        maxAvailableWidth: root.width
        maxAvailableHeight: root.height

        onDismissRequested: root.dispensas += 1

        GrafanaPanel {
            id: painel

            anchors.fill: parent
            controller: controlador
        }
    }

    function conferir(condicao, mensagem) {
        if (!condicao) {
            console.error("FALHOU: " + mensagem);
            root.falhas += 1;
        }
    }

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

    // Quem tem o foco agora. `focus` e nao `activeFocus`: a janela offscreen
    // do harness nunca fica ATIVA, entao `activeFocus` e' falso em tudo mesmo
    // depois de `forceActiveFocus` — o que se mede aqui e' para onde o painel
    // MANDOU o foco, que e' a decisao dele.
    function focadoDentroDe(item) {
        if (item.focus === true && item.children.length === 0) {
            return item;
        }
        for (let indice = 0; indice < item.children.length; ++indice) {
            const achado = root.focadoDentroDe(item.children[indice]);
            if (achado !== null) {
                return achado;
            }
        }
        return null;
    }

    Component.onCompleted: {
        // 1 · PRIMEIRO USO: o foco vai para o endereco.
        painel.takeFocus();
        const primeiro = root.focadoDentroDe(painel);
        root.conferir(primeiro !== null, "ao abrir sem instancia, nada recebeu o foco");
        if (primeiro !== null) {
            root.conferir(String(primeiro.text).indexOf("localhost") >= 0,
                          "o foco do primeiro uso nao caiu no endereco: " + primeiro.text);
        }

        // 2 · USO DIARIO: com instancia e resultado, o foco vai para o filtro.
        controlador.handleProfile({ url: "http://grafana.lab:3000", tokenSource: "none" }, true);
        controlador.probe();
        controlador.handleProbed({
            reachable: true, authenticated: true, version: "11.2.0",
            dashboards: [{ title: "fila", folderTitle: "Dev", url: "/d/abc" }],
            dataSources: [], matches: []
        });
        painel.takeFocus();
        const diario = root.focadoDentroDe(painel);
        root.conferir(diario !== null && String(diario.text) === "",
                      "o foco do uso diario nao caiu no filtro vazio");

        // 3 · ESC FECHA, e a moldura e' quem ouve.
        // O QUE SE MEDE e' que dispensar pelo teclado e' o MESMO caminho do
        // clique fora. A entrega da tecla em si (`Keys.onEscapePressed`) o
        // harness nao consegue simular — esta' dito no KvPanelFrame.
        moldura.dismissFromKeyboard();
        root.conferir(root.dispensas === 1,
                      "dispensar pelo teclado nao pediu para fechar: " + root.dispensas);

        // 4 · A AREA DOS ACHADOS SOBREVIVE AO TAMANHO. Uma moldura baixa nao
        // pode devolver altura negativa, e uma alta tem de dar espaco util.
        const achados = root.achar(painel, "grafanaAchados");
        root.conferir(achados !== null, "a area dos achados sumiu");
        if (achados !== null) {
            moldura.panelHeight = 240;
            root.conferir(achados.height >= 0,
                          "moldura baixa deu altura negativa aos achados: " + achados.height);
            moldura.panelHeight = 620;
            root.conferir(achados.height > 60,
                          "moldura alta nao deu espaco aos achados: " + achados.height);
            // Estreito: o painel nao pode ficar com largura negativa em campo
            // nenhum, que e' como o QML desenha caixa invisivel sem reclamar.
            moldura.panelWidth = 320;
            root.conferir(achados.width > 0,
                          "moldura estreita zerou a largura dos achados: " + achados.width);
            moldura.panelWidth = 560;
        }

        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
