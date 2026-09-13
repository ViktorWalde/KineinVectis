// O BOTAO PRIMARIO DESLIGADO NAO PODE PARECER LIGADO.
//
// Por que existe (2026-09-13): o "compose up" do painel de containers e'
// primario (ambar) e fica desligado sem projeto. Com 72% de opacidade o ambar
// continuava a coisa mais chamativa do painel; o clique nao fazia nada e a
// leitura do autor foi "a parte de Docker nao esta' dando certo". Nao ha' erro,
// nao ha' warning: e' a classe de falha silenciosa que so' um teste de estado
// visual pega.
//
// MUTACAO QUE PROVA O GATE: troque `accented` por `primary` na cor de fundo do
// KvButton.qml e a primeira assercao cai (o desligado volta a vestir o acento).
import QtQuick
import KineinVectis

Item {
    id: root

    width: 400
    height: 200

    KvButton { id: ligado; text: "compose up"; iconName: "run"; primary: true; enabled: true }
    KvButton { id: desligado; text: "compose up"; iconName: "run"; primary: true; enabled: false; y: 40 }
    KvButton { id: comum; text: "Atualizar"; iconName: "refresh"; enabled: false; y: 80 }

    // A mesma regra vale para o botao SO' DE ICONE (a fileira de acoes de um
    // container): desligado, nem acento, nem vermelho — apagado.
    KvIconButton { id: iconePerigoLigado; iconName: "close"; danger: true; enabled: true; x: 200 }
    KvIconButton { id: iconePerigoDesligado; iconName: "close"; danger: true; enabled: false; x: 240 }
    KvIconButton { id: iconePrimarioDesligado; iconName: "run"; primary: true; enabled: false; x: 280 }
    KvIconButton { id: iconeComumDesligado; iconName: "file"; enabled: false; x: 320 }

    Component.onCompleted: {
        let failures = 0;

        // Ligado veste o acento e nao tem borda; desligado e' um botao comum.
        if (!Qt.colorEqual(ligado.color, Theme.accent) || ligado.border.width !== 0) failures += 1;
        if (Qt.colorEqual(desligado.color, Theme.accent)) failures += 2;
        if (!Qt.colorEqual(desligado.color, comum.color)) failures += 4;
        if (desligado.border.width !== 1) failures += 8;
        // E o texto apagado, nao o texto escuro-sobre-ambar do primario.
        const textoDesligado = root.textoDe(desligado);
        const textoComum = root.textoDe(comum);
        if (textoDesligado === null || textoComum === null) failures += 16;
        else if (!Qt.colorEqual(textoDesligado.color, textoComum.color)
                 || !Qt.colorEqual(textoDesligado.color, Theme.textDisabled)) failures += 32;
        // A opacidade continua marcando o desligado (o comum ja' era assim).
        if (desligado.opacity >= 1.0 || ligado.opacity !== 1.0) failures += 64;
        // O icone acompanha: escuro-sobre-ambar so' no ligado; apagado nos
        // desligados (antes, o desligado comum ficava com icone de ligado).
        const iconeLigado = root.iconeDe(ligado);
        const iconeDesligado = root.iconeDe(desligado);
        const iconeComum = root.iconeDe(comum);
        if (iconeLigado === null || iconeDesligado === null || iconeComum === null) failures += 128;
        else {
            if (!Qt.colorEqual(iconeLigado.iconColor, Theme.background0)) failures += 256;
            if (!Qt.colorEqual(iconeDesligado.iconColor, Theme.textDisabled)) failures += 512;
            if (!Qt.colorEqual(iconeComum.iconColor, Theme.textDisabled)) failures += 1024;
        }

        const iconeDe = b => b.children[0];
        if (!Qt.colorEqual(iconeDe(iconePerigoLigado).iconColor, Theme.errorSoft)) failures += 2048;
        if (!Qt.colorEqual(iconeDe(iconePerigoDesligado).iconColor, Theme.textDisabled)) failures += 4096;
        if (!Qt.colorEqual(iconeDe(iconePrimarioDesligado).iconColor, Theme.textDisabled)
                || Qt.colorEqual(iconePrimarioDesligado.color, Theme.accent)) failures += 8192;
        if (!Qt.colorEqual(iconeDe(iconeComumDesligado).iconColor, Theme.textDisabled)) failures += 16384;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }

    function iconeDe(botao) {
        for (let i = 0; i < botao.children.length; i++) {
            const filho = botao.children[i];
            for (let j = 0; j < filho.children.length; j++) {
                if (filho.children[j].hasOwnProperty("iconColor")) return filho.children[j];
            }
        }
        return null;
    }

    function textoDe(botao) {
        for (let i = 0; i < botao.children.length; i++) {
            const filho = botao.children[i];
            for (let j = 0; j < filho.children.length; j++) {
                if (filho.children[j].hasOwnProperty("font") && filho.children[j].hasOwnProperty("text"))
                    return filho.children[j];
            }
        }
        return null;
    }
}
